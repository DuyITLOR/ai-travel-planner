//! Backend validator, the last check before a plan reaches the group.
//! Money is computed here, in integer VND, never by the model.
//!
//! Enforced now: `over-member-cap`, `over-trip-budget`. The place rules
//! (opening hours, travel time, rain) need the shared place data and arrive in PA#3.

use serde::Serialize;
use uuid::Uuid;

use super::model::{Plan, Stop, Trip};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    pub rule: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    pub amount_vnd: i64,
    pub limit_vnd: i64,
}

/// 2^53 - 1: the largest amount the contract sends. Anything larger is reported as this,
/// which is over every budget, so such a plan is always rejected.
pub const MAX_VND: i64 = 9_007_199_254_740_991;

fn clamp(v: i64) -> i64 {
    v.clamp(0, MAX_VND)
}

/// Cost of one stop for one person.
pub fn stop_cost(stop: &Stop) -> i64 {
    clamp(stop.items.iter().fold(0i64, |acc, i| {
        acc.saturating_add(i.unit_price_vnd.saturating_mul(i.quantity))
    }))
}

/// Everyone in the trip follows the same schedule, so each person spends this much.
pub fn per_person_cost(plan: &Plan) -> i64 {
    clamp(
        plan.stops
            .iter()
            .fold(0i64, |acc, s| acc.saturating_add(stop_cost(s))),
    )
}

/// What the whole trip spends.
pub fn total_cost(plan: &Plan, trip: &Trip) -> i64 {
    clamp(per_person_cost(plan).saturating_mul(trip.members.len() as i64))
}

pub fn validate(plan: &Plan, trip: &Trip) -> Vec<Violation> {
    let mut out = Vec::new();
    let per_person = per_person_cost(plan);

    // Each person's spending stays within the lowest personal cap in the group.
    let lowest = trip
        .members
        .iter()
        .filter_map(|m| m.cap_vnd.map(|c| (c, m.user_id)))
        .min();
    if let Some((cap, user)) = lowest
        && per_person > cap
    {
        out.push(Violation {
            rule: "over-member-cap",
            message: format!(
                "Each person would spend {per_person} VND, over the lowest member cap of {cap} VND."
            ),
            user_id: Some(user),
            amount_vnd: per_person,
            limit_vnd: cap,
        });
    }

    // The total for everyone stays within the trip budget.
    let total = total_cost(plan, trip);
    if total > trip.budget_vnd {
        out.push(Violation {
            rule: "over-trip-budget",
            message: format!(
                "The plan costs {total} VND in total, over the trip budget of {} VND.",
                trip.budget_vnd
            ),
            user_id: None,
            amount_vnd: total,
            limit_vnd: trip.budget_vnd,
        });
    }
    out
}
