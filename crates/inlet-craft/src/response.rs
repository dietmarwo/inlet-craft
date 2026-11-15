use crate::diagnostic::{Diagnostic, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimeResponse {
    pub plug_delay_s: f64,
    pub mixed_time_constants_s: Vec<f64>,
    pub sensor_time_constant_s: f64,
    pub averaging_interval_s: f64,
    pub mean_age_s: f64,
    pub age_std_s: f64,
    /// Engineering definition: distance traveled during 10–90% step rise.
    pub rise_10_90_s: f64,
    pub t10_s: f64,
    pub t90_s: f64,
}
impl TimeResponse {
    pub fn new(plug: f64, mixed: Vec<f64>, sensor: f64, interval: f64) -> Self {
        let mean = plug + mixed.iter().sum::<f64>() + sensor + interval / 2.;
        let variance =
            mixed.iter().map(|t| t * t).sum::<f64>() + sensor * sensor + interval * interval / 12.;
        let mut r = Self {
            plug_delay_s: plug,
            mixed_time_constants_s: mixed,
            sensor_time_constant_s: sensor,
            averaging_interval_s: interval,
            mean_age_s: mean,
            age_std_s: variance.sqrt(),
            rise_10_90_s: 0.,
            t10_s: 0.,
            t90_s: 0.,
        };
        r.t10_s = r.quantile(0.1);
        r.t90_s = r.quantile(0.9);
        r.rise_10_90_s = r.t90_s - r.t10_s;
        r
    }
    pub fn taus(&self) -> Vec<f64> {
        self.mixed_time_constants_s
            .iter()
            .copied()
            .chain([self.sensor_time_constant_s])
            .filter(|t| *t > 0.)
            .collect()
    }
    pub fn step(&self, time: f64) -> f64 {
        let t = time - self.plug_delay_s;
        if t < 0. {
            return 0.;
        }
        let taus = self.taus();
        let interval = self.averaging_interval_s;
        if interval == 0. {
            return mixed_step(t, &taus);
        }
        // Midpoint quadrature; exact for a chain without mixed tanks.
        if taus.is_empty() {
            return (t / interval).min(1.);
        }
        let upper = t.min(interval);
        let n = 128;
        let dt = upper / n as f64;
        (0..n)
            .map(|i| mixed_step(t - (i as f64 + 0.5) * dt, &taus))
            .sum::<f64>()
            * dt
            / interval
    }
    fn quantile(&self, p: f64) -> f64 {
        let taus = self.taus();
        if taus.is_empty() {
            return self.plug_delay_s + p * self.averaging_interval_s;
        }
        let mn = taus.iter().copied().fold(f64::INFINITY, f64::min);
        let mx = taus.iter().sum::<f64>();
        if taus.len() > 2 && (mx / mn > 200. || taus.len() > 12) {
            return f64::NAN;
        }
        let mut lo = self.plug_delay_s;
        let mut hi = self.mean_age_s + 40. * taus.iter().sum::<f64>() + self.averaging_interval_s;
        if taus.len() > 2 {
            let chunks = ((hi - self.plug_delay_s) / mn / 16.).ceil().max(1.);
            let averaging = if self.averaging_interval_s > 0. {
                128.
            } else {
                1.
            };
            if chunks * taus.len() as f64 * 160. * 96. * averaging > 50_000_000. {
                return f64::NAN;
            }
        }
        for _ in 0..48 {
            let m = (lo + hi) / 2.;
            let y = self.step(m);
            if !y.is_finite() {
                return f64::NAN;
            }
            if y < p {
                lo = m;
            } else {
                hi = m;
            }
        }
        (lo + hi) / 2.
    }
}
/// Stable convolution of first-order tanks by uniformization of the serial
/// phase-type CTMC. Splitting in time keeps Poisson weights representable;
/// this handles repeated and nearly-equal constants without subtractive poles.
pub fn mixed_step(t: f64, taus: &[f64]) -> f64 {
    if t < 0. {
        return 0.;
    }
    if taus.is_empty() {
        return 1.;
    }
    let active: Vec<_> = taus.iter().copied().filter(|x| *x > 0.).collect();
    if active.is_empty() {
        return 1.;
    }
    if active.len() == 1 {
        return -(-t / active[0]).exp_m1();
    }
    if active.len() == 2 {
        let (a, b) = (active[0], active[1]);
        if (a - b).abs() < 1e-8 * a.max(b) {
            let x = t / ((a + b) * 0.5);
            return 1. - (-x).exp() * (1. + x);
        }
        return 1. - (a * (-t / a).exp() - b * (-t / b).exp()) / (a - b);
    }
    // General small chain: fixed-step unconditionally stable implicit
    // trapezoid-free exponential propagation using uniformization.
    let lambda = active.iter().map(|x| 1. / x).fold(0., f64::max);
    let chunks = (lambda * t / 16.).ceil().max(1.) as usize;
    // Extreme stiffness: numerical CDF by deterministic convolution fallback
    // is rejected by public validation instead of allocating without a bound.
    if chunks.saturating_mul(active.len()) > 20000 {
        return f64::NAN;
    }
    let x = lambda * t / chunks as f64;
    let n = active.len();
    let mut state = vec![0.; n + 1];
    state[0] = 1.;
    for _ in 0..chunks {
        let mut z = state.clone();
        let mut weight = (-x).exp();
        let mut sum: Vec<_> = z.iter().map(|v| weight * v).collect();
        for k in 1..160 {
            for j in (1..=n).rev() {
                let inflow = z[j - 1] / active[j - 1] / lambda;
                let stay = if j == n {
                    z[j]
                } else {
                    z[j] * (1. - 1. / active[j] / lambda)
                };
                z[j] = stay + inflow;
            }
            z[0] *= 1. - 1. / active[0] / lambda;
            weight *= x / k as f64;
            for j in 0..=n {
                sum[j] += weight * z[j];
            }
            if k > 32 && weight < 1e-16 {
                break;
            }
        }
        state = sum;
    }
    state[n]
}
pub fn validate_response(r: &TimeResponse) -> Result<()> {
    if !r.t10_s.is_finite() || !r.t90_s.is_finite() {
        Err(Diagnostic::new(
            "E_RESPONSE",
            "elements",
            "time-constant stiffness exceeds supported response solver; reduce ratio or chain length",
        ))
    } else {
        Ok(())
    }
}
