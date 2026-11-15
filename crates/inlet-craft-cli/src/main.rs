#![allow(clippy::result_large_err)] // Structured diagnostics are intentionally returned by value.
#![forbid(unsafe_code)]
use inlet_craft::{diagnostic::Diagnostic, *};
use serde_json::{Value, json};
use std::sync::{Arc, atomic::AtomicBool};
fn read(path: &str) -> Result<String> {
    let meta = std::fs::metadata(path).map_err(|e| Diagnostic::new("E_IO", path, e.to_string()))?;
    if meta.len() > 8_000_000 {
        return Err(Diagnostic::new(
            "E_RESOURCE",
            path,
            "maximum input file size 8MB",
        ));
    }
    std::fs::read_to_string(path).map_err(|e| Diagnostic::new("E_IO", path, e.to_string()))
}
fn run(args: &[String]) -> Result<Value> {
    let command = args.first().map(String::as_str).unwrap_or("help");
    let file = || {
        args.get(1)
            .ok_or_else(|| Diagnostic::new("E_USAGE", "argv", "command requires a JSON file"))
    };
    match command {
        "describe" => Ok(describe()),
        "check" | "evaluate" => {
            let case: Case = parse(&read(file()?)?)?;
            let r = evaluate(&case)?;
            if command == "check" {
                Ok(
                    json!({"valid":true,"warning_count":r.warning_count,"warnings":r.warnings,"evidence":r.verification_status}),
                )
            } else {
                Ok(json!(r))
            }
        }
        "verify" => {
            let v = if let Some(file) = args.get(1) {
                verification::verify(parse(&read(file)?)?)
            } else {
                verification::builtin_verify()
            };
            Ok(json!(v))
        }
        "sweep" => {
            let case: Case = parse(&read(file()?)?)?;
            if args.len() == 2 {
                return Ok(
                    json!({"schema":"inlet-craft.sweep/0.1","axis":"declared platform Cartesian distribution","evaluation":evaluate(&case)?}),
                );
            }
            if args.len() != 4 {
                return Err(Diagnostic::new(
                    "E_USAGE",
                    "argv",
                    "sweep CASE JSON_POINTER comma,separated,values",
                ));
            }
            let mut rows = vec![];
            let base = serde_json::to_value(&case)
                .map_err(|e| Diagnostic::new("E_JSON", "case", e.to_string()))?;
            for raw in args[3].split(',') {
                if rows.len() >= 128 {
                    return Err(Diagnostic::new(
                        "E_RESOURCE",
                        "argv",
                        "maximum128 sweep points",
                    ));
                }
                let x: f64 = raw.parse().map_err(|_| {
                    Diagnostic::new("E_USAGE", "argv", "sweep values must be finite numbers")
                })?;
                if !x.is_finite() {
                    return Err(Diagnostic::new("E_USAGE", "argv", "nonfinite sweep value"));
                }
                let mut value = base.clone();
                let dest = value.pointer_mut(&args[2]).ok_or_else(|| {
                    Diagnostic::new("E_VARIABLE", &args[2], "missing JSON pointer")
                })?;
                *dest = json!(x);
                let result: Result<Evaluation> =
                    parse::<Case>(&value.to_string()).and_then(|c| evaluate(&c));
                rows.push(match result {
                    Ok(r) => json!({"value":x,"evaluation":r}),
                    Err(e) => json!({"value":x,"error":e}),
                });
            }
            Ok(json!({"schema":"inlet-craft.sweep/0.1","axis":args[2],"rows":rows}))
        }
        "optimize" => {
            let study: Study = parse(&read(file()?)?)?;
            let cancel = Arc::new(AtomicBool::new(false));
            Ok(json!(optimize(&study, &cancel)?))
        }
        "help" | "--help" | "-h" => Ok(
            json!({"usage":"inlet-craft describe | check CASE | evaluate CASE | verify [FIXTURES] | sweep CASE [JSON_POINTER comma,values] | optimize STUDY","note":"All values SI; outputs JSON; sensor response_time_s means first-order tau"}),
        ),
        _ => Err(Diagnostic::new(
            "E_USAGE",
            "argv",
            "unknown command; run --help",
        )),
    }
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("serializable result")
            );
            if args.first().is_some_and(|a| a == "verify")
                && value.get("all_passed") == Some(&json!(false))
            {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", json!({"error":e}));
            std::process::exit(2);
        }
    }
}
