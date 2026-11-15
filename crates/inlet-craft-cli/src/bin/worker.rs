#![forbid(unsafe_code)]
use inlet_craft::{Study, optimize};
use optimization_job_api::{
    JobSpec, Result,
    result::{Evidence, Outcome},
    worker::{self, Prepared, TaskResult, Worker},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
struct InletWorker;
struct PreparedStudy {
    study: Study,
    cancel: AtomicBool,
}
impl Worker for InletWorker {
    fn describe(&self) -> Value {
        worker::description(
            "inlet-craft",
            env!("CARGO_PKG_VERSION"),
            json!([{"id":"inlet-craft.optimize/1","payload_schema":inlet_craft::describe()["schemas"]["study"]}]),
            json!([{"id":"inlet-system-design","description":"Budgeted inlet design, correlation-based feasibility; no experimental or optimality claim","payload_schemas":["inlet-craft.optimize/1"]}]),
            1,
        )
    }
    fn prepare(&self, spec: &mut JobSpec, _base: &Path) -> Result<Box<dyn Prepared>> {
        if spec.worker != "inlet-craft" || spec.payload_schema != "inlet-craft.optimize/1" {
            return Err("unsupported inlet-craft worker/profile".into());
        }
        if spec.resources.inner_threads != 1 {
            return Err("inner_threads must be1; use jobs for independent studies".into());
        }
        if !spec.inputs.is_empty() {
            return Err("all study data must be inline; inputs unsupported".into());
        }
        let mut study: Study = inlet_craft::parse(&spec.payload.to_string())?;
        study.wall_time_seconds = study.wall_time_seconds.min(spec.limits.wall_time_seconds);
        study.validate()?;
        spec.payload = serde_json::to_value(&study)?;
        Ok(Box::new(PreparedStudy {
            study,
            cancel: AtomicBool::new(false),
        }))
    }
}
impl Prepared for PreparedStudy {
    fn tasks(&self) -> usize {
        1
    }
    fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
    fn solve(&self, _task: usize, remaining_seconds: f64) -> Result<TaskResult> {
        let mut study = self.study.clone();
        study.wall_time_seconds = study.wall_time_seconds.min(remaining_seconds.max(1e-6));
        let r = optimize(&study, &self.cancel)?;
        let best = r.best.as_ref();
        let feasible = best.is_some_and(|p| p.feasible && p.cold_rechecked);
        let outcome = Outcome {
            termination: r.termination.clone(),
            claim: if feasible { "best_found" } else { "none" }.into(),
            feasible: Some(feasible),
            objective: best.map(|b| b.objectives[0]),
            x: best.map(|b| b.x.clone()),
            evidence: Some(Evidence {
                level: if best.is_some_and(|b| b.cold_rechecked) {
                    "rechecked"
                } else {
                    "claimed"
                }
                .into(),
                checks: if best.is_some() {
                    vec!["objective".into(), "bounds".into(), "constraints".into()]
                } else {
                    vec![]
                },
                ..Default::default()
            }),
            details: serde_json::to_value(&r)?,
            ..Default::default()
        };
        Ok(TaskResult {
            outcome,
            work: BTreeMap::from([
                ("evaluations".into(), r.evaluations as u64),
                ("cold_rechecks".into(), r.cold_recheck_evaluations as u64),
            ]),
        })
    }
}
fn main() {
    worker::run(InletWorker);
}
