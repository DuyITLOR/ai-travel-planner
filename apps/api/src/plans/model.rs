use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::validator::{per_person_cost, stop_cost, total_cost};

/// A trip: the group (one member for an individual trip) and its total budget.
#[derive(Debug, Clone)]
pub struct Trip {
    pub id: Uuid,
    /// Group total for everyone, in VND. For an individual trip: that person's amount.
    pub budget_vnd: i64,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone)]
pub struct Member {
    pub user_id: Uuid,
    /// Personal spending cap in VND; `None` = no personal cap.
    pub cap_vnd: Option<i64>,
}

impl Trip {
    pub fn is_member(&self, user: Uuid) -> bool {
        self.members.iter().any(|m| m.user_id == user)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    Saved,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub status: PlanStatus,
    /// Goes up by one on every edit of the draft. Saving does not change it.
    pub version: i64,
    pub created_by: Uuid,
    pub saved_at: Option<DateTime<Utc>>,
    pub saved_by: Option<Uuid>,
    pub stops: Vec<Stop>,
}

#[derive(Debug, Clone)]
pub struct Stop {
    pub place_id: String,
    /// `YYYY-MM-DD`
    pub day: String,
    /// `HH:MM`, 24-hour, Asia/Ho_Chi_Minh
    pub start: String,
    pub end: String,
    pub items: Vec<Item>,
}

/// A menu item or ticket chosen at a stop. `quantity` is per person.
#[derive(Debug, Clone)]
pub struct Item {
    pub item_id: String,
    pub name: String,
    pub unit_price_vnd: i64,
    pub quantity: i64,
}

// ---- Response shape (the `Plan` resource of docs/contract/save-plan.md) ----

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    id: Uuid,
    trip_id: Uuid,
    status: PlanStatus,
    version: i64,
    created_by: Uuid,
    saved_at: Option<String>,
    saved_by: Option<Uuid>,
    stops: Vec<StopView>,
    cost: CostView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StopView {
    place_id: String,
    day: String,
    start: String,
    end: String,
    items: Vec<ItemView>,
    cost_per_person_vnd: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemView {
    item_id: String,
    name: String,
    unit_price_vnd: i64,
    quantity: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CostView {
    per_person_vnd: i64,
    member_count: usize,
    total_vnd: i64,
    budget_vnd: i64,
}

impl PlanView {
    pub fn new(plan: &Plan, trip: &Trip) -> Self {
        let per_person = per_person_cost(plan);
        let member_count = trip.members.len();
        Self {
            id: plan.id,
            trip_id: plan.trip_id,
            status: plan.status,
            version: plan.version,
            created_by: plan.created_by,
            saved_at: plan
                .saved_at
                .map(|t| t.to_rfc3339_opts(SecondsFormat::Millis, true)),
            saved_by: plan.saved_by,
            stops: plan
                .stops
                .iter()
                .map(|s| StopView {
                    place_id: s.place_id.clone(),
                    day: s.day.clone(),
                    start: s.start.clone(),
                    end: s.end.clone(),
                    items: s
                        .items
                        .iter()
                        .map(|i| ItemView {
                            item_id: i.item_id.clone(),
                            name: i.name.clone(),
                            unit_price_vnd: i.unit_price_vnd,
                            quantity: i.quantity,
                        })
                        .collect(),
                    cost_per_person_vnd: stop_cost(s),
                })
                .collect(),
            cost: CostView {
                per_person_vnd: per_person,
                member_count,
                total_vnd: total_cost(plan, trip),
                budget_vnd: trip.budget_vnd,
            },
        }
    }
}
