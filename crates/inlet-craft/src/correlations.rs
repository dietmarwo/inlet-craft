//! Equations, provenance and conventions: docs/developer/formula-registry.md.
use std::f64::consts::PI;
pub fn aspiration_facing(st: f64, r: f64) -> f64 {
    let k = 2. + 0.617 / r;
    1. + (r - 1.) * k * st / (1. + k * st)
}
pub fn aspiration_oblique(st: f64, r: f64, angle_deg: f64) -> f64 {
    if st == 0. {
        return 1.;
    }
    let s = st * (0.022 * angle_deg).exp();
    let k = 2. + 0.617 / r;
    // Algebraic quotient avoids cancellation at small Stokes number.
    let quotient = k / 2.617 * (1. + 2.617 * s) / (1. + k * s);
    let z = 0.55 * s * (0.25 * s).exp();
    let capture = if z.is_infinite() { 1. } else { z / (1. + z) };
    1. + (r * angle_deg.to_radians().cos() - 1.) * quotient * capture
}
/// Hangal and Willeke (1990a), 61..90deg. The Stokes exponent is sqrt(U/U0) =
/// 1/sqrt(R): at 90deg aspiration must fall as wind outruns inlet velocity
/// (R up). The 0.1.0 candidate used sqrt(R), which reversed that trend.
pub fn aspiration_right_angle(st: f64, r: f64, angle_deg: f64) -> f64 {
    1. + (r * angle_deg.to_radians().cos() - 1.) * 3. * st.powf(1. / r.sqrt())
}
pub fn aspiration_calm(st_inlet: f64, settling_ratio: f64, vertical_angle_deg: f64) -> f64 {
    settling_ratio * vertical_angle_deg.to_radians().cos()
        + (-4. * st_inlet.powf(1. + settling_ratio.sqrt()) / (1. + 2. * st_inlet)).exp()
}
pub fn transmission_facing(st: f64, r: f64) -> f64 {
    if st == 0. {
        return 1.;
    }
    if r >= 1. {
        (1. + (r - 1.) / (1. + 2.66 * st.powf(-2. / 3.))) / (1. + (r - 1.) / (1. + 0.418 / st))
    } else {
        (-75. * (0.09 * (st * (1. / r - 1.)).powf(0.3)).powi(2)).exp()
    }
}
pub fn transmission_oblique(st: f64, r: f64, angle_deg: f64, upward: bool) -> f64 {
    if st == 0. {
        return 1.;
    }
    let theta = angle_deg.to_radians();
    let iv = if r <= 1. {
        0.09 * (st * theta.cos() * (1. / r - 1.)).powf(0.3)
    } else {
        0.
    };
    let alpha = (12. * ((1. - angle_deg / 90.) - (-angle_deg).exp())).to_radians();
    let phi = theta + if upward { alpha } else { -alpha };
    // W eq 17/18 as printed: Stk*sqrt(U0/U), not sqrt(Stk*U0/U) (0.1.0 error).
    let iw = st * r.sqrt() * phi.sin() * (phi / 2.).sin();
    (-75. * (iv + iw).powi(2)).exp()
}
pub fn diffusion_laminar(xi: f64) -> f64 {
    if xi == 0. {
        return 1.;
    }
    (-3.66 * xi - 0.2672 * xi / (xi + 0.10079 * xi.cbrt())).exp()
}
pub fn diffusion_turbulent(xi: f64, re: f64, sc: f64) -> f64 {
    (-xi * 0.0118 * re.powf(7. / 8.) * sc.cbrt()).exp()
}
pub fn settling_laminar(k: f64) -> f64 {
    if k <= 0. {
        return 1.;
    }
    if k >= 1. {
        return 0.;
    }
    let c = k.cbrt();
    let s = (1. - c * c).sqrt();
    1. - 2. / PI * (2. * k * s - c * s + c.asin())
}
pub fn settling_turbulent(z: f64) -> f64 {
    (-4. * z / PI).exp()
}
pub fn turbulent_inertia(st: f64, re: f64, u: f64, d: f64, l: f64, q: f64) -> f64 {
    let vt =
        (6e-4 * (0.0395 * st * re.powf(0.75)).powi(2) + 2e-8 * re) * u / (5.03 * re.powf(0.125));
    (-PI * d * l * vt / q).exp()
}
pub fn bend_laminar(st: f64, angle_rad: f64) -> f64 {
    if angle_rad == 0. || st == 0. {
        return 1.;
    }
    let z = (0.452 * st / 0.171 + 2.242) * (st / 0.171).ln();
    let logsum = if z > 40. { z } else { z.exp().ln_1p() };
    (-2. * angle_rad / PI * logsum).exp()
}
pub fn bend_turbulent(st: f64, angle_rad: f64) -> f64 {
    (-2.823 * st * angle_rad).exp()
}
#[allow(clippy::approx_constant)] // 3.14 is the fitted coefficient, not mathematical pi.
pub fn contraction(
    st_upstream_velocity_downstream_diameter: f64,
    area_ratio: f64,
    half_angle_deg: f64,
) -> f64 {
    let x = 2. * st_upstream_velocity_downstream_diameter * (1. - area_ratio);
    if x == 0. {
        return 1.;
    }
    1. / (1. + (x / (3.14 * (-0.0185 * half_angle_deg).exp())).powf(1.24))
}
