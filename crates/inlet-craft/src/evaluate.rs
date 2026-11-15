use crate::{correlations as c, diagnostic::*, model::*, primitives::*, response::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, f64::consts::PI};
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mechanism {
    pub formula: String,
    pub efficiency: f64,
    pub validity: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ElementResult {
    pub index: usize,
    pub kind: String,
    pub flow_m3_s: f64,
    pub reynolds: Option<f64>,
    pub efficiency: f64,
    pub mechanisms: Vec<Mechanism>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SizeResult {
    pub slip_correction: f64,
    pub relaxation_time_s: f64,
    pub settling_velocity_m_s: f64,
    pub diffusion_coefficient_m2_s: f64,
    pub settling_re: f64,
    pub aerodynamic_diameter_m: f64,
    pub physical_diameter_m: f64,
    pub aspiration_efficiency: f64,
    pub transport_efficiency: f64,
    pub overall_efficiency: f64,
    /// overall_efficiency minus the same product with every transport
    /// mechanism above one replaced by one. Zero unless an extrapolated
    /// correlation exceeds its physical bound. A reported sensitivity; the
    /// efficiencies themselves are never clipped.
    pub nonphysical_transport_excess: f64,
    pub sensor_particle_flux_fraction: f64,
    pub bypass_particle_flux_fraction: f64,
    pub elements: Vec<ElementResult>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PmResult {
    pub nominal_cut_m: f64,
    pub ideal_mass_kg_m3: f64,
    pub sampled_mass_kg_m3: f64,
    pub efficiency: Option<f64>,
    /// Mass-weighted nonphysical_transport_excess divided by ideal mass: the
    /// share of `efficiency` that rests on transport efficiencies above one.
    pub nonphysical_excess: f64,
    pub quadrature_bins: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScenarioResult {
    pub conditions: Scenario,
    pub sizes: Vec<SizeResult>,
    pub pm: Vec<PmResult>,
    pub time_response: TimeResponse,
    pub mean_position_offset_m: f64,
    pub distance_resolution_10_90_m: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WarningSummary {
    pub diagnostic: Diagnostic,
    pub occurrences: usize,
    pub minimum_value: Option<f64>,
    pub maximum_value: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SizeStatistics {
    pub aerodynamic_diameter_m: f64,
    pub weighted_mean: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub weighted_std: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub schema: String,
    pub verification_status: String,
    pub assumptions: Vec<String>,
    pub assumed_distribution: Option<Distribution>,
    pub scenarios: Vec<ScenarioResult>,
    pub size_statistics: Vec<SizeStatistics>,
    pub warnings: Vec<WarningSummary>,
    pub warning_count: usize,
    pub warning_event_count: usize,
}
#[derive(Default)]
struct Collector(BTreeMap<String, WarningSummary>);
impl Collector {
    fn add(&mut self, warnings: Vec<Diagnostic>) {
        for d in warnings {
            let key = format!("{}|{}|{:?}|{:?}", d.code, d.path, d.formula, d.quantity);
            let e = self.0.entry(key).or_insert(WarningSummary {
                diagnostic: d.clone(),
                occurrences: 0,
                minimum_value: d.value,
                maximum_value: d.value,
            });
            e.occurrences += 1;
            if let Some(v) = d.value {
                e.minimum_value = Some(e.minimum_value.unwrap_or(v).min(v));
                e.maximum_value = Some(e.maximum_value.unwrap_or(v).max(v));
            }
        }
    }
}
fn reg(re: f64, options: &Options, path: &str, w: &mut Vec<Diagnostic>) -> Result<bool> {
    if (2000. ..=4000.).contains(&re) {
        if !options.extend_laminar_transition {
            return Err(Diagnostic::new(
                "E_TRANSITION",
                path,
                format!(
                    "Re={re:.6}; no correlation for 2000..4000; opt in with options.extend_laminar_transition"
                ),
            ));
        }
        let mut d = Diagnostic::new(
            "W_TRANSITION",
            path,
            "laminar equations explicitly extended to transition flow",
        );
        d.value = Some(re);
        d.quantity = Some("Re".into());
        d.allowed = Some([0., 2000.]);
        w.push(d);
        Ok(false)
    } else {
        Ok(re > 4000.)
    }
}
fn push_mech(
    out: &mut Vec<Mechanism>,
    formula: &str,
    eta: f64,
    transport: bool,
    path: &str,
    w: &mut Vec<Diagnostic>,
    start: usize,
) -> Result<()> {
    let before = w.len();
    efficiency(eta, transport, path, w)?;
    for d in &mut w[before..] {
        d.formula = Some(formula.into());
    }
    out.push(Mechanism {
        formula: formula.into(),
        efficiency: eta,
        validity: if w[start..]
            .iter()
            .any(|d| ["W_RANGE", "W_NONPHYSICAL", "W_CALM_TRANSMISSION"].contains(&d.code.as_str()))
        {
            "warning"
        } else {
            "within_declared_range"
        }
        .into(),
    });
    Ok(())
}
#[allow(clippy::too_many_arguments)] // Explicit source equation inputs and diagnostic context.
fn probe(
    p: &Particle,
    a: &Air,
    u: f64,
    u0: f64,
    d: f64,
    theta: f64,
    phi: f64,
    upward: bool,
    opt: &Options,
    path: &str,
    w: &mut Vec<Diagnostic>,
) -> Result<(f64, f64, Vec<Mechanism>)> {
    let tau = p.relaxation_time_s(a);
    let v = p.settling_m_s(a);
    let st = tau * u0 / d;
    let sti = tau * u / d;
    let ratio = u0 / u;
    let calm = opt.aspiration_mode == AspirationMode::Calm || u0 == 0.;
    let slow = !calm
        && (opt.aspiration_mode == AspirationMode::Slow
            || (opt.aspiration_mode == AspirationMode::Auto && u0 <= 1.5));
    if (calm || slow) && upward && phi < 90. {
        return Err(Diagnostic::new(
            "E_UNCOVERED",
            path,
            "downward-facing calm/slow aperture is outside the chosen Grinshpun geometry",
        ));
    }
    if opt.aspiration_mode == AspirationMode::Calm && u0 > 0. {
        return Err(Diagnostic::new(
            "E_REGIME",
            path,
            "calm mode requires zero apparent wind",
        ));
    }
    let mut mechanisms = vec![];
    let start = w.len();
    let mut moving = 1.;
    let mut name = "aspiration.grinshpun_calm";
    if !calm {
        if theta == 0. {
            name = "aspiration.belyaev_levin";
            range(w, path, name, "Stk_freestream", st, 0.05, 2.03);
            range(w, path, name, "U0/U", ratio, 0.17, 5.6);
            moving = c::aspiration_facing(st, ratio);
        } else if theta <= 60. + 1e-10 {
            name = "aspiration.durham_lundgren";
            range(w, path, name, "Stk_freestream", st, 0.02, 4.);
            range(w, path, name, "U0/U", ratio, 0.5, 2.);
            moving = c::aspiration_oblique(st, ratio, theta);
        } else if (61. - 1e-10..=90. + 1e-10).contains(&theta) {
            name = "aspiration.hangal_willeke";
            range(w, path, name, "Stk_freestream", st, 0.02, 0.2);
            range(w, path, name, "U0/U", ratio, 0.5, 2.);
            moving = c::aspiration_right_angle(st, ratio, theta);
        } else {
            return Err(Diagnostic::new(
                "E_ANGLE_UNCOVERED",
                path,
                format!(
                    "apparent yaw={theta:.5}deg; covered moving-air angles are 0..60 and 61..90deg"
                ),
            ));
        }
        // Boundary jumps are diagnostics with both evaluations, not smoothing.
        let jump0 = c::aspiration_oblique(st, ratio, 0.) - c::aspiration_facing(st, ratio);
        let jump60 =
            c::aspiration_right_angle(st, ratio, 61.) - c::aspiration_oblique(st, ratio, 60.);
        let mut diag = Diagnostic::new(
            "W_REGIME_BOUNDARY",
            path,
            format!(
                "empirical endpoint jumps: 0deg {jump0:.6e}; 60 to 61deg {jump60:.6e}; no smoothing"
            ),
        );
        diag.formula = Some(name.into());
        w.push(diag);
    }
    let mut asp = moving;
    if calm || slow {
        let calm_eta = c::aspiration_calm(sti, v / u, phi);
        range(
            w,
            path,
            "aspiration.grinshpun_calm",
            "Stk_inlet",
            sti,
            0.001,
            100.,
        );
        range(
            w,
            path,
            "aspiration.grinshpun_calm",
            "Vts/U",
            v / u,
            0.001,
            1.,
        );
        if calm {
            asp = calm_eta;
            name = "aspiration.grinshpun_calm";
        } else {
            let z = v / u0;
            let delta = z * z; // Horizontal freestream is orthogonal to vertical settling.
            let f = (-z).exp();
            asp = moving * (1. + delta).sqrt() * f + calm_eta * (-(-z).exp_m1());
            name = "aspiration.grinshpun_slow";
            w.push(Diagnostic::new("W_SLOW_POLICY",path,"slow-air blend U0<=1.5m/s follows source regime division; extension below0.5m/s is explicit; underlying correlation validity retained"));
        }
    }
    push_mech(&mut mechanisms, name, asp, false, path, w, start)?;
    let start = w.len();
    let trans;
    if calm {
        trans = 1.;
        w.push(Diagnostic::new("W_CALM_TRANSMISSION",path,"no additional calm-air probe wall-impaction model; gravitational loss belongs to downstream tube; transmission assumed unity"));
        name = "transmission.calm_assumed_unity";
    } else if theta == 0. {
        name = if ratio >= 1. {
            "transmission.liu"
        } else {
            "transmission.hangal_willeke"
        };
        range(
            w,
            path,
            name,
            "Stk_freestream",
            st,
            if ratio >= 1. { 0.01 } else { 0.02 },
            if ratio >= 1. { 100. } else { 4. },
        );
        range(
            w,
            path,
            name,
            "U0/U",
            ratio,
            if ratio >= 1. { 1. } else { 0.25 },
            if ratio >= 1. { 10. } else { 1. },
        );
        trans = c::transmission_facing(st, ratio);
    } else {
        name = "transmission.hangal_willeke_angle";
        range(w, path, name, "Stk_freestream", st, 0.02, 4.);
        range(w, path, name, "U0/U", ratio, 0.25, 4.);
        trans = c::transmission_oblique(st, ratio, theta, upward);
    }
    push_mech(&mut mechanisms, name, trans, true, path, w, start)?;
    Ok((asp, trans, mechanisms))
}
#[allow(clippy::too_many_arguments)] // Explicit geometry, flow, and particle inputs.
fn tube_mechanisms(
    p: &Particle,
    a: &Air,
    q: f64,
    d: f64,
    length: f64,
    inclination: f64,
    opt: &Options,
    path: &str,
    w: &mut Vec<Diagnostic>,
) -> Result<(f64, Vec<Mechanism>)> {
    let re = reynolds(a, q, d);
    let turbulent = reg(re, opt, path, w)?;
    let u = velocity(q, d);
    let tau = p.relaxation_time_s(a);
    let v = p.settling_m_s(a);
    let diff = p.diffusion_m2_s(a);
    let xi = PI * diff * length / q;
    let mut out = vec![];
    let start = w.len();
    let eta = if turbulent {
        c::diffusion_turbulent(xi, re, a.viscosity_pa_s() / a.density_kg_m3() / diff)
    } else {
        c::diffusion_laminar(xi)
    };
    push_mech(
        &mut out,
        if turbulent {
            "tube.diffusion_friedlander"
        } else {
            "tube.diffusion_holman"
        },
        eta,
        true,
        path,
        w,
        start,
    )?;
    let start = w.len();
    let inc = inclination.to_radians();
    range(
        w,
        path,
        "tube.settling_heyder",
        "abs(Vts*sin(i)/U)",
        (v * inc.sin() / u).abs(),
        0.,
        0.1,
    );
    let z = length * v * inc.cos() / d / u;
    let eta = if turbulent {
        c::settling_turbulent(z)
    } else {
        c::settling_laminar(0.75 * z)
    };
    push_mech(
        &mut out,
        if turbulent {
            "tube.settling_schwendiman"
        } else {
            "tube.settling_heyder"
        },
        eta,
        true,
        path,
        w,
        start,
    )?;
    if turbulent {
        let start = w.len();
        range(w, path, "tube.inertia_lee_gieseke", "Re", re, 4000., 15600.);
        let eta = c::turbulent_inertia(tau * u / d, re, u, d, length, q);
        push_mech(
            &mut out,
            "tube.inertia_lee_gieseke",
            eta,
            true,
            path,
            w,
            start,
        )?;
    }
    Ok((out.iter().map(|m| m.efficiency).product(), out))
}
pub fn evaluate_particle(
    case: &Case,
    scenario: &Scenario,
    p: &Particle,
) -> Result<(SizeResult, Vec<Diagnostic>)> {
    let mut w = vec![];
    let a = &case.air;
    p.validate()?;
    if ![
        a.slip(p.physical_diameter_m),
        p.relaxation_time_s(a),
        p.settling_m_s(a),
        p.diffusion_m2_s(a),
        p.settling_re(a),
    ]
    .iter()
    .all(|v| v.is_finite() && *v > 0.)
    {
        return Err(Diagnostic::new(
            "E_NUMERIC",
            "particles",
            "particle/air primitives overflowed or underflowed",
        ));
    }
    range(
        &mut w,
        "particles",
        "particle.stokes",
        "settling_Re",
        p.settling_re(a),
        0.,
        0.1,
    );
    range(
        &mut w,
        "particles",
        "particle.allen_raabe",
        "Kn",
        2. * a.mean_free_path_m() / p.physical_diameter_m,
        0.03,
        7.2,
    );
    let mut d = 0.;
    let mut q = case.flow.total_m3_s;
    let mut aspiration = 1.;
    let mut transport = 1.;
    let mut capped_transport = 1.;
    let mut above_one = false;
    let mut elements = vec![];
    let mut split_penetration = None;
    for (i, e) in case.elements.iter().enumerate() {
        let path = format!("elements[{i}]");
        let start = w.len();
        let (kind, eff, mechanisms, re) = match e {
            Element::Probe {
                inner_diameter_m,
                outer_diameter_m,
                angle_to_heading_deg,
                angle_to_vertical_deg,
                upward_sampling,
            } => {
                d = *inner_diameter_m;
                if outer_diameter_m.is_none() {
                    w.push(Diagnostic::new(
                        "W_WALL_ASSUMED",
                        &path,
                        "thin-walled probe assumed; supply outer_diameter_m to verify ratio",
                    ));
                }
                let yaw = (scenario.yaw_from_heading_deg - angle_to_heading_deg).to_radians();
                let theta = (angle_to_vertical_deg.to_radians().sin() * yaw.cos())
                    .acos()
                    .to_degrees();
                let (asp, t, m) = probe(
                    p,
                    a,
                    velocity(q, d),
                    scenario.apparent_speed_m_s,
                    d,
                    theta,
                    *angle_to_vertical_deg,
                    *upward_sampling,
                    &case.options,
                    &path,
                    &mut w,
                )?;
                aspiration = asp;
                ("probe", t, m, Some(reynolds(a, q, d)))
            }
            Element::Tube {
                length_m,
                inclination_deg,
                ..
            } => {
                let (t, m) = tube_mechanisms(
                    p,
                    a,
                    q,
                    d,
                    *length_m,
                    *inclination_deg,
                    &case.options,
                    &path,
                    &mut w,
                )?;
                ("tube", t, m, Some(reynolds(a, q, d)))
            }
            Element::Bend {
                angle_deg,
                radius_m,
                ..
            } => {
                let re = reynolds(a, q, d);
                let turbulent = reg(re, &case.options, &path, &mut w)?;
                let st = p.relaxation_time_s(a) * velocity(q, d) / d;
                let name = if turbulent {
                    "bend.pui_turbulent"
                } else {
                    "bend.pui_laminar"
                };
                range(
                    &mut w,
                    &path,
                    name,
                    "curvature_radius/tube_radius",
                    2. * radius_m / d,
                    5.,
                    30.,
                );
                if !turbulent {
                    range(&mut w, &path, name, "fit_Re", re, 1000., 1000.);
                    range(&mut w, &path, name, "fit_tube_diameter_m", d, 0.004, 0.0085);
                }
                let t = if turbulent {
                    c::bend_turbulent(st, angle_deg.to_radians())
                } else {
                    c::bend_laminar(st, angle_deg.to_radians())
                };
                let mut m = vec![];
                push_mech(&mut m, name, t, true, &path, &mut w, start)?;
                // Arc length adds diffusion; settling geometry is not known from
                // radius/angle alone, so do not invent an inclination.
                let l = radius_m * angle_deg.to_radians();
                let xi = PI * p.diffusion_m2_s(a) * l / q;
                let dt = if turbulent {
                    c::diffusion_turbulent(
                        xi,
                        re,
                        a.viscosity_pa_s() / a.density_kg_m3() / p.diffusion_m2_s(a),
                    )
                } else {
                    c::diffusion_laminar(xi)
                };
                let ms = w.len();
                push_mech(
                    &mut m,
                    if turbulent {
                        "tube.diffusion_friedlander"
                    } else {
                        "tube.diffusion_holman"
                    },
                    dt,
                    true,
                    &path,
                    &mut w,
                    ms,
                )?;
                w.push(Diagnostic::new("W_BEND_SETTLING",&path,"bend arc diffusion and impaction included; gravitational/inertial interaction and arc inclination unknown, settling omitted"));
                ("bend", t * dt, m, Some(re))
            }
            Element::Contraction {
                outlet_diameter_m,
                half_angle_deg,
                length_m,
            } => {
                let name = "contraction.muyshondt";
                let u = velocity(q, d);
                let st = p.relaxation_time_s(a) * u / outlet_diameter_m;
                let ar = (outlet_diameter_m / d).powi(2);
                let x = 2. * st * (1. - ar);
                range(&mut w, &path, name, "2St*(1-Ao/Ai)", x, 0.001, 100.);
                range(
                    &mut w,
                    &path,
                    name,
                    "half_angle_deg",
                    *half_angle_deg,
                    12.,
                    90.,
                );
                range(&mut w, &path, name, "Ao/Ai", ar, 0.062, 0.571);
                range(
                    &mut w,
                    &path,
                    name,
                    "downstream_Re",
                    reynolds(a, q, *outlet_diameter_m),
                    1120.,
                    58500.,
                );
                w.push(Diagnostic::new("W_SOURCE_CONFLICT",&path,"contraction uses Brockmann factor2; PLC paper Eq32 omits factor2. Original validation interval not fully resolved"));
                if *length_m > 0. {
                    w.push(Diagnostic::new("W_CONTRACTION_TRANSPORT",&path,"conical volume included in travel time; wall diffusion/settling inside taper unmodeled"));
                }
                let t = c::contraction(st, ar, *half_angle_deg);
                let mut m = vec![];
                push_mech(&mut m, name, t, true, &path, &mut w, start)?;
                d = *outlet_diameter_m;
                ("contraction", t, m, Some(reynolds(a, q, d)))
            }
            Element::Buffer { volume_m3 } => {
                if *volume_m3 > 0. {
                    w.push(Diagnostic::new(
                        "W_BUFFER_LOSS_UNMODELED",
                        &path,
                        "buffer models mixing time V/Q only; particle wall deposition is unmodeled",
                    ));
                }
                ("buffer", 1., vec![], None)
            }
            Element::Split { sensor_m3_s } => {
                split_penetration = Some(transport);
                q = *sensor_m3_s;
                w.push(Diagnostic::new("W_IDEAL_SPLIT",&path,"ideal well-mixed split: concentration unchanged; branch aspiration and impaction unmodeled"));
                ("split", 1., vec![], None)
            }
        };
        transport *= eff;
        // The probe's first mechanism is aspiration, which may exceed one;
        // its element efficiency is the transmission factor alone.
        let transport_factors = if kind == "probe" {
            &mechanisms[1..]
        } else {
            &mechanisms[..]
        };
        above_one |= transport_factors.iter().any(|m| m.efficiency > 1.);
        capped_transport *= transport_factors
            .iter()
            .map(|m| m.efficiency.min(1.))
            .product::<f64>();
        elements.push(ElementResult {
            index: i,
            kind: kind.into(),
            flow_m3_s: q,
            reynolds: re,
            efficiency: eff,
            mechanisms,
        });
    }
    let overall = aspiration * transport;
    efficiency(overall, false, "overall_efficiency", &mut w)?;
    let sensor_fraction = case.flow.sensor_m3_s / case.flow.total_m3_s;
    Ok((
        SizeResult {
            slip_correction: a.slip(p.physical_diameter_m),
            relaxation_time_s: p.relaxation_time_s(a),
            settling_velocity_m_s: p.settling_m_s(a),
            diffusion_coefficient_m2_s: p.diffusion_m2_s(a),
            settling_re: p.settling_re(a),
            aerodynamic_diameter_m: p.aerodynamic_diameter_m(a),
            physical_diameter_m: p.physical_diameter_m,
            aspiration_efficiency: aspiration,
            transport_efficiency: transport,
            overall_efficiency: overall,
            nonphysical_transport_excess: if above_one {
                overall - aspiration * capped_transport
            } else {
                0.
            },
            sensor_particle_flux_fraction: overall * sensor_fraction,
            bypass_particle_flux_fraction: aspiration
                * split_penetration.unwrap_or(transport)
                * (1. - sensor_fraction),
            elements,
        },
        w,
    ))
}
pub fn time_response(case: &Case) -> Result<TimeResponse> {
    let mut plug = 0.;
    let mut mixed = vec![];
    let mut q = case.flow.total_m3_s;
    let mut d = 0.;
    for e in &case.elements {
        match e {
            Element::Probe {
                inner_diameter_m, ..
            } => d = *inner_diameter_m,
            Element::Tube { length_m, .. } => plug += PI * d * d * length_m / (4. * q),
            Element::Bend {
                angle_deg,
                radius_m,
                ..
            } => plug += PI * d * d / 4. * radius_m * angle_deg.to_radians() / q,
            Element::Contraction {
                outlet_diameter_m,
                length_m,
                ..
            } => {
                plug += PI
                    * length_m
                    * (d * d + d * outlet_diameter_m + outlet_diameter_m * outlet_diameter_m)
                    / (12. * q);
                d = *outlet_diameter_m;
            }
            Element::Buffer { volume_m3 } => {
                if *volume_m3 > 0. {
                    mixed.push(volume_m3 / q);
                }
            }
            Element::Split { sensor_m3_s } => q = *sensor_m3_s,
        }
    }
    let r = TimeResponse::new(
        plug,
        mixed,
        case.sensor.response_time_s,
        case.sensor.sampling_interval_s,
    );
    validate_response(&r)?;
    Ok(r)
}
fn pm_grid(case: &Case, cut: f64) -> Vec<(Particle, f64)> {
    let Some(dist) = &case.distribution else {
        return vec![];
    };
    let a = &case.air;
    let props = &case.particles;
    let upper = Particle::from_aerodynamic(cut, props.density_kg_m3, props.shape_factor, a)
        .physical_diameter_m;
    let mut grid = vec![];
    for m in &dist.modes {
        let make = |d| Particle {
            physical_diameter_m: d,
            density_kg_m3: props.density_kg_m3,
            shape_factor: props.shape_factor,
        };
        let mu = m.geometric_mean_diameter_m.ln();
        let sigma = m.geometric_std_dev.ln();
        if sigma == 0. {
            if m.geometric_mean_diameter_m <= upper {
                let d = m.geometric_mean_diameter_m;
                grid.push((
                    make(d),
                    m.number_concentration_m3 * PI / 6. * props.density_kg_m3 * d.powi(3),
                ));
            }
            continue;
        }
        // Integrate the mass-shifted normal, whose mean is mu+3sigma².
        // Analytic total mass avoids quadrature denominator bias. Bounds ±9σ
        // capture mass to ~1e-19 relative in a full mode, cut applied in da.
        let mass_mu = mu + 3. * sigma * sigma;
        let hi = upper.ln().min(mass_mu + 9. * sigma);
        let lo = mass_mu - 9. * sigma;
        if hi <= lo {
            continue;
        }
        let delta = (hi - lo) / dist.quadrature_bins as f64;
        let total = m.number_concentration_m3 * PI / 6.
            * props.density_kg_m3
            * (3. * mu + 4.5 * sigma * sigma).exp();
        let mut row = vec![];
        for i in 0..dist.quadrature_bins {
            let logd = lo + (i as f64 + 0.5) * delta;
            let z = (logd - mass_mu) / sigma;
            let weight = total * (-0.5 * z * z).exp() / (sigma * (2. * PI).sqrt()) * delta;
            row.push((make(logd.exp()), weight));
        }
        // Normalize to exact truncated lognormal mass at the nominal cut.
        let cdf = |z: f64| 0.5 * libm::erfc(-z / 2f64.sqrt());
        let exact = total * (cdf((hi - mass_mu) / sigma) - cdf((lo - mass_mu) / sigma));
        let sum = row.iter().map(|(_, w)| w).sum::<f64>();
        if sum > 0. {
            for (p, w) in row {
                grid.push((p, w * exact / sum));
            }
        }
    }
    grid
}
pub fn evaluate(case: &Case) -> Result<Evaluation> {
    case.validate()?;
    let mut collector = Collector::default();
    let mut initial = vec![];
    range(
        &mut initial,
        "air.temperature_k",
        "air.sutherland",
        "temperature_k",
        case.air.temperature_k,
        200.,
        350.,
    );
    range(
        &mut initial,
        "air.pressure_pa",
        "air.sutherland",
        "pressure_pa",
        case.air.pressure_pa,
        20000.,
        120000.,
    );
    collector.add(initial);
    let time = time_response(case)?;
    let particles = case.particles();
    let mut scenarios = vec![];
    let pmgrids: [Vec<(Particle, f64)>; 2] = [pm_grid(case, 2.5e-6), pm_grid(case, 10e-6)];
    for conditions in case.scenarios()? {
        let mut sizes = vec![];
        for p in &particles {
            let (s, w) = evaluate_particle(case, &conditions, p)?;
            collector.add(w);
            sizes.push(s);
        }
        let mut pm = vec![];
        if let Some(dist) = &case.distribution {
            for (cut, grid) in [2.5e-6, 10e-6].iter().zip(&pmgrids) {
                let ideal = grid.iter().map(|(_, w)| w).sum::<f64>();
                let mut sampled = 0.;
                let mut excess = 0.;
                for (p, weight) in grid {
                    let (s, w) = evaluate_particle(case, &conditions, p)?;
                    collector.add(w);
                    sampled += weight * s.overall_efficiency;
                    excess += weight * s.nonphysical_transport_excess;
                }
                pm.push(PmResult {
                    nominal_cut_m: *cut,
                    ideal_mass_kg_m3: ideal,
                    sampled_mass_kg_m3: sampled,
                    efficiency: if ideal > 0. {
                        Some(sampled / ideal)
                    } else {
                        None
                    },
                    nonphysical_excess: if ideal > 0. { excess / ideal } else { 0. },
                    quadrature_bins: dist.quadrature_bins,
                });
            }
        }
        scenarios.push(ScenarioResult {
            mean_position_offset_m: conditions.riding_speed_m_s * time.mean_age_s,
            distance_resolution_10_90_m: conditions.riding_speed_m_s * time.rise_10_90_s,
            conditions,
            sizes,
            pm,
            time_response: time.clone(),
        });
    }
    let mut stats = vec![];
    for (i, particle) in particles.iter().enumerate() {
        let mean = scenarios
            .iter()
            .map(|s| s.conditions.weight * s.sizes[i].overall_efficiency)
            .sum::<f64>();
        stats.push(SizeStatistics {
            aerodynamic_diameter_m: particle.aerodynamic_diameter_m(&case.air),
            weighted_mean: mean,
            minimum: scenarios
                .iter()
                .map(|s| s.sizes[i].overall_efficiency)
                .fold(f64::INFINITY, f64::min),
            maximum: scenarios
                .iter()
                .map(|s| s.sizes[i].overall_efficiency)
                .fold(f64::NEG_INFINITY, f64::max),
            weighted_std: scenarios
                .iter()
                .map(|s| s.conditions.weight * (s.sizes[i].overall_efficiency - mean).powi(2))
                .sum::<f64>()
                .sqrt(),
        });
    }
    let warnings: Vec<_> = collector.0.into_values().collect();
    let count = warnings.len();
    let events = warnings.iter().map(|w| w.occurrences).sum();
    Ok(Evaluation{schema:"inlet-craft.result/0.1".into(),verification_status:"provisional_literature_equations; G0/G1 evidence incomplete; no on-bike validation".into(),assumptions:vec!["Independent mechanism efficiencies multiplied; no coupled particle trajectories".into(),"Conductive clean tubes, dry air; no electrostatics, humidity, sensor optics or platform wake".into(),"PM compares mass below an ideal aerodynamic hard cut; distribution is number-lognormal in physical diameter; sensor cut response not modeled".into(),"Position offset is mean sample age; resolution is distance traveled during 10–90% modeled step rise; sensor response_time_s is tau".into(),"Warnings are grouped by code/path/formula/quantity; minimum/maximum preserve evaluated range and occurrence count includes PM quadrature".into()],assumed_distribution:case.distribution.clone(),scenarios,size_statistics:stats,warnings,warning_count:count,warning_event_count:events})
}
