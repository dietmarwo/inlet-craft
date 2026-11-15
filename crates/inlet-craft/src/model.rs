use crate::{diagnostic::*, primitives::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
fn one() -> f64 {
    1.
}
fn density() -> f64 {
    1000.
}
fn zero_vec() -> Vec<f64> {
    vec![0.]
}
fn case_schema() -> String {
    "inlet-craft.case/0.1".into()
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub schema: String,
    #[serde(default)]
    pub air: Air,
    pub platform: Platform,
    pub particles: Particles,
    pub elements: Vec<Element>,
    pub flow: Flow,
    #[serde(default)]
    pub sensor: Sensor,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    pub distribution: Option<Distribution>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub speed_m_s: Vec<f64>,
    #[serde(default = "zero_vec")]
    pub crosswind_m_s: Vec<f64>,
    #[serde(default = "zero_vec")]
    pub headwind_m_s: Vec<f64>,
    #[serde(default)]
    pub speed_weights: Option<Vec<f64>>,
    #[serde(default)]
    pub crosswind_weights: Option<Vec<f64>>,
    #[serde(default)]
    pub headwind_weights: Option<Vec<f64>>,
    #[serde(default)]
    pub wind_samples: Option<Vec<WindSample>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WindSample {
    pub speed_m_s: f64,
    pub direction_from_heading_deg: f64,
    #[serde(default = "one")]
    pub weight: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Particles {
    #[serde(default)]
    pub aerodynamic_diameter_m: Vec<f64>,
    #[serde(default)]
    pub physical_diameter_m: Vec<f64>,
    #[serde(default = "density")]
    pub density_kg_m3: f64,
    #[serde(default = "one")]
    pub shape_factor: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Element {
    Probe {
        inner_diameter_m: f64,
        #[serde(default)]
        outer_diameter_m: Option<f64>,
        #[serde(default)]
        angle_to_heading_deg: f64,
        #[serde(default = "default_phi")]
        angle_to_vertical_deg: f64,
        #[serde(default)]
        upward_sampling: bool,
    },
    Tube {
        length_m: f64,
        inner_diameter_m: f64,
        #[serde(default)]
        inclination_deg: f64,
    },
    Bend {
        angle_deg: f64,
        radius_m: f64,
        #[serde(default)]
        inner_diameter_m: Option<f64>,
    },
    Contraction {
        outlet_diameter_m: f64,
        half_angle_deg: f64,
        #[serde(default)]
        length_m: f64,
    },
    Buffer {
        volume_m3: f64,
    },
    Split {
        sensor_m3_s: f64,
    },
}
fn default_phi() -> f64 {
    90.
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    pub total_m3_s: f64,
    pub sensor_m3_s: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sensor {
    /// First-order time constant, not t90.
    #[serde(default)]
    pub response_time_s: f64,
    #[serde(default)]
    pub sampling_interval_s: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default)]
    pub extend_laminar_transition: bool,
    #[serde(default)]
    pub aspiration_mode: AspirationMode,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AspirationMode {
    #[default]
    Auto,
    Moving,
    Slow,
    Calm,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Distribution {
    /// Number-lognormal modes in PHYSICAL diameter.
    pub modes: Vec<Mode>,
    #[serde(default = "default_bins")]
    pub quadrature_bins: usize,
}
fn default_bins() -> usize {
    64
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mode {
    pub number_concentration_m3: f64,
    pub geometric_mean_diameter_m: f64,
    pub geometric_std_dev: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub riding_speed_m_s: f64,
    pub headwind_m_s: f64,
    pub crosswind_m_s: f64,
    pub apparent_speed_m_s: f64,
    pub yaw_from_heading_deg: f64,
    pub weight: f64,
}
pub fn apparent_wind(riding: f64, wind_speed: f64, direction_from_heading_deg: f64) -> (f64, f64) {
    let theta = direction_from_heading_deg.to_radians();
    let x = riding + wind_speed * theta.cos();
    let y = wind_speed * theta.sin();
    (x.hypot(y), y.atan2(x).to_degrees())
}
fn weights(n: usize, w: &Option<Vec<f64>>, path: &str) -> Result<Vec<f64>> {
    if let Some(w) = w {
        if w.len() != n
            || w.iter().any(|v| !v.is_finite() || *v < 0.)
            || !w.iter().sum::<f64>().is_finite()
            || w.iter().sum::<f64>() <= 0.
        {
            return Err(Diagnostic::new(
                "E_WEIGHTS",
                path,
                "weights must match samples, be finite/nonnegative and have positive finite sum",
            ));
        }
        let s = w.iter().sum::<f64>();
        Ok(w.iter().map(|v| v / s).collect())
    } else {
        Ok(vec![1. / n as f64; n])
    }
}
impl Case {
    pub fn particles(&self) -> Vec<Particle> {
        if !self.particles.aerodynamic_diameter_m.is_empty() {
            self.particles
                .aerodynamic_diameter_m
                .iter()
                .map(|d| {
                    Particle::from_aerodynamic(
                        *d,
                        self.particles.density_kg_m3,
                        self.particles.shape_factor,
                        &self.air,
                    )
                })
                .collect()
        } else {
            self.particles
                .physical_diameter_m
                .iter()
                .map(|d| Particle {
                    physical_diameter_m: *d,
                    density_kg_m3: self.particles.density_kg_m3,
                    shape_factor: self.particles.shape_factor,
                })
                .collect()
        }
    }
    pub fn scenarios(&self) -> Result<Vec<Scenario>> {
        let p = &self.platform;
        let sw = weights(
            p.speed_m_s.len(),
            &p.speed_weights,
            "platform.speed_weights",
        )?;
        let mut winds = vec![];
        if let Some(samples) = &p.wind_samples {
            if samples.is_empty() || samples.len() > 128 {
                return Err(Diagnostic::new(
                    "E_SAMPLES",
                    "platform.wind_samples",
                    "expected 1..128 wind samples",
                ));
            }
            if p.crosswind_m_s != [0.]
                || p.headwind_m_s != [0.]
                || p.crosswind_weights.is_some()
                || p.headwind_weights.is_some()
            {
                return Err(Diagnostic::new(
                    "E_WIND",
                    "platform",
                    "wind_samples cannot be combined with component wind arrays/weights",
                ));
            }
            let ww = weights(
                samples.len(),
                &Some(samples.iter().map(|s| s.weight).collect()),
                "platform.wind_samples.weight",
            )?;
            for (i, s) in samples.iter().enumerate() {
                nonnegative(s.speed_m_s, "platform.wind_samples.speed_m_s")?;
                if !s.direction_from_heading_deg.is_finite() {
                    return Err(Diagnostic::new(
                        "E_ANGLE",
                        "platform.wind_samples.direction_from_heading_deg",
                        "must be finite",
                    ));
                }
                let t = s.direction_from_heading_deg.to_radians();
                winds.push((s.speed_m_s * t.cos(), s.speed_m_s * t.sin(), ww[i]));
            }
        } else {
            let cw = weights(
                p.crosswind_m_s.len(),
                &p.crosswind_weights,
                "platform.crosswind_weights",
            )?;
            let hw = weights(
                p.headwind_m_s.len(),
                &p.headwind_weights,
                "platform.headwind_weights",
            )?;
            for (i, h) in p.headwind_m_s.iter().enumerate() {
                for (j, c) in p.crosswind_m_s.iter().enumerate() {
                    winds.push((*h, *c, hw[i] * cw[j]));
                }
            }
        }
        if winds.len() * p.speed_m_s.len() > 512 {
            return Err(Diagnostic::new(
                "E_RESOURCE",
                "platform",
                "at most 512 Cartesian scenarios",
            ));
        }
        let mut scenarios = vec![];
        for (i, speed) in p.speed_m_s.iter().enumerate() {
            for (h, c, w) in &winds {
                if sw[i] * w == 0. {
                    continue;
                }
                let x = speed + h;
                scenarios.push(Scenario {
                    riding_speed_m_s: *speed,
                    headwind_m_s: *h,
                    crosswind_m_s: *c,
                    apparent_speed_m_s: x.hypot(*c),
                    yaw_from_heading_deg: c.atan2(x).to_degrees(),
                    weight: sw[i] * w,
                });
            }
        }
        Ok(scenarios)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != case_schema() {
            return Err(Diagnostic::new(
                "E_SCHEMA",
                "schema",
                "expected inlet-craft.case/0.1",
            ));
        }
        self.air.validate()?;
        for (name, arr, positive) in [
            ("speed_m_s", &self.platform.speed_m_s, true),
            ("crosswind_m_s", &self.platform.crosswind_m_s, false),
            ("headwind_m_s", &self.platform.headwind_m_s, false),
        ] {
            if arr.is_empty() || arr.len() > 128 {
                return Err(Diagnostic::new(
                    "E_SAMPLES",
                    &format!("platform.{name}"),
                    "expected 1..128 samples",
                ));
            }
            for (i, v) in arr.iter().enumerate() {
                let path = format!("platform.{name}[{i}]");
                if positive {
                    nonnegative(*v, &path)?;
                } else if !v.is_finite() {
                    return Err(Diagnostic::new(
                        "E_FINITE",
                        &path,
                        "expected finite wind component",
                    ));
                }
            }
        }
        let p = &self.particles;
        if p.aerodynamic_diameter_m.is_empty() == p.physical_diameter_m.is_empty() {
            return Err(Diagnostic::new(
                "E_PARTICLES",
                "particles",
                "supply exactly one nonempty diameter basis",
            ));
        }
        let ds = if p.aerodynamic_diameter_m.is_empty() {
            &p.physical_diameter_m
        } else {
            &p.aerodynamic_diameter_m
        };
        if ds.len() > 256 {
            return Err(Diagnostic::new(
                "E_RESOURCE",
                "particles",
                "at most 256 diameters",
            ));
        }
        for (i, d) in ds.iter().enumerate() {
            finite_positive(*d, &format!("particles.diameter[{i}]"))?;
            if *d < 1e-10 || *d > 1e-3 {
                return Err(Diagnostic::new(
                    "E_DIAMETER",
                    &format!("particles.diameter[{i}]"),
                    "supported computational diameter range 1e-10..1e-3m",
                ));
            }
        }
        Particle {
            physical_diameter_m: ds[0],
            density_kg_m3: p.density_kg_m3,
            shape_factor: p.shape_factor,
        }
        .validate()?;
        finite_positive(self.flow.total_m3_s, "flow.total_m3_s")?;
        finite_positive(self.flow.sensor_m3_s, "flow.sensor_m3_s")?;
        if self.flow.sensor_m3_s > self.flow.total_m3_s {
            return Err(Diagnostic::new(
                "E_FLOW",
                "flow.sensor_m3_s",
                "sensor flow exceeds total flow",
            ));
        }
        nonnegative(self.sensor.response_time_s, "sensor.response_time_s")?;
        nonnegative(
            self.sensor.sampling_interval_s,
            "sensor.sampling_interval_s",
        )?;
        if self.elements.is_empty() || self.elements.len() > 100 {
            return Err(Diagnostic::new(
                "E_ELEMENTS",
                "elements",
                "expected 1..100 elements, first a probe",
            ));
        }
        let mut diameter = 0.;
        let mut split = false;
        for (i, e) in self.elements.iter().enumerate() {
            let path = format!("elements[{i}]");
            match e {
                Element::Probe {
                    inner_diameter_m,
                    outer_diameter_m,
                    angle_to_heading_deg,
                    angle_to_vertical_deg,
                    ..
                } => {
                    if i != 0 {
                        return Err(Diagnostic::new(
                            "E_PROBE",
                            &path,
                            "exactly one probe must be first",
                        ));
                    }
                    finite_positive(*inner_diameter_m, &path)?;
                    diameter = *inner_diameter_m;
                    if let Some(outer) = outer_diameter_m {
                        finite_positive(*outer, &path)?;
                        if *outer < diameter || outer / diameter >= 1.1 {
                            return Err(Diagnostic::new(
                                "E_UNCOVERED",
                                &path,
                                "thin wall requires 1<=outer/inner<1.1",
                            ));
                        }
                    }
                    if !angle_to_heading_deg.is_finite()
                        || angle_to_heading_deg.abs() > 180.
                        || !angle_to_vertical_deg.is_finite()
                        || !(0. ..=90.).contains(angle_to_vertical_deg)
                    {
                        return Err(Diagnostic::new(
                            "E_ANGLE",
                            &path,
                            "heading angle -180..180deg and vertical angle 0..90deg required",
                        ));
                    }
                }
                Element::Tube {
                    length_m,
                    inner_diameter_m,
                    inclination_deg,
                } => {
                    finite_positive(*inner_diameter_m, &path)?;
                    nonnegative(*length_m, &path)?;
                    if (inner_diameter_m - diameter).abs() > 1e-10 * diameter {
                        return Err(Diagnostic::new(
                            "E_UNCOVERED",
                            &path,
                            "diameter changes require an explicit contraction; enlargements unsupported",
                        ));
                    }
                    if !inclination_deg.is_finite() || inclination_deg.abs() > 90. {
                        return Err(Diagnostic::new(
                            "E_ANGLE",
                            &path,
                            "inclination must be -90..90deg",
                        ));
                    }
                }
                Element::Bend {
                    angle_deg,
                    radius_m,
                    inner_diameter_m,
                } => {
                    nonnegative(*angle_deg, &path)?;
                    finite_positive(*radius_m, &path)?;
                    if *angle_deg > 3600. {
                        return Err(Diagnostic::new(
                            "E_ANGLE",
                            &path,
                            "bend angle <=3600deg required",
                        ));
                    }
                    if let Some(d) = inner_diameter_m {
                        finite_positive(*d, &path)?;
                        if (*d - diameter).abs() > 1e-10 * diameter {
                            return Err(Diagnostic::new(
                                "E_DIAMETER",
                                &path,
                                "bend diameter must match preceding tube",
                            ));
                        }
                    }
                }
                Element::Contraction {
                    outlet_diameter_m,
                    half_angle_deg,
                    length_m,
                } => {
                    finite_positive(*outlet_diameter_m, &path)?;
                    nonnegative(*length_m, &path)?;
                    if *outlet_diameter_m >= diameter {
                        return Err(Diagnostic::new(
                            "E_UNCOVERED",
                            &path,
                            "contraction outlet must be smaller than inlet; enlargements unsupported",
                        ));
                    }
                    if !half_angle_deg.is_finite() || *half_angle_deg <= 0. || *half_angle_deg > 90.
                    {
                        return Err(Diagnostic::new(
                            "E_ANGLE",
                            &path,
                            "contraction half-angle must be (0,90]deg",
                        ));
                    }
                    diameter = *outlet_diameter_m;
                }
                Element::Buffer { volume_m3 } => {
                    nonnegative(*volume_m3, &path)?;
                }
                Element::Split { sensor_m3_s } => {
                    if split {
                        return Err(Diagnostic::new(
                            "E_FLOW",
                            &path,
                            "only one ideal split supported",
                        ));
                    }
                    split = true;
                    finite_positive(*sensor_m3_s, &path)?;
                    if (*sensor_m3_s - self.flow.sensor_m3_s).abs() > 1e-10 * self.flow.total_m3_s {
                        return Err(Diagnostic::new(
                            "E_FLOW",
                            &path,
                            "split flow must equal flow.sensor_m3_s",
                        ));
                    }
                }
            }
            if i == 0 && !matches!(e, Element::Probe { .. }) {
                return Err(Diagnostic::new(
                    "E_PROBE",
                    &path,
                    "first element must be a thin-walled probe",
                ));
            }
        }
        if let Some(d) = &self.distribution {
            if d.modes.is_empty()
                || d.modes.len() > 8
                || d.quadrature_bins < 16
                || d.quadrature_bins > 256
            {
                return Err(Diagnostic::new(
                    "E_DISTRIBUTION",
                    "distribution",
                    "1..8 modes and 16..256 quadrature bins required",
                ));
            }
            for (i, m) in d.modes.iter().enumerate() {
                let path = format!("distribution.modes[{i}]");
                finite_positive(m.number_concentration_m3, &path)?;
                finite_positive(m.geometric_mean_diameter_m, &path)?;
                if !(1e-10..=1e-3).contains(&m.geometric_mean_diameter_m) {
                    return Err(Diagnostic::new(
                        "E_DIAMETER",
                        &path,
                        "mode geometric mean must be within1e-10..1e-3m computational range",
                    ));
                }
                if !m.geometric_std_dev.is_finite()
                    || m.geometric_std_dev < 1.
                    || m.geometric_std_dev > 5.
                {
                    return Err(Diagnostic::new(
                        "E_DISTRIBUTION",
                        &path,
                        "geometric_std_dev must be 1..5 (1=monodisperse)",
                    ));
                }
            }
        }
        let n = self.scenarios()?.len();
        let grid = self
            .distribution
            .as_ref()
            .map_or(0, |d| 2 * d.modes.len() * d.quadrature_bins);
        if n * (ds.len() + grid) * self.elements.len() > 2_000_000 {
            return Err(Diagnostic::new(
                "E_RESOURCE",
                "case",
                "scenario/particle/element work estimate exceeds2000000; reduce quadrature or samples",
            ));
        }
        Ok(())
    }
}
pub fn parse<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    let mut de = serde_json::Deserializer::from_str(text);
    let value = serde_path_to_error::deserialize(&mut de)
        .map_err(|e| Diagnostic::new("E_JSON", &e.path().to_string(), e.inner().to_string()))?;
    de.end()
        .map_err(|e| Diagnostic::new("E_JSON", "$", e.to_string()))?;
    Ok(value)
}
