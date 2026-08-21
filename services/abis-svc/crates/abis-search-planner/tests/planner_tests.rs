//! Tests for the search planner.

use eel_abis_search_planner::domain::{ShardSpec, StepKind};
use eel_abis_search_planner::error::PlannerError;
use eel_abis_search_planner::estimator::CostModel;
use eel_abis_search_planner::executor::execute_plan;
use eel_abis_search_planner::plan::QueryPlanner;
use uuid::Uuid;

fn shard(gallery_size: usize) -> ShardSpec {
    ShardSpec {
        shard_id: Uuid::now_v7(),
        gallery_size,
        available_steps: vec![
            StepKind::DemographicPrune,
            StepKind::FaceAnn,
            StepKind::FingerprintExact,
            StepKind::IrisExact,
        ],
    }
}

#[test]
fn plan_no_shards_errors() {
    assert!(matches!(
        QueryPlanner::default().plan(vec![], 0.9),
        Err(PlannerError::NoShards)
    ));
}

#[test]
fn plan_invalid_threshold_high() {
    assert!(matches!(
        QueryPlanner::default().plan(vec![shard(1000)], 1.5),
        Err(PlannerError::InvalidThreshold(_))
    ));
}

#[test]
fn plan_invalid_threshold_negative() {
    assert!(matches!(
        QueryPlanner::default().plan(vec![shard(1000)], -0.1),
        Err(PlannerError::InvalidThreshold(_))
    ));
}

#[test]
fn plan_steps_sorted_by_cost() {
    let plan = QueryPlanner::default()
        .plan(vec![shard(1_000_000)], 0.9)
        .unwrap();
    for w in plan.steps.windows(2) {
        assert!(w[0].estimated_cost_ms <= w[1].estimated_cost_ms + 1e-6);
    }
}

#[test]
fn plan_demographic_prune_is_first() {
    let plan = QueryPlanner::default()
        .plan(vec![shard(1_000_000)], 0.9)
        .unwrap();
    assert_eq!(plan.steps[0].kind, StepKind::DemographicPrune);
}

#[test]
fn plan_total_cost_is_sum_of_steps() {
    let plan = QueryPlanner::default()
        .plan(vec![shard(500_000)], 0.8)
        .unwrap();
    let expected: f64 = plan.steps.iter().map(|s| s.estimated_cost_ms).sum();
    assert!((plan.total_estimated_cost_ms - expected).abs() < 1e-6);
}

#[test]
fn plan_includes_all_available_steps() {
    let plan = QueryPlanner::default()
        .plan(vec![shard(100_000)], 0.9)
        .unwrap();
    assert_eq!(plan.steps.len(), 4);
}

#[test]
fn plan_excludes_unavailable_steps() {
    let s = ShardSpec {
        shard_id: Uuid::now_v7(),
        gallery_size: 100_000,
        available_steps: vec![StepKind::FaceAnn],
    };
    let plan = QueryPlanner::default().plan(vec![s], 0.9).unwrap();
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.steps[0].kind, StepKind::FaceAnn);
}

#[test]
fn executor_returns_result_per_shard() {
    let s1 = shard(100_000);
    let s2 = shard(200_000);
    let ids = vec![s1.shard_id, s2.shard_id];
    let plan = QueryPlanner::default().plan(vec![s1, s2], 0.9).unwrap();
    let scores: Vec<(Uuid, f64)> = ids.iter().map(|&id| (id, 0.5)).collect();
    assert_eq!(execute_plan(&plan, &scores).unwrap().len(), 2);
}

#[test]
fn executor_early_terminates() {
    let shards: Vec<ShardSpec> = (0..5).map(|_| shard(100_000)).collect();
    let ids: Vec<Uuid> = shards.iter().map(|s| s.shard_id).collect();
    let plan = QueryPlanner::default().plan(shards, 0.95).unwrap();
    let scores: Vec<(Uuid, f64)> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, if i == 0 { 0.97 } else { 0.5 }))
        .collect();
    let results = execute_plan(&plan, &scores).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn cost_model_prune_cheapest_vs_exact() {
    let model = CostModel::default();
    assert!(
        model.estimate_step_cost(&StepKind::DemographicPrune, 1_000_000)
            < model.estimate_step_cost(&StepKind::FingerprintExact, 1_000_000)
    );
}

#[test]
fn cost_model_scales_with_gallery() {
    let model = CostModel::default();
    assert!(
        model.estimate_step_cost(&StepKind::FaceAnn, 1_000_000)
            > model.estimate_step_cost(&StepKind::FaceAnn, 100_000)
    );
}

#[test]
fn plan_shards_preserved() {
    let s = shard(50_000);
    let id = s.shard_id;
    let plan = QueryPlanner::default().plan(vec![s], 0.9).unwrap();
    assert_eq!(plan.shards[0].shard_id, id);
}

#[test]
fn plan_threshold_stored() {
    let plan = QueryPlanner::default()
        .plan(vec![shard(10_000)], 0.85)
        .unwrap();
    assert!((plan.early_termination_threshold - 0.85).abs() < 1e-10);
}

#[test]
fn plan_no_steps_errors() {
    let s = ShardSpec {
        shard_id: Uuid::now_v7(),
        gallery_size: 1000,
        available_steps: vec![],
    };
    assert!(matches!(
        QueryPlanner::default().plan(vec![s], 0.9),
        Err(PlannerError::NoSteps)
    ));
}
