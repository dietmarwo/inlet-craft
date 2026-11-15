use inlet_craft::{correlations as c, model::*, primitives::*, response::*, search::*, *};
use std::sync::atomic::AtomicBool;
fn example() -> Case {
    parse(include_str!("../../../examples/bicycle.json")).unwrap()
}
fn close(a: f64, b: f64, atol: f64) {
    assert!(
        (a - b).abs() <= atol,
        "{a:0.15e} vs {b:0.15e}, tolerance {atol}"
    );
}
#[test]
fn references_are_reproduced_and_evidence_is_honest() {
    let v = verification::builtin_verify();
    for r in &v.rows {
        assert!(
            r.passed,
            "{} {:?} vs {}",
            r.fixture.id, r.actual, r.fixture.expected
        );
    }
    assert!(!v.experimental_validation);
    assert!(v.gate_g0.starts_with("incomplete"));
}
#[test]
fn failed_reference_row_is_retained() {
    let mut f = verification::builtin_verify().rows[0].fixture.clone();
    f.expected = 9.;
    let v = verification::verify(vec![f]);
    assert!(!v.all_passed);
    assert_eq!(v.rows.len(), 1);
    assert!(!v.rows[0].passed);
}
#[test]
fn invalid_fixture_tolerance_cannot_pass() {
    let mut f = verification::builtin_verify().rows[0].fixture.clone();
    f.relative_tolerance = -1.;
    assert!(!verification::verify(vec![f]).all_passed);
}
#[test]
fn particle_diameter_roundtrips_and_relaxation_is_aerodynamic() {
    let a = Air::default();
    for d in [1e-9, 1e-8, 1e-7, 1e-6, 1e-5, 1e-4] {
        for rho in [500., 1000., 2500., 10000.] {
            for chi in [1., 1.5, 3.] {
                let p = Particle {
                    physical_diameter_m: d,
                    density_kg_m3: rho,
                    shape_factor: chi,
                };
                let da = p.aerodynamic_diameter_m(&a);
                let q = Particle::from_aerodynamic(da, rho, chi, &a);
                close(q.physical_diameter_m / d, 1., 1e-12);
                let sphere = Particle {
                    physical_diameter_m: da,
                    density_kg_m3: 1000.,
                    shape_factor: 1.,
                };
                close(
                    p.relaxation_time_s(&a) / sphere.relaxation_time_s(&a),
                    1.,
                    1e-12,
                );
            }
        }
    }
}
#[test]
fn air_pressure_and_dimensional_relations() {
    let a = Air::default();
    let b = Air {
        pressure_pa: 2. * a.pressure_pa,
        ..a.clone()
    };
    close(b.mean_free_path_m() / a.mean_free_path_m(), 0.5, 1e-14);
    close(b.density_kg_m3() / a.density_kg_m3(), 2., 1e-14);
    let p = Particle {
        physical_diameter_m: 1e-6,
        density_kg_m3: 1000.,
        shape_factor: 1.,
    };
    close(p.settling_m_s(&a) / p.relaxation_time_s(&a), G, 1e-14);
}
#[test]
fn matched_speeds_and_small_particle_limits() {
    for st in [0., 1e-20, 0.01, 0.1, 1., 100.] {
        close(c::aspiration_facing(st, 1.), 1., 1e-14);
        close(c::transmission_facing(st, 1.), 1., 1e-14);
    }
    close(c::aspiration_facing(1e15, 2.), 2., 1e-13);
    let a = Air::default();
    let tiny = Particle {
        physical_diameter_m: 1e-12,
        density_kg_m3: 1000.,
        shape_factor: 1.,
    };
    let tau = tiny.relaxation_time_s(&a);
    close(c::aspiration_calm(tau / 0.004, tau * G, 90.), 1., 1e-6);
}
#[test]
fn transport_zero_length_and_angle_limits() {
    close(c::diffusion_laminar(0.), 1., 0.);
    close(c::settling_laminar(0.), 1., 0.);
    close(c::settling_laminar(1.), 0., 0.);
    close(c::bend_laminar(0.2, 0.), 1., 0.);
    close(c::bend_turbulent(0.2, 0.), 1., 0.);
    close(c::contraction(0.1, 1., 45.), 1., 0.);
}
#[test]
fn transport_monotonicity_and_bounds() {
    let mut prev = [1.; 3];
    for i in 0..1000 {
        let x = i as f64 / 1000.;
        let v = [
            c::diffusion_laminar(x),
            c::settling_laminar(x),
            c::bend_laminar(x, std::f64::consts::FRAC_PI_2),
        ];
        for j in 0..3 {
            assert!(v[j] >= -1e-14 && v[j] <= 1. + 1e-14);
            assert!(v[j] <= prev[j] + 1e-14);
            prev[j] = v[j];
        }
    }
}
#[test]
fn contraction_uses_factor_two_and_upstream_velocity() {
    let c = example();
    let p = &c.particles()[3];
    let tau = p.relaxation_time_s(&c.air);
    let upstream = 0.008;
    let downstream = 0.004;
    let q = 2. / 60000.;
    let expected = c::contraction(tau * velocity(q, upstream) / downstream, 0.25, 45.);
    let wrong = c::contraction(tau * velocity(q, downstream) / downstream, 0.25, 45.);
    assert!((expected - wrong).abs() > 0.01);
}
#[test]
fn angles_symmetry_and_tailwind_gap_rejection() {
    let mut a = example();
    a.distribution = None;
    a.platform.speed_m_s = vec![5.];
    a.platform.crosswind_m_s = vec![1.];
    let first = evaluate(&a).unwrap();
    a.platform.crosswind_m_s = vec![-1.];
    let second = evaluate(&a).unwrap();
    close(
        first.scenarios[0].sizes[3].overall_efficiency,
        second.scenarios[0].sizes[3].overall_efficiency,
        1e-12,
    );
    a.platform.crosswind_m_s = vec![0.];
    if let Element::Probe {
        angle_to_heading_deg,
        ..
    } = &mut a.elements[0]
    {
        *angle_to_heading_deg = 60.5;
    }
    assert_eq!(evaluate(&a).unwrap_err().code, "E_ANGLE_UNCOVERED");
    if let Element::Probe {
        angle_to_heading_deg,
        ..
    } = &mut a.elements[0]
    {
        *angle_to_heading_deg = 0.;
    }
    a.platform.headwind_m_s = vec![-10.];
    assert_eq!(evaluate(&a).unwrap_err().code, "E_ANGLE_UNCOVERED");
}
#[test]
fn calm_and_slow_modes_are_finite_and_reported() {
    let mut a = example();
    a.distribution = None;
    a.platform.speed_m_s = vec![0.];
    a.platform.crosswind_m_s = vec![0.];
    let r = evaluate(&a).unwrap();
    assert!(
        r.scenarios[0]
            .sizes
            .iter()
            .all(|s| s.overall_efficiency.is_finite())
    );
    assert_eq!(
        r.scenarios[0].sizes[0].elements[0].mechanisms[0].formula,
        "aspiration.grinshpun_calm"
    );
    a.platform.speed_m_s = vec![0.1];
    let r = evaluate(&a).unwrap();
    assert_eq!(
        r.scenarios[0].sizes[0].elements[0].mechanisms[0].formula,
        "aspiration.grinshpun_slow"
    );
}
#[test]
fn out_of_range_values_are_reported_without_clamping() {
    let mut a = example();
    a.distribution = None;
    a.platform.speed_m_s = vec![7.];
    a.platform.crosswind_m_s = vec![0.];
    a.particles.aerodynamic_diameter_m = vec![100e-6];
    if let Element::Probe {
        angle_to_heading_deg,
        ..
    } = &mut a.elements[0]
    {
        *angle_to_heading_deg = 90.;
    }
    let r = evaluate(&a).unwrap();
    assert!(r.scenarios[0].sizes[0].aspiration_efficiency < 0.);
    assert!(
        r.warnings
            .iter()
            .any(|w| w.diagnostic.code == "W_NONPHYSICAL")
    );
    assert!(r.warnings.iter().any(|w| w.diagnostic.code == "W_RANGE"
        && w.minimum_value.is_some()
        && w.maximum_value.is_some()));
}
#[test]
fn unknown_nested_fields_and_trailing_data_rejected() {
    let mut v = serde_json::to_value(example()).unwrap();
    v["sensor"]["typo"] = serde_json::json!(2);
    let e = parse::<Case>(&v.to_string()).unwrap_err();
    assert_eq!(e.code, "E_JSON");
    assert!(e.path.contains("sensor"));
    let mut v = serde_json::to_value(example()).unwrap();
    v["elements"][0]["blunt"] = serde_json::json!(true);
    assert!(parse::<Case>(&v.to_string()).is_err());
    assert!(parse::<Case>(&(serde_json::to_string(&example()).unwrap() + " {} ")).is_err());
}
#[test]
fn unsupported_geometry_and_diameter_changes_rejected() {
    let mut c = example();
    if let Element::Probe {
        outer_diameter_m, ..
    } = &mut c.elements[0]
    {
        *outer_diameter_m = Some(0.005);
    }
    assert_eq!(c.validate().unwrap_err().code, "E_UNCOVERED");
    let mut c = example();
    if let Element::Tube {
        inner_diameter_m, ..
    } = &mut c.elements[1]
    {
        *inner_diameter_m = 0.008;
    }
    assert_eq!(c.validate().unwrap_err().code, "E_UNCOVERED");
}
#[test]
fn transition_flow_requires_opt_in() {
    let mut c = example();
    c.distribution = None;
    c.flow.total_m3_s = 3000. * std::f64::consts::PI * 0.004 * c.air.viscosity_pa_s()
        / (4. * c.air.density_kg_m3());
    assert_eq!(evaluate(&c).unwrap_err().code, "E_TRANSITION");
    c.options.extend_laminar_transition = true;
    assert!(
        evaluate(&c)
            .unwrap()
            .warnings
            .iter()
            .any(|d| d.diagnostic.code == "W_TRANSITION")
    );
}
#[test]
fn particle_count_through_split_is_conserved() {
    let mut c = example();
    c.distribution = None;
    c.elements.truncate(2);
    let before = evaluate(&c).unwrap();
    c.elements.push(Element::Split {
        sensor_m3_s: c.flow.sensor_m3_s,
    });
    let after = evaluate(&c).unwrap();
    for (a, b) in before.scenarios[0]
        .sizes
        .iter()
        .zip(&after.scenarios[0].sizes)
    {
        close(a.overall_efficiency, b.overall_efficiency, 1e-14);
        close(
            b.sensor_particle_flux_fraction + b.bypass_particle_flux_fraction,
            b.overall_efficiency,
            1e-12,
        );
    }
}
#[test]
fn sampling_weights_are_normalized() {
    let mut c = example();
    c.distribution = None;
    c.platform.speed_weights = Some(vec![1., 2., 0., 1.]);
    let r = evaluate(&c).unwrap();
    close(
        r.scenarios.iter().map(|s| s.conditions.weight).sum(),
        1.,
        1e-14,
    );
    c.platform.crosswind_weights = Some(vec![-1., 2.]);
    assert_eq!(c.validate().unwrap_err().code, "E_WEIGHTS");
}
#[test]
fn apparent_wind_vectors_and_stationary_case() {
    let (s, t) = apparent_wind(3., 4., 90.);
    close(s, 5., 1e-14);
    close(t, 53.13010235415598, 1e-12);
    let (s, t) = apparent_wind(0., 4., 30.);
    close(s, 4., 1e-14);
    close(t, 30., 1e-12);
}
#[test]
fn monodisperse_pm_matches_size_resolved() {
    let mut c = example();
    c.particles.aerodynamic_diameter_m = vec![2e-6];
    c.distribution = Some(Distribution {
        modes: vec![Mode {
            number_concentration_m3: 1e6,
            geometric_mean_diameter_m: 2e-6,
            geometric_std_dev: 1.,
        }],
        quadrature_bins: 32,
    });
    for s in evaluate(&c).unwrap().scenarios {
        for pm in s.pm {
            close(pm.efficiency.unwrap(), s.sizes[0].overall_efficiency, 1e-12);
        }
    }
}
#[test]
fn pm_quadrature_converges() {
    let mut c = example();
    c.platform.speed_m_s = vec![5.];
    c.platform.crosswind_m_s = vec![0.];
    c.distribution.as_mut().unwrap().quadrature_bins = 64;
    let a = evaluate(&c).unwrap();
    c.distribution.as_mut().unwrap().quadrature_bins = 128;
    let b = evaluate(&c).unwrap();
    for (p, q) in a.scenarios[0].pm.iter().zip(&b.scenarios[0].pm) {
        close(p.ideal_mass_kg_m3 / q.ideal_mass_kg_m3, 1., 1e-12);
        close(p.efficiency.unwrap(), q.efficiency.unwrap(), 0.002);
    }
}
#[test]
fn two_tanks_match_analytic_step_including_equal_poles() {
    for tau in [0.01, 0.5, 2.] {
        for i in 0..100 {
            let t = i as f64 * tau / 10.;
            close(
                mixed_step(t, &[tau, tau]),
                1. - (-t / tau).exp() * (1. + t / tau),
                1e-13,
            );
        }
    }
    close(
        mixed_step(1., &[1., 2.]),
        1. - (2. * (-0.5_f64).exp() - (-1_f64).exp()),
        1e-13,
    );
}
#[test]
fn general_tank_chain_conserves_probability() {
    for t in [0.1_f64, 1., 5., 20.] {
        let x = t;
        let expected = 1. - (-x).exp() * (1. + x + x * x / 2.);
        close(mixed_step(t, &[1., 1., 1.]), expected, 1e-12);
    }
}
#[test]
fn response_averaging_and_position_offsets() {
    let r = TimeResponse::new(2., vec![], 0., 4.);
    close(r.mean_age_s, 4., 1e-14);
    close(r.t10_s, 2.4, 1e-12);
    close(r.t90_s, 5.6, 1e-12);
    let c = example();
    let e = evaluate(&c).unwrap();
    for s in &e.scenarios {
        close(
            s.mean_position_offset_m,
            s.conditions.riding_speed_m_s * s.time_response.mean_age_s,
            1e-12,
        );
    }
}
#[test]
fn unsupported_stiff_time_chain_is_rejected_quickly() {
    let r = TimeResponse::new(0., vec![1e-12, 1., 1.], 0., 0.);
    assert_eq!(validate_response(&r).unwrap_err().code, "E_RESPONSE");
}
fn small_study() -> Study {
    let mut s: Study = parse(include_str!("../../../examples/bicycle-study.json")).unwrap();
    s.case.platform.speed_m_s = vec![5.];
    s.case.platform.crosswind_m_s = vec![0.];
    s.case.distribution.as_mut().unwrap().quadrature_bins = 16;
    s.variables.truncate(2);
    s.max_evaluations = 12;
    s.wall_time_seconds = 60.;
    s.constraints = Constraints::default();
    s
}
#[test]
fn search_is_deterministic_bounded_and_cold_rechecked() {
    let s = small_study();
    let a = optimize(&s, &AtomicBool::new(false)).unwrap();
    let b = optimize(&s, &AtomicBool::new(false)).unwrap();
    assert_eq!(a.evaluations, s.max_evaluations);
    assert_eq!(a.best.as_ref().unwrap().x, b.best.as_ref().unwrap().x);
    assert!(a.best.as_ref().unwrap().cold_rechecked);
    assert!(a.pareto.iter().all(|p| p.cold_rechecked));
}
#[test]
fn no_design_found_is_not_infeasibility_proof() {
    let mut s = small_study();
    s.constraints.max_distance_resolution_m = Some(0.);
    let r = optimize(&s, &AtomicBool::new(false)).unwrap();
    assert_eq!(r.status, "no_feasible_design_found");
    assert!(!r.best.unwrap().feasible);
    assert!(r.claim.contains("no proof"));
}
#[test]
fn mode_search_keeps_rechecked_nondominated_points() {
    let mut s = small_study();
    s.multi_objective = true;
    s.case.particles.aerodynamic_diameter_m = vec![5e-6];
    s.case.distribution.as_mut().unwrap().modes = vec![Mode {
        number_concentration_m3: 1e6,
        geometric_mean_diameter_m: 5e-6,
        geometric_std_dev: 1.,
    }];
    s.max_evaluations = 20;
    let r = optimize(&s, &AtomicBool::new(false)).unwrap();
    assert!(r.engine.contains("MODE"));
    assert!(
        !r.pareto.is_empty(),
        "MODE must produce a feasible coarse-particle archive"
    );
    assert!(r.pareto.iter().all(|c| c.cold_rechecked && c.feasible));
    for (a, i) in r.pareto.iter().zip(0..) {
        for (b, j) in r.pareto.iter().zip(0..) {
            if i != j {
                assert!(!a.objectives.iter().zip(b.objectives).all(|(a, b)| *a <= b));
            }
        }
    }
}
#[test]
fn cancellation_prevents_solver_and_rechecks() {
    let r = optimize(&small_study(), &AtomicBool::new(true)).unwrap();
    assert_eq!(r.evaluations, 0);
    assert_eq!(r.cold_recheck_evaluations, 0);
    assert!(r.baseline.is_none());
    assert_eq!(r.termination, "cancelled");
}
#[test]
fn schemas_generated_from_rust_are_strict() {
    let d = describe();
    assert_eq!(
        d["schemas"]["case"]["additionalProperties"],
        serde_json::json!(false)
    );
    assert!(
        d["formula_registry"]
            .as_str()
            .unwrap()
            .contains("G0 is incomplete")
    );
}

#[test]
fn tilted_probe_uses_vector_angle_and_boundary_roundoff() {
    let mut c = example();
    c.distribution = None;
    c.platform.speed_m_s = vec![5.];
    c.platform.crosswind_m_s = vec![0.];
    if let Element::Probe {
        angle_to_vertical_deg,
        ..
    } = &mut c.elements[0]
    {
        *angle_to_vertical_deg = 0.;
    }
    let r = evaluate(&c).unwrap();
    assert_eq!(
        r.scenarios[0].sizes[0].elements[0].mechanisms[0].formula,
        "aspiration.hangal_willeke"
    );
    if let Element::Probe {
        angle_to_vertical_deg,
        ..
    } = &mut c.elements[0]
    {
        *angle_to_vertical_deg = 30.;
    }
    assert!(evaluate(&c).is_ok());
}

#[test]
fn downward_facing_aperture_rejected_for_calm_and_slow() {
    let mut c = example();
    c.distribution = None;
    c.platform.crosswind_m_s = vec![0.];
    if let Element::Probe {
        angle_to_vertical_deg,
        upward_sampling,
        ..
    } = &mut c.elements[0]
    {
        *angle_to_vertical_deg = 0.;
        *upward_sampling = true;
    }
    for wind in [0., 0.1] {
        c.platform.speed_m_s = vec![wind];
        assert_eq!(evaluate(&c).unwrap_err().code, "E_UNCOVERED");
    }
}

#[test]
fn unequal_three_tank_quantiles_match_independent_scipy_expm() {
    // Independently computed absorbing CTMC exponential + Brent roots;
    // source/procedure recorded in independent-review.md, tolerance1e-10.
    let r = TimeResponse::new(0., vec![0.3, 0.7, 1.5], 0., 0.);
    close(r.t10_s, 0.7986830031577664, 1e-10);
    close(r.t90_s, 4.704048480781246, 1e-10);
}

#[test]
fn right_angle_aspiration_falls_as_wind_outruns_inlet() {
    // At 90deg, (R*cos-1) = -1, so only the Stokes exponent carries R.
    // Wind faster than inlet velocity (larger R = U0/U) must not improve
    // aspiration of inertial particles (Vincent et al.: A90 decreases in
    // St*R^0.5). The 0.1.0 exponent sqrt(R) reversed this trend.
    for st in [0.02, 0.05, 0.1, 0.2] {
        let a: Vec<_> = [0.5, 1., 2.]
            .iter()
            .map(|r| c::aspiration_right_angle(st, *r, 90.))
            .collect();
        assert!(a[0] > a[1] && a[1] > a[2], "St={st}: {a:?}");
    }
    close(
        c::aspiration_right_angle(0.1, 2., 90.),
        1. - 3. * 0.1_f64.powf(0.5_f64.sqrt()),
        1e-14,
    );
}

fn liu_case() -> Case {
    // Facing, sub-isokinetic (U0/U ~ 1.9): Liu transmission extrapolates
    // slightly above one for Stokes numbers below ~0.0039.
    let mut c = example();
    c.platform.speed_m_s = vec![5.];
    c.platform.crosswind_m_s = vec![0.];
    c.particles.aerodynamic_diameter_m = vec![3e-7, 1e-5];
    c
}

#[test]
fn nonphysical_excess_is_reported_without_clipping() {
    let r = evaluate(&liu_case()).unwrap();
    let fine = &r.scenarios[0].sizes[0];
    let liu = &fine.elements[0].mechanisms[1];
    assert_eq!(liu.formula, "transmission.liu");
    assert!(liu.efficiency > 1., "raw extrapolation is kept");
    assert!(fine.nonphysical_transport_excess > 0.);
    close(
        fine.overall_efficiency - fine.nonphysical_transport_excess,
        fine.aspiration_efficiency
            * fine
                .elements
                .iter()
                .map(|e| {
                    let m = if e.kind == "probe" {
                        &e.mechanisms[1..]
                    } else {
                        &e.mechanisms[..]
                    };
                    m.iter().map(|m| m.efficiency.min(1.)).product::<f64>()
                })
                .product::<f64>(),
        1e-15,
    );
    // The coarse size is inside the physical range: no excess at all.
    assert_eq!(r.scenarios[0].sizes[1].nonphysical_transport_excess, 0.);
    let w = r
        .warnings
        .iter()
        .find(|w| w.diagnostic.code == "W_NONPHYSICAL")
        .unwrap();
    assert_eq!(w.diagnostic.formula.as_deref(), Some("transmission.liu"));
    assert!(w.minimum_value.unwrap() > 1. && w.maximum_value.unwrap() < 1.01);
    let pm10 = &r.scenarios[0].pm[1];
    assert!(pm10.nonphysical_excess > 0. && pm10.nonphysical_excess < 1e-3);
}

#[test]
fn nonphysical_allowance_is_explicit_and_bounded() {
    let mut s = small_study();
    s.case = liu_case();
    let strict = s.assess(&[], s.case.clone()).unwrap();
    assert!(!strict.feasible);
    assert!(strict.violations[0].contains("max_nonphysical_pm10_excess"));
    assert!(strict.nonphysical_pm10_excess > 0.);
    s.constraints.max_nonphysical_pm10_excess = Some(strict.nonphysical_pm10_excess * 0.5);
    let tight = s.assess(&[], s.case.clone()).unwrap();
    assert!(!tight.feasible && tight.violations[0].contains("nonphysical PM10 excess"));
    s.constraints.max_nonphysical_pm10_excess = Some(1e-3);
    let allowed = s.assess(&[], s.case.clone()).unwrap();
    assert!(allowed.feasible, "{:?}", allowed.violations);
    assert_eq!(
        allowed.objectives, strict.objectives,
        "allowance never alters values"
    );
}

#[test]
fn negative_efficiency_is_rejected_even_with_allowance() {
    let mut s = small_study();
    s.case = liu_case();
    s.case.particles.aerodynamic_diameter_m = vec![1e-4];
    if let Element::Probe {
        angle_to_heading_deg,
        ..
    } = &mut s.case.elements[0]
    {
        *angle_to_heading_deg = 90.;
    }
    s.constraints.max_nonphysical_pm10_excess = Some(1.);
    let c = s.assess(&[], s.case.clone()).unwrap();
    assert!(!c.feasible);
    assert!(c.violations.iter().any(|v| v.contains("negative")));
}

#[test]
fn bend_radius_must_exceed_tube_radius() {
    let mut c = example();
    if let Element::Bend { radius_m, .. } = &mut c.elements[2] {
        *radius_m = 0.002;
    }
    let e = evaluate(&c).unwrap_err();
    assert_eq!(
        (e.code.as_str(), e.path.as_str()),
        ("E_UNCOVERED", "elements[2]")
    );
}

#[test]
fn oblique_wall_impaction_is_linear_in_stokes_as_printed() {
    // W eq 17/18: Iw = Stk*sqrt(U0/U)*sin(.)*sin(./2). For U0/U > 1 the
    // vena-contracta term Iv is zero, so sqrt(-ln T / 75) = Iw must double
    // when Stk doubles. The 0.1.0 form sqrt(Stk*U0/U) grew by sqrt(2).
    for upward in [false, true] {
        let iw = |st: f64| (-c::transmission_oblique(st, 2., 45., upward).ln() / 75.).sqrt();
        close(iw(0.2) / iw(0.1), 2., 1e-12);
    }
}
