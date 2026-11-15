#![allow(clippy::result_large_err)] // Structured diagnostics are intentionally returned by value.
#![forbid(unsafe_code)]
//! Aerosol inlet analysis with explicit literature validity diagnostics.
pub mod correlations;
pub mod diagnostic;
pub mod evaluate;
pub mod model;
pub mod primitives;
pub mod response;
pub mod search;
pub mod verification;
pub use diagnostic::{Diagnostic, Result};
pub use evaluate::{Evaluation, evaluate};
pub use model::{Case, parse};
pub use search::{SearchResult, Study, optimize};
/// Machine-readable interfaces, generated from the actual Rust models.
pub fn describe() -> serde_json::Value {
    serde_json::json!({"name":"inlet-craft","version":env!("CARGO_PKG_VERSION"),"schemas":{"case":schemars::schema_for!(Case),"result":schemars::schema_for!(Evaluation),"study":schemars::schema_for!(Study),"search":schemars::schema_for!(SearchResult),"verification":schemars::schema_for!(verification::Verification)},"formulas":serde_json::from_str::<serde_json::Value>(include_str!("../../../fixtures/formula-catalog.json")).expect("embedded catalog"),"formula_registry":include_str!("../../../docs/developer/formula-registry.md"),"diagnostic_codes":["E_JSON","E_IO","E_USAGE","E_SCHEMA","E_POSITIVE","E_NONNEGATIVE","E_SHAPE","E_WEIGHTS","E_SAMPLES","E_RESOURCE","E_FINITE","E_WIND","E_PARTICLES","E_DIAMETER","E_FLOW","E_ELEMENTS","E_PROBE","E_ANGLE","E_UNCOVERED","E_TRANSITION","E_REGIME","E_ANGLE_UNCOVERED","E_NUMERIC","E_RESPONSE","E_DISTRIBUTION","E_BUDGET","E_VARIABLES","E_VARIABLE","E_BOUNDS","E_DIMENSION","E_CASE","E_EMPTY_PM","E_FIXTURE","W_RANGE","W_NONPHYSICAL","W_TRANSITION","W_REGIME_BOUNDARY","W_SLOW_POLICY","W_CALM_TRANSMISSION","W_WALL_ASSUMED","W_BEND_SETTLING","W_SOURCE_CONFLICT","W_CONTRACTION_TRANSPORT","W_BUFFER_LOSS_UNMODELED","W_IDEAL_SPLIT"],"evidence":"provisional_literature_equations; G0/G1 incomplete; V1/V2 not performed"})
}
