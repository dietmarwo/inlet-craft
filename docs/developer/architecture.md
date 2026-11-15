# Architecture

The workspace has three first-party crates. `inlet-craft` owns SI primitives, strict models, literature equations, diagnostics, compositional evaluation, response, evidence fixtures and design search. `inlet-craft-cli` owns JSON command dispatch and the separate actual SDK worker binary. `inlet-craft-py` is a PyO3 JSON bridge with a small dict wrapper. The core and executables forbid unsafe Rust; external dependencies are not covered by that first-party attribute.

Input validation precedes physics. The aerodynamic/physical conversion is implicit because Cunningham slip depends on diameter. The evaluator resolves apparent wind and inlet orientation, computes aspiration separately from transmission, then multiplies transport mechanisms. Each requested size has an element breakdown; distribution quadrature uses the same path and aggregates diagnostic ranges. Buffers/splits are explicit assumptions rather than hidden empirical formulas.

Response is independent of particle size and computed once per case. One/two first-order stages use analytic formulas, repeated poles have a stable equal-tank limit, and longer chains use phase-type uniformization with preflight work caps. Plug delay and the trailing average are added to the response. All public JSON includes precise engineering definitions of mean age/resolution.

The search wrapper mutates only declared design fields, validates every candidate, and calls actual fcmaes DE or MODE. MODE uses3 objectives and1 aggregate violation. The solver's state cannot alter the evaluator. Returned points are rebuilt and evaluated cold, then constraint-checked again. Invalid rows remain evidence, bounded in the compact search JSON. Walltime/cancellation are checked between objectives and before each cold recheck; a runner provides hard process limits.

Versioned case/result/study/search/verification schemas are generated from Rust types with schemars. `describe` embeds the formula catalog and complete provenance ledger. `scripts/check_docs.py` checks generated artifacts and executes authored code fences. A static documentation site is generated for hosting; nothing is deployed automatically.

The two ecosystem dependencies are source snapshots from upstream HEAD retrieved on2026-11-14, recorded in each `vendor/*/VENDORED.md`. Their algorithm code is unchanged; only fcmaes workspace-inherited Cargo metadata is made standalone. This makes the source ZIP independent of later ecosystem git changes. Third-party registry crates still come from Cargo with the lockfile.

Numerical resource limits include512 scenarios,256 requested diameters,100 elements,8 modes,16–256 quadrature bins, and a2million scenario/particle/element work estimate. The response solver has a separate cap. These bounds prevent accidental unbounded allocations or stiff-chain hangs; they are computational support limits, not experimentally established aerosol limits.

Outstanding scientific work is listed in the evidence ledger and validation protocol. The architecture deliberately leaves room for verified alternative correlations; it does not silently replace missing physics with fitted constants.
