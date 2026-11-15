//! SI primitives. Aerodynamic and physical diameter are different quantities.
use crate::diagnostic::{Diagnostic, Result, finite_positive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const G: f64 = 9.80665;
pub const KB: f64 = 1.380649e-23;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Air {
    pub temperature_k: f64,
    pub pressure_pa: f64,
}
impl Default for Air {
    fn default() -> Self {
        Self {
            temperature_k: 293.15,
            pressure_pa: 101325.,
        }
    }
}
impl Air {
    pub fn validate(&self) -> Result<()> {
        finite_positive(self.temperature_k, "air.temperature_k")?;
        finite_positive(self.pressure_pa, "air.pressure_pa")
    }
    pub fn viscosity_pa_s(&self) -> f64 {
        1.716e-5 * (self.temperature_k / 273.15).powf(1.5) * (273.15 + 111.)
            / (self.temperature_k + 111.)
    }
    pub fn density_kg_m3(&self) -> f64 {
        self.pressure_pa / (287.05 * self.temperature_k)
    }
    pub fn mean_free_path_m(&self) -> f64 {
        self.viscosity_pa_s() / self.pressure_pa
            * (std::f64::consts::PI * 287.05 * self.temperature_k / 2.).sqrt()
    }
    pub fn slip(&self, diameter_m: f64) -> f64 {
        let kn = 2. * self.mean_free_path_m() / diameter_m;
        1. + kn * (1.142 + 0.558 * (-0.999 / kn).exp())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Particle {
    pub physical_diameter_m: f64,
    pub density_kg_m3: f64,
    pub shape_factor: f64,
}
impl Particle {
    pub fn validate(&self) -> Result<()> {
        finite_positive(self.physical_diameter_m, "particle.physical_diameter_m")?;
        finite_positive(self.density_kg_m3, "particle.density_kg_m3")?;
        if !self.shape_factor.is_finite() || self.shape_factor < 1. {
            return Err(Diagnostic::new(
                "E_SHAPE",
                "particle.shape_factor",
                "dynamic shape factor must be >=1",
            ));
        }
        Ok(())
    }
    pub fn relaxation_time_s(&self, a: &Air) -> f64 {
        self.density_kg_m3 * self.physical_diameter_m.powi(2) * a.slip(self.physical_diameter_m)
            / (18. * a.viscosity_pa_s() * self.shape_factor)
    }
    pub fn settling_m_s(&self, a: &Air) -> f64 {
        G * self.relaxation_time_s(a)
    }
    pub fn diffusion_m2_s(&self, a: &Air) -> f64 {
        KB * a.temperature_k * a.slip(self.physical_diameter_m)
            / (3.
                * std::f64::consts::PI
                * a.viscosity_pa_s()
                * self.physical_diameter_m
                * self.shape_factor)
    }
    pub fn settling_re(&self, a: &Air) -> f64 {
        a.density_kg_m3() * self.settling_m_s(a) * self.physical_diameter_m / a.viscosity_pa_s()
    }
    pub fn aerodynamic_diameter_m(&self, a: &Air) -> f64 {
        invert_slip_size(
            self.physical_diameter_m.powi(2)
                * a.slip(self.physical_diameter_m)
                * self.density_kg_m3
                / 1000.
                / self.shape_factor,
            a,
        )
    }
    pub fn from_aerodynamic(da: f64, density: f64, shape: f64, a: &Air) -> Self {
        Self {
            physical_diameter_m: invert_slip_size(
                da * da * a.slip(da) * 1000. * shape / density,
                a,
            ),
            density_kg_m3: density,
            shape_factor: shape,
        }
    }
}
fn invert_slip_size(target: f64, a: &Air) -> f64 {
    let mut lo = 0.;
    let mut hi = target.sqrt() + 1.;
    for _ in 0..100 {
        let m = (lo + hi) * 0.5;
        if m * m * a.slip(m) > target {
            hi = m;
        } else {
            lo = m;
        }
    }
    (lo + hi) * 0.5
}
pub fn velocity(q: f64, d: f64) -> f64 {
    4. * q / (std::f64::consts::PI * d * d)
}
pub fn reynolds(a: &Air, q: f64, d: f64) -> f64 {
    a.density_kg_m3() * velocity(q, d) * d / a.viscosity_pa_s()
}
