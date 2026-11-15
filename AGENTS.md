# inlet-craft contributor guide

Read README.md, docs/developer/formula-registry.md and docs/validation/protocol.md first. Implement equations from published primary/standard sources; never inspect or port PLC software. Record source conflicts and do not label arithmetic as experimental evidence. Rust core forbids unsafe code. JSON unknown fields are errors, SI units appear in field names, and nonphysical extrapolations are reported without clipping.

Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets --locked -- -D warnings, cargo test --workspace --locked, native wheel/integration checks, and documentation checker. Keep generated schemas/catalog/reference synchronized with describe. Do not tag, publish or deploy automatically. Record changes under Unreleased. The repository follows the wider fcmaes project instructions.
