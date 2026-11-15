use crate::{correlations as c, diagnostic::*, primitives::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub id: String,
    pub formula: String,
    pub inputs: Vec<f64>,
    pub expected: f64,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
    pub evidence_kind: String,
    pub source: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VerificationRow {
    pub fixture: Fixture,
    pub actual: Option<f64>,
    pub passed: bool,
    pub error: Option<Diagnostic>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub schema: String,
    pub all_passed: bool,
    pub rows: Vec<VerificationRow>,
    pub gate_g0: String,
    pub gate_g1: String,
    pub experimental_validation: bool,
}
pub fn fixture_value(f: &Fixture) -> Result<f64> {
    let v = &f.inputs;
    let get = |n| {
        v.get(n)
            .copied()
            .ok_or_else(|| Diagnostic::new("E_FIXTURE", &f.id, "missing numeric input"))
    };
    Ok(match f.formula.as_str() {
        "air.viscosity" => Air {
            temperature_k: get(0)?,
            pressure_pa: 101325.,
        }
        .viscosity_pa_s(),
        "air.mean_free_path" => Air {
            temperature_k: get(0)?,
            pressure_pa: get(1)?,
        }
        .mean_free_path_m(),
        "particle.slip" => Air::default().slip(get(0)?),
        "particle.settling" => Particle {
            physical_diameter_m: get(0)?,
            density_kg_m3: get(1)?,
            shape_factor: 1.,
        }
        .settling_m_s(&Air::default()),
        "particle.diffusion" => Particle {
            physical_diameter_m: get(0)?,
            density_kg_m3: 1000.,
            shape_factor: 1.,
        }
        .diffusion_m2_s(&Air::default()),
        "aspiration.belyaev_levin" => c::aspiration_facing(get(0)?, get(1)?),
        "aspiration.durham_lundgren" => c::aspiration_oblique(get(0)?, get(1)?, get(2)?),
        "aspiration.hangal_willeke" => c::aspiration_right_angle(get(0)?, get(1)?, get(2)?),
        "aspiration.grinshpun_calm" => c::aspiration_calm(get(0)?, get(1)?, get(2)?),
        "transmission.facing" => c::transmission_facing(get(0)?, get(1)?),
        "transmission.hangal_willeke_angle" => {
            c::transmission_oblique(get(0)?, get(1)?, get(2)?, get(3)? != 0.)
        }
        "tube.diffusion_holman" => c::diffusion_laminar(get(0)?),
        "tube.diffusion_friedlander" => c::diffusion_turbulent(get(0)?, get(1)?, get(2)?),
        "tube.settling_heyder" => c::settling_laminar(get(0)?),
        "tube.settling_schwendiman" => c::settling_turbulent(get(0)?),
        "tube.inertia_lee_gieseke" => {
            c::turbulent_inertia(get(0)?, get(1)?, get(2)?, get(3)?, get(4)?, get(5)?)
        }
        "bend.pui_laminar" => c::bend_laminar(get(0)?, get(1)?),
        "bend.pui_turbulent" => c::bend_turbulent(get(0)?, get(1)?),
        "contraction.muyshondt" => c::contraction(get(0)?, get(1)?, get(2)?),
        _ => {
            return Err(Diagnostic::new(
                "E_FIXTURE",
                &f.id,
                "unknown fixture formula",
            ));
        }
    })
}
pub fn verify(fixtures: Vec<Fixture>) -> Verification {
    let rows: Vec<_> = fixtures
        .into_iter()
        .map(|fixture| {
            let computed = fixture_value(&fixture);
            let (actual, error) = match computed {
                Ok(v) => (Some(v), None),
                Err(e) => (None, Some(e)),
            };
            let passed = fixture.expected.is_finite()
                && fixture.absolute_tolerance.is_finite()
                && fixture.absolute_tolerance >= 0.
                && fixture.relative_tolerance.is_finite()
                && fixture.relative_tolerance >= 0.
                && actual.is_some_and(|v| {
                    v.is_finite()
                        && (v - fixture.expected).abs()
                            <= fixture.absolute_tolerance
                                + fixture.relative_tolerance * fixture.expected.abs()
                });
            VerificationRow {
                fixture,
                actual,
                passed,
                error,
            }
        })
        .collect();
    Verification {
        schema: "inlet-craft.verification/0.1".into(),
        all_passed: !rows.is_empty() && rows.iter().all(|r| r.passed),
        rows,
        gate_g0: "incomplete_original_source_and_published_point_coverage".into(),
        gate_g1: "incomplete_independent_oracle_coverage".into(),
        experimental_validation: false,
    }
}
pub fn builtin_verify() -> Verification {
    verify(
        serde_json::from_str(include_str!("../../../fixtures/reference.json"))
            .expect("embedded fixtures validated by tests"),
    )
}
