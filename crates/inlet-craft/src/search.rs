//! Budgeted fcmaes DE ask/tell; every returned point re-evaluated cold.
use crate::{diagnostic::*, evaluate::*, model::*};
use fcmaes_core::{De, DeParams, Fitness, Mode, ModeParams};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Variable {
    pub path: String,
    pub lower: f64,
    pub upper: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Constraints {
    #[serde(default)]
    pub max_pm10_relative_deviation: Option<f64>,
    #[serde(default)]
    pub max_distance_resolution_m: Option<f64>,
    #[serde(default)]
    pub max_total_flow_m3_s: Option<f64>,
    #[serde(default)]
    pub max_warning_count: Option<usize>,
    #[serde(default)]
    pub require_laminar: bool,
    #[serde(default)]
    pub distance_reference_speed_m_s: Option<f64>,
    /// Explicit allowance for transport correlations extrapolated above one.
    /// Absent: any such value rejects the candidate. Present: accepted only if,
    /// in every scenario, the share of the PM10 ratio resting on efficiencies
    /// above one (PmResult.nonphysical_excess) is at most this value. Negative
    /// efficiencies are always rejected. Nothing is clipped.
    #[serde(default)]
    pub max_nonphysical_pm10_excess: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Study {
    pub schema: String,
    pub case: Case,
    pub variables: Vec<Variable>,
    pub max_evaluations: usize,
    #[serde(default = "default_seed")]
    pub seed: u64,
    #[serde(default = "default_seconds")]
    pub wall_time_seconds: f64,
    #[serde(default)]
    pub multi_objective: bool,
    #[serde(default)]
    pub constraints: Constraints,
}
fn default_seed() -> u64 {
    42
}
fn default_seconds() -> f64 {
    60.
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub x: Vec<f64>,
    /// All minimized: max PM10 absolute relative bias, max 10–90% distance,
    /// total flow in L/min. Units retained in names below.
    pub objectives: [f64; 3],
    pub maximum_pm10_relative_deviation: f64,
    pub maximum_distance_resolution_m: f64,
    pub distance_resolution_objective_m: f64,
    pub distance_reference_speed_m_s: Option<f64>,
    pub total_flow_l_min: f64,
    pub warning_count: usize,
    /// Largest PM10 nonphysical_excess over scenarios (0 if none).
    pub nonphysical_pm10_excess: f64,
    pub feasible: bool,
    pub normalized_violation: f64,
    pub violations: Vec<String>,
    pub case: Case,
    pub cold_rechecked: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchResult {
    pub schema: String,
    pub engine: String,
    pub status: String,
    pub termination: String,
    pub seed: u64,
    pub evaluations: usize,
    pub failed_evaluations: usize,
    pub cold_recheck_evaluations: usize,
    pub elapsed_seconds: f64,
    pub best: Option<Candidate>,
    pub pareto: Vec<Candidate>,
    pub failed_rows: Vec<FailedRow>,
    pub failed_rows_omitted: usize,
    pub baseline: Option<Candidate>,
    pub claim: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FailedRow {
    pub evaluation: usize,
    pub x: Vec<f64>,
    pub error: Diagnostic,
}
impl Study {
    pub fn validate(&self) -> Result<()> {
        if self.schema != "inlet-craft.study/0.1" {
            return Err(Diagnostic::new(
                "E_SCHEMA",
                "schema",
                "expected inlet-craft.study/0.1",
            ));
        }
        self.case.validate()?;
        if self.case.distribution.is_none() {
            return Err(Diagnostic::new(
                "E_DISTRIBUTION",
                "case.distribution",
                "PM10 design search requires an explicit size distribution",
            ));
        }
        if self.variables.is_empty() || self.variables.len() > 16 {
            return Err(Diagnostic::new(
                "E_VARIABLES",
                "variables",
                "expected 1..16 variables",
            ));
        }
        if self.max_evaluations < 4 || self.max_evaluations > 1_000_000 {
            return Err(Diagnostic::new(
                "E_BUDGET",
                "max_evaluations",
                "budget 4..1000000",
            ));
        }
        finite_positive(self.wall_time_seconds, "wall_time_seconds")?;
        if self.wall_time_seconds > 86400. {
            return Err(Diagnostic::new(
                "E_BUDGET",
                "wall_time_seconds",
                "maximum 86400s",
            ));
        }
        let base = serde_json::to_value(&self.case)
            .map_err(|e| Diagnostic::new("E_JSON", "case", e.to_string()))?;
        let mut seen = std::collections::BTreeSet::new();
        for (i, v) in self.variables.iter().enumerate() {
            let path = format!("variables[{i}]");
            nonnegative(v.lower, &path)?;
            finite_positive(v.upper, &path)?;
            if v.lower >= v.upper || !seen.insert(&v.path) {
                return Err(Diagnostic::new(
                    "E_BOUNDS",
                    &path,
                    "distinct variable paths and lower<upper required",
                ));
            }
            let parts: Vec<_> = v.path.split('/').collect();
            let allowed = v.path == "/flow/total_m3_s"
                || (parts.len() == 4
                    && parts[1] == "elements"
                    && parts[2].parse::<usize>().is_ok()
                    && [
                        "inner_diameter_m",
                        "length_m",
                        "radius_m",
                        "volume_m3",
                        "outlet_diameter_m",
                        "half_angle_deg",
                    ]
                    .contains(&parts[3]));
            if !allowed || base.pointer(&v.path).and_then(|n| n.as_f64()).is_none() {
                return Err(Diagnostic::new(
                    "E_VARIABLE",
                    &path,
                    "path must select an existing supported numeric design field by JSON pointer",
                ));
            }
        }
        for (name, x) in [
            (
                "max_pm10_relative_deviation",
                self.constraints.max_pm10_relative_deviation,
            ),
            (
                "max_distance_resolution_m",
                self.constraints.max_distance_resolution_m,
            ),
            ("max_total_flow_m3_s", self.constraints.max_total_flow_m3_s),
            (
                "distance_reference_speed_m_s",
                self.constraints.distance_reference_speed_m_s,
            ),
            (
                "max_nonphysical_pm10_excess",
                self.constraints.max_nonphysical_pm10_excess,
            ),
        ] {
            if let Some(x) = x {
                nonnegative(x, &format!("constraints.{name}"))?;
            }
        }
        Ok(())
    }
    pub fn candidate_case(&self, x: &[f64]) -> Result<Case> {
        if x.len() != self.variables.len() {
            return Err(Diagnostic::new(
                "E_DIMENSION",
                "x",
                "wrong decision dimension",
            ));
        }
        let mut case = serde_json::to_value(&self.case)
            .map_err(|e| Diagnostic::new("E_JSON", "case", e.to_string()))?;
        for (v, x) in self.variables.iter().zip(x) {
            if !x.is_finite() || *x < v.lower || *x > v.upper {
                return Err(Diagnostic::new(
                    "E_BOUNDS",
                    &v.path,
                    "decision outside bounds",
                ));
            }
            let dest = case
                .pointer_mut(&v.path)
                .ok_or_else(|| Diagnostic::new("E_VARIABLE", &v.path, "missing path"))?;
            *dest = serde_json::json!(x);
        }
        // One inlet diameter variable changes all connected equal-diameter
        // sections until the first contraction. Explicit tube variables override.
        if let Some((_, x)) = self
            .variables
            .iter()
            .zip(x)
            .find(|(v, _)| v.path == "/elements/0/inner_diameter_m")
        {
            let old = match self.case.elements[0] {
                Element::Probe {
                    inner_diameter_m, ..
                } => inner_diameter_m,
                _ => 0.,
            };
            for i in 1..self.case.elements.len() {
                if matches!(self.case.elements[i], Element::Contraction { .. }) {
                    break;
                }
                let ptr = format!("/elements/{i}/inner_diameter_m");
                if self.variables.iter().any(|v| v.path == ptr) {
                    continue;
                }
                if let Some(n) = case.pointer_mut(&ptr)
                    && n.as_f64().is_some_and(|d| (d - old).abs() < 1e-12)
                {
                    *n = serde_json::json!(x);
                }
            }
            if let Some(outer) = case.pointer_mut("/elements/0/outer_diameter_m")
                && let Some(d) = outer.as_f64()
            {
                *outer = serde_json::json!(d / old * x);
            }
        }
        for (v, x) in self.variables.iter().zip(x) {
            if let Some(prefix) = v.path.strip_suffix("/outlet_diameter_m") {
                let i = prefix
                    .rsplit('/')
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(usize::MAX);
                for (j, element) in self
                    .case
                    .elements
                    .iter()
                    .enumerate()
                    .skip(i.saturating_add(1))
                {
                    if matches!(element, Element::Contraction { .. }) {
                        break;
                    }
                    let ptr = format!("/elements/{j}/inner_diameter_m");
                    if let Some(n) = case.pointer_mut(&ptr)
                        && n.is_number()
                    {
                        *n = serde_json::json!(x);
                    }
                }
            }
        }
        let c: Case = serde_json::from_value(case)
            .map_err(|e| Diagnostic::new("E_CASE", "case", e.to_string()))?;
        c.validate()?;
        Ok(c)
    }
    pub fn assess(&self, x: &[f64], case: Case) -> Result<Candidate> {
        let r = evaluate(&case)?;
        let mut bias = 0_f64;
        let mut nonphysical_excess = 0_f64;
        for s in &r.scenarios {
            let pm10 = s.pm.iter().find(|p| p.nominal_cut_m == 10e-6);
            let p = pm10.and_then(|p| p.efficiency).ok_or_else(|| {
                Diagnostic::new("E_EMPTY_PM", "distribution", "no mass below PM10 cut")
            })?;
            bias = bias.max((p - 1.).abs());
            nonphysical_excess =
                nonphysical_excess.max(pm10.map_or(0., |p| p.nonphysical_excess.abs()));
        }
        let maximum_distance = r
            .scenarios
            .iter()
            .map(|s| s.distance_resolution_10_90_m)
            .fold(0., f64::max);
        let distance = if let Some(speed) = self.constraints.distance_reference_speed_m_s {
            r.scenarios[0].time_response.rise_10_90_s * speed
        } else {
            maximum_distance
        };
        let flow = case.flow.total_m3_s * 60000.;
        let mut violations = vec![];
        let mut violation = 0.;
        let mut bound = |name: &str, value: f64, max: Option<f64>| {
            if let Some(max) = max
                && value > max
            {
                violations.push(format!("{name}: {value:.6e} > {max:.6e}"));
                violation += (value - max) / max.max(1e-12);
            }
        };
        bound(
            "PM10 relative deviation",
            bias,
            self.constraints.max_pm10_relative_deviation,
        );
        bound(
            "distance resolution m",
            distance,
            self.constraints.max_distance_resolution_m,
        );
        bound(
            "total flow m3/s",
            case.flow.total_m3_s,
            self.constraints.max_total_flow_m3_s,
        );
        bound(
            "warning count",
            r.warning_count as f64,
            self.constraints.max_warning_count.map(|n| n as f64),
        );
        if self.constraints.require_laminar
            && r.scenarios
                .iter()
                .flat_map(|s| &s.sizes)
                .flat_map(|s| &s.elements)
                .any(|e| e.reynolds.is_some_and(|re| re >= 2000.))
        {
            violations.push("require_laminar: Re>=2000".into());
            violation += 1.;
        }
        let nonphysical: Vec<_> = r
            .warnings
            .iter()
            .filter(|w| w.diagnostic.code == "W_NONPHYSICAL")
            .collect();
        if nonphysical
            .iter()
            .any(|w| w.minimum_value.is_none_or(|v| v < 0.))
        {
            violations.push("negative extrapolated efficiency cannot be accepted by search".into());
            violation += 1e6;
        } else if !nonphysical.is_empty() {
            match self.constraints.max_nonphysical_pm10_excess {
                None => {
                    violations.push(
                        "nonphysical extrapolation cannot be accepted by search without constraints.max_nonphysical_pm10_excess"
                            .into(),
                    );
                    violation += 1e6;
                }
                Some(max) if nonphysical_excess > max => {
                    violations.push(format!(
                        "nonphysical PM10 excess: {nonphysical_excess:.6e} > {max:.6e}"
                    ));
                    violation += 1e6;
                }
                Some(_) => {}
            }
        }
        Ok(Candidate {
            x: x.to_vec(),
            objectives: [bias, distance, flow],
            maximum_pm10_relative_deviation: bias,
            maximum_distance_resolution_m: maximum_distance,
            distance_resolution_objective_m: distance,
            distance_reference_speed_m_s: self.constraints.distance_reference_speed_m_s,
            total_flow_l_min: flow,
            warning_count: r.warning_count,
            nonphysical_pm10_excess: nonphysical_excess,
            feasible: violations.is_empty(),
            normalized_violation: violation,
            violations,
            case,
            cold_rechecked: false,
        })
    }
}
fn dominates(a: &Candidate, b: &Candidate) -> bool {
    a.objectives.iter().zip(b.objectives).all(|(a, b)| *a <= b)
        && a.objectives.iter().zip(b.objectives).any(|(a, b)| *a < b)
}
fn preferred(a: &Candidate, b: &Candidate) -> bool {
    if a.feasible != b.feasible {
        return a.feasible;
    }
    if !a.feasible && a.normalized_violation != b.normalized_violation {
        return a.normalized_violation < b.normalized_violation;
    }
    a.objectives[0] < b.objectives[0]
}
pub fn optimize(study: &Study, cancel: &AtomicBool) -> Result<SearchResult> {
    study.validate()?;
    let start = Instant::now();
    let deadline = Duration::from_secs_f64(study.wall_time_seconds);
    let lower: Vec<_> = study.variables.iter().map(|v| v.lower).collect();
    let upper: Vec<_> = study.variables.iter().map(|v| v.upper).collect();
    let popsize = (8 * lower.len()).clamp(8, 64).min(study.max_evaluations) as i32;
    let params = DeParams {
        popsize,
        seed: study.seed,
        max_evaluations: study.max_evaluations as u64,
        ..Default::default()
    };
    let mut de = De::new(
        Fitness::bounded(lower.len(), 1, &lower, &upper),
        &[],
        &[],
        None,
        &params,
    );
    let mut mode = if study.multi_objective {
        Some(Mode::new(
            Fitness::bounded(lower.len(), 4, &lower, &upper),
            3,
            1,
            None,
            &ModeParams {
                popsize,
                seed: study.seed,
                ..Default::default()
            },
        ))
    } else {
        None
    };
    let baseline = if !cancel.load(Ordering::Relaxed) && start.elapsed() < deadline {
        study.assess(&[], study.case.clone()).ok()
    } else {
        None
    };
    let mut best = None;
    let mut archive: Vec<Candidate> = vec![];
    let mut failures = vec![];
    let mut failed = 0;
    let mut evaluations = 0;
    let mut termination = "evaluation_limit";
    'outer: while evaluations < study.max_evaluations {
        if cancel.load(Ordering::Relaxed) {
            termination = "cancelled";
            break;
        }
        if start.elapsed() >= deadline {
            termination = "wall_time";
            break;
        }
        let xs = if let Some(m) = &mut mode {
            m.ask()
        } else {
            de.ask()
        };
        let mut ys = vec![];
        let mut multi_ys = vec![];
        for x in &xs {
            if evaluations >= study.max_evaluations {
                break 'outer;
            }
            if cancel.load(Ordering::Relaxed) {
                termination = "cancelled";
                break 'outer;
            }
            if start.elapsed() >= deadline {
                termination = "wall_time";
                break 'outer;
            }
            evaluations += 1;
            match study
                .candidate_case(x)
                .and_then(|case| study.assess(x, case))
            {
                Ok(candidate) => {
                    let y = if candidate.feasible {
                        candidate.objectives[0]
                    } else {
                        1e12 + candidate.normalized_violation.min(1e100)
                    };
                    ys.push(y);
                    multi_ys.push(vec![
                        candidate.objectives[0],
                        candidate.objectives[1],
                        candidate.objectives[2],
                        candidate.normalized_violation,
                    ]);
                    if best.as_ref().is_none_or(|b| preferred(&candidate, b)) {
                        best = Some(candidate.clone());
                    }
                    if candidate.feasible
                        && !archive.iter().any(|p| {
                            dominates(p, &candidate) || p.objectives == candidate.objectives
                        })
                    {
                        archive.retain(|p| !dominates(&candidate, p));
                        archive.push(candidate);
                        if archive.len() > 256 {
                            archive.sort_by(|a, b| a.objectives[0].total_cmp(&b.objectives[0]));
                            archive.truncate(256);
                        }
                    }
                }
                Err(error) => {
                    ys.push(1e120);
                    multi_ys.push(vec![1e120, 1e120, 1e120, 1e120]);
                    failed += 1;
                    if failures.len() < 256 {
                        failures.push(FailedRow {
                            evaluation: evaluations,
                            x: x.clone(),
                            error,
                        });
                    }
                }
            }
        }
        if let Some(m) = &mut mode {
            m.tell(&multi_ys);
        } else {
            de.tell(&ys);
        }
    }
    // A failed recheck is evidence, not a reason to discard the whole search:
    // the candidate is kept, marked infeasible and unrechecked, with the error.
    let recheck = |c: &Candidate| -> Candidate {
        match study
            .candidate_case(&c.x)
            .and_then(|case| study.assess(&c.x, case))
        {
            Ok(mut cold) => {
                cold.cold_rechecked = true;
                cold
            }
            Err(e) => {
                let mut failed = c.clone();
                failed.feasible = false;
                failed.cold_rechecked = false;
                failed.violations.push(format!("cold recheck failed: {e}"));
                failed.normalized_violation += 1e6;
                failed
            }
        }
    };
    let mut rechecks = 0;
    if let Some(b) = best.take() {
        if !cancel.load(Ordering::Relaxed) && start.elapsed() < deadline {
            best = Some(recheck(&b));
            rechecks += 1;
        } else {
            best = Some(b);
        }
    }
    for p in &mut archive {
        if cancel.load(Ordering::Relaxed) || start.elapsed() >= deadline {
            break;
        }
        *p = recheck(p);
        rechecks += 1;
    }
    archive.retain(|p| p.feasible && p.cold_rechecked);
    // Acceptance needs a completed cold recheck; an unrechecked best (deadline
    // or cancellation) is returned for inspection with cold_rechecked=false.
    let status = if best
        .as_ref()
        .is_some_and(|b| b.feasible && b.cold_rechecked)
    {
        "feasible_design_found"
    } else if termination == "cancelled" {
        "cancelled_without_feasible_design"
    } else {
        "no_feasible_design_found"
    };
    Ok(SearchResult{schema:"inlet-craft.search/0.1".into(),engine:if study.multi_objective{"fcmaes-core MODE ask/tell (3 objectives, 1 aggregate constraint)"}else{"fcmaes-core DE ask/tell"}.into(),status:status.into(),termination:termination.into(),seed:study.seed,evaluations,failed_evaluations:failed,cold_recheck_evaluations:rechecks,elapsed_seconds:start.elapsed().as_secs_f64(),best,pareto:archive,failed_rows_omitted:failed.saturating_sub(failures.len()),failed_rows:failures,baseline,claim:"Finite-budget search; no proof of infeasibility or optimality. Feasibility is conditional on the declared correlations/assumptions and warning allowance.".into()})
}
