use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub path: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
}
impl Diagnostic {
    pub fn new(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            message: message.into(),
            quantity: None,
            value: None,
            allowed: None,
            formula: None,
        }
    }
}
pub type Result<T> = std::result::Result<T, Diagnostic>;
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.path, self.message)
    }
}
impl std::error::Error for Diagnostic {}
pub fn range(
    w: &mut Vec<Diagnostic>,
    path: &str,
    formula: &str,
    quantity: &str,
    value: f64,
    lo: f64,
    hi: f64,
) {
    if value < lo || value > hi {
        let mut d = Diagnostic::new(
            "W_RANGE",
            path,
            format!(
                "{formula}: {quantity}={value:.6e} outside [{lo:.6e}, {hi:.6e}]; extrapolated without clamping"
            ),
        );
        d.quantity = Some(quantity.into());
        d.value = Some(value);
        d.allowed = Some([lo, hi]);
        d.formula = Some(formula.into());
        w.push(d);
    }
}
pub fn finite_positive(x: f64, path: &str) -> Result<()> {
    if !x.is_finite() || x <= 0. {
        Err(Diagnostic::new(
            "E_POSITIVE",
            path,
            "expected a finite positive SI value",
        ))
    } else {
        Ok(())
    }
}
pub fn nonnegative(x: f64, path: &str) -> Result<()> {
    if !x.is_finite() || x < 0. {
        Err(Diagnostic::new(
            "E_NONNEGATIVE",
            path,
            "expected a finite nonnegative SI value",
        ))
    } else {
        Ok(())
    }
}
pub fn efficiency(x: f64, transport: bool, path: &str, w: &mut Vec<Diagnostic>) -> Result<f64> {
    if !x.is_finite() {
        return Err(Diagnostic::new(
            "E_NUMERIC",
            path,
            "correlation produced a nonfinite value",
        ));
    }
    if x < 0. || (transport && x > 1. + 1e-12) {
        w.push(Diagnostic::new(
            "W_NONPHYSICAL",
            path,
            format!("extrapolated efficiency {x}; returned unmodified"),
        ));
    }
    Ok(x)
}
