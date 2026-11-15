use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
fn pyerr(e: inlet_craft::Diagnostic) -> PyErr {
    PyValueError::new_err(serde_json::json!({"error":e}).to_string())
}
#[pyfunction]
fn evaluate_json(case_json: &str) -> PyResult<String> {
    let c = inlet_craft::parse(case_json).map_err(pyerr)?;
    let r = inlet_craft::evaluate(&c).map_err(pyerr)?;
    serde_json::to_string(&r).map_err(|e| PyValueError::new_err(e.to_string()))
}
#[pyfunction]
fn check_json(case_json: &str) -> PyResult<String> {
    let c = inlet_craft::parse(case_json).map_err(pyerr)?;
    let r = inlet_craft::evaluate(&c).map_err(pyerr)?;
    // Same report as `inlet-craft check`.
    Ok(serde_json::json!({
        "valid": true,
        "warning_count": r.warning_count,
        "warnings": r.warnings,
        "evidence": r.verification_status,
    })
    .to_string())
}
#[pyfunction]
fn describe_json() -> String {
    inlet_craft::describe().to_string()
}
#[pyfunction]
fn verify_json() -> String {
    serde_json::to_string(&inlet_craft::verification::builtin_verify()).expect("embedded report")
}
#[pyfunction]
fn optimize_json(study_json: &str) -> PyResult<String> {
    let s = inlet_craft::parse(study_json).map_err(pyerr)?;
    let r = inlet_craft::optimize(&s, &std::sync::atomic::AtomicBool::new(false)).map_err(pyerr)?;
    serde_json::to_string(&r).map_err(|e| PyValueError::new_err(e.to_string()))
}
#[pymodule]
fn _inlet_craft(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(evaluate_json, m)?)?;
    m.add_function(wrap_pyfunction!(check_json, m)?)?;
    m.add_function(wrap_pyfunction!(describe_json, m)?)?;
    m.add_function(wrap_pyfunction!(verify_json, m)?)?;
    m.add_function(wrap_pyfunction!(optimize_json, m)?)?;
    Ok(())
}
