//! Where plans live. The save runs as one atomic step: idempotency lookup, ownership,
//! version check, validator, status change and storing the key happen under one lock
//! (one transaction in the Postgres store that replaces this in PA#3).

use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::{
    model::{Plan, PlanStatus, PlanView, Trip},
    validator::{Violation, validate},
};

/// How long a stored Idempotency-Key is honoured.
pub const KEY_TTL_HOURS: i64 = 24;

pub struct SaveCommand {
    pub user: Uuid,
    pub plan_id: Uuid,
    pub key: String,
    /// What makes two requests "the same": the plan id and the JSON body.
    pub fingerprint: Value,
    pub expected_version: i64,
}

#[derive(Debug)]
pub enum SaveOutcome {
    /// First successful save, or a repeat on a plan already saved at that version.
    Saved(Value),
    /// Same key, same request: the stored body, unchanged.
    Replayed(Value),
}

#[derive(Debug)]
pub enum SaveFailure {
    NotFound,
    NotOwner,
    VersionConflict {
        current: i64,
    },
    Rejected(Vec<Violation>),
    KeyReused,
    /// Storage broke. The message is logged, never sent to the client.
    Internal(String),
}

pub trait PlanStore: Send + Sync {
    fn save(&self, cmd: SaveCommand) -> Result<SaveOutcome, SaveFailure>;
    /// `GET /plans/{planId}`: the plan as `user` may see it now. Only `NotFound`,
    /// `NotOwner` (a draft read by a member who did not create it) and `Internal` occur.
    fn read(&self, user: Uuid, plan_id: Uuid) -> Result<Value, SaveFailure>;
}

struct StoredKey {
    fingerprint: Value,
    body: Value,
    at: DateTime<Utc>,
}

#[derive(Default)]
struct State {
    trips: HashMap<Uuid, Trip>,
    plans: HashMap<Uuid, Plan>,
    keys: HashMap<(Uuid, String), StoredKey>,
}

/// Where "now" comes from. Tests replace it to move past the key's 24 hours.
pub type Clock = Box<dyn Fn() -> DateTime<Utc> + Send + Sync>;

/// In-memory store: used by the tests and until the Postgres store lands.
pub struct InMemoryPlanStore {
    state: Mutex<State>,
    clock: Clock,
}

impl Default for InMemoryPlanStore {
    fn default() -> Self {
        Self {
            state: Mutex::default(),
            clock: Box::new(Utc::now),
        }
    }
}

impl InMemoryPlanStore {
    pub fn with_clock(clock: Clock) -> Self {
        Self {
            state: Mutex::default(),
            clock,
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn insert_trip(&self, trip: Trip) {
        self.lock().trips.insert(trip.id, trip);
    }

    pub fn insert_plan(&self, plan: Plan) {
        self.lock().plans.insert(plan.id, plan);
    }

    /// A member leaves the trip (or is removed from it).
    pub fn remove_member(&self, trip: Uuid, user: Uuid) {
        if let Some(t) = self.lock().trips.get_mut(&trip) {
            t.members.retain(|m| m.user_id != user);
        }
    }

    pub fn plan(&self, id: Uuid) -> Option<Plan> {
        self.lock().plans.get(&id).cloned()
    }
}

impl PlanStore for InMemoryPlanStore {
    fn read(&self, user: Uuid, plan_id: Uuid) -> Result<Value, SaveFailure> {
        let state = self.lock();
        let plan = state.plans.get(&plan_id).ok_or(SaveFailure::NotFound)?;
        let trip = state
            .trips
            .get(&plan.trip_id)
            .ok_or(SaveFailure::NotFound)?;
        if !trip.is_member(user) {
            return Err(SaveFailure::NotFound);
        }
        // A draft is the creator's own until it is saved.
        if plan.status == PlanStatus::Draft && plan.created_by != user {
            return Err(SaveFailure::NotOwner);
        }
        serde_json::to_value(PlanView::new(plan, trip))
            .map_err(|e| SaveFailure::Internal(e.to_string()))
    }

    fn save(&self, cmd: SaveCommand) -> Result<SaveOutcome, SaveFailure> {
        let mut state = self.lock();
        let now = (self.clock)();

        // 1. The plan exists and the caller is in its trip; otherwise it does not exist for them.
        //    Checked before the key, so a member who left the trip cannot replay a stored plan.
        let plan = state
            .plans
            .get(&cmd.plan_id)
            .cloned()
            .ok_or(SaveFailure::NotFound)?;
        let trip = state
            .trips
            .get(&plan.trip_id)
            .cloned()
            .ok_or(SaveFailure::NotFound)?;
        if !trip.is_member(cmd.user) {
            return Err(SaveFailure::NotFound);
        }

        // 2. Only the member who created the plan saves it.
        if plan.created_by != cmd.user {
            return Err(SaveFailure::NotOwner);
        }

        // 3. A key already used by this caller within 24 hours: replay or refuse.
        let key_id = (cmd.user, cmd.key.clone());
        if let Some(stored) = state.keys.get(&key_id) {
            if now - stored.at < Duration::hours(KEY_TTL_HOURS) {
                return if stored.fingerprint == cmd.fingerprint {
                    Ok(SaveOutcome::Replayed(stored.body.clone()))
                } else {
                    Err(SaveFailure::KeyReused)
                };
            }
            state.keys.remove(&key_id);
        }

        // 4. The caller reviewed the version that is stored now.
        if plan.version != cmd.expected_version {
            return Err(SaveFailure::VersionConflict {
                current: plan.version,
            });
        }

        // 5. Saved already at this version: safe to repeat, nothing changes.
        let body = if plan.status == PlanStatus::Saved {
            serde_json::to_value(PlanView::new(&plan, &trip))
        } else {
            // 6. The validator runs one last time, here in the backend.
            let violations = validate(&plan, &trip);
            if !violations.is_empty() {
                return Err(SaveFailure::Rejected(violations));
            }
            let mut saved = plan.clone();
            saved.status = PlanStatus::Saved;
            saved.saved_at = Some(now);
            saved.saved_by = Some(cmd.user);
            // Everything that can fail happens before the stored plan changes.
            let body = serde_json::to_value(PlanView::new(&saved, &trip))
                .map_err(|e| SaveFailure::Internal(e.to_string()))?;
            state.plans.insert(cmd.plan_id, saved);
            Ok(body)
        }
        .map_err(|e: serde_json::Error| SaveFailure::Internal(e.to_string()))?;

        // 7. Only a 200 is stored under the key, together with the save.
        state.keys.insert(
            key_id,
            StoredKey {
                fingerprint: cmd.fingerprint,
                body: body.clone(),
                at: now,
            },
        );
        Ok(SaveOutcome::Saved(body))
    }
}
