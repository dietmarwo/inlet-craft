# inlet-craft implementation and verification log

> **Note, 2026-11-15:** this is the delivery log of the 0.1.0 source candidate. A later review corrected the right-angle aspiration exponent to the primary source (the PLC paper misprints it), corrected the oblique-probe wall-impaction term to the printed equation, and changed the search's handling of extrapolated efficiencies; the full-distribution bicycle studies now find accepted designs. See CHANGELOG.md and docs/bicycle-study.md for current results. The figures below are the original record.

Date: 2026-11-14 (user timezone: America/Argentina/Salta).
Deliverable: complete new Rust project, source candidate 0.1.0, based on both supplied documents.

## Outcome

Implemented the Rust library, command line, native Python binding, fcmaes DE/MODE search, optimization-jobs SDK worker, examples, schemas, documentation/reference/static site, CI configuration and release preparation. The ZIP is a complete source tree rather than a patch against a nonexistent starting repository. It includes pinned ecosystem source dependencies and the raw numerical evidence; build outputs and Cargo registry cache are excluded.

**Software checks pass. Scientific validation is provisional.** In particular, **four of six approximate published virtual-system anchors fail**. At 150 L/min the implementation predicts roughly 5–6% loss where the paper's graph indicates about 40%. This is a material unresolved discrepancy. It is not explained away by minor air-property differences, fitted away, or labeled as passed. G0/G1 are not closed, G2 awaits practitioner acceptance, and no physical experiment or independent particle-tracking CFD was performed. No release tag, package publication, external repository creation or site deployment was performed.

## Test results

| Check | Observed result |
|---|---|
| Release Rust workspace tests | **32 passed, 0 failed** |
| Reference/identity/high-precision fixtures | **33/33 passed**; evidence types distinguished |
| Installed aerosolpy comparison | **150/150 grid cases passed** |
| Strict Clippy | Passed workspace/all-targets with warnings denied |
| Rust formatting | Passed `cargo fmt --all -- --check` |
| Native Python wheel | Built and installed for CPython 3.12/Linux x86_64; CLI/Python report parity passed |
| Python integration | 8 checks passed normally and under `python -O` |
| SDK worker | Validate, execute, checksummed journal, evidence and run manifest passed |
| Cancellation | Pre-existing cancel and in-flight cancel completed within the 10 s test bound |
| JSON schemas | Case/result integration plus 4 additional study/search/verification validations passed |
| Documentation | 16 authored Markdown files inventoried; 2 authored executable fences run; 5 schemas generated |
| Published virtual system | **2/6 approximate anchors passed; 4 failed and retained** |
| AeroSampleR / IGOR Pro | Not executed; Rscript/IGOR runtime unavailable |
| On-bike experimental validation | Not performed; draft preregistration protocol supplied |

Fixture composition: reference_parameter: 1, independent_equation_arithmetic: 27, analytic_identity: 2, published_approximate_adjacent_geometry: 3. Formula arithmetic is not a published measurement. The 3 approximate published bend anchors use adjacent geometry and an a priori absolute 0.06 tolerance; they do not establish precision physical validation.

The aerosolpy grid spans 25 physical diameters from 10nm to 10µm, 3 temperatures(273.15,293.15,313.15K) and 2 pressures(80000,101325Pa), 150 cases total. Acceptance thresholds were fixed before running:5% relative slip, 5% relative diffusion and 0.02 absolute diffusional penetration. Observed maxima:

- Slip correction: **1.3185% relative difference**.
- Diffusion coefficient: **1.0655% relative difference**.
- Diffusional penetration: **0.0039174 absolute difference**.

Differences are expected because aerosolpy uses different mean-free-path/slip choices and Gormley–Kennedy diffusion, while this implementation selects Allen–Raabe/Holman. No coefficient was adjusted to improve agreement. The oracle is an installed MIT package, not copied source.

## Increment record

| Increment | Implementation and checks | Status against the supplied plan |
|---|---|---|
| I0 | Formula registry written first; original/standard-source attribution, source conflicts, validity, licensing and existing-tool search recorded | Registry delivered; stronger G0 source/number completeness not met |
| I1 | Dry-air viscosity/density/mean free path, slip, relaxation/settling/diffusion, density/shape-aware implicit diameter conversion | Implemented; dimensional/roundtrip/grid checks passed |
| I2 | Facing, oblique, right-angle, calm/slow aspiration; probe transmission and endpoint diagnostics | Implemented with explicit fitted ranges and uncovered geometry |
| I3 | Laminar/turbulent tubes, opt-in transition estimate, bends, contractions, mixing buffers, ideal split and serial composition | Implemented with explicit omitted physics; published system reproduction fails 4 anchors; G1 open |
| I4 | Strict nested SI JSON, schemas, describe/check/evaluate/verify, native Python bridge | Implemented; schema/CLI/native checks passed |
| I5 | Apparent wind and weighted scenarios, tilt/yaw geometry, number-lognormal modes, ideal-cut mass response, time/position/resolution, sweep | Implemented; analytic and independent CTMC checks passed |
| I6 | Actual fcmaes DE and MODE ask/tell, budgeted evaluation, constraints, cold checks, SDK worker and cancellation | Implemented; 7 studies/2624 optimizer evaluations; practitioner acceptance pending |
| I7 | Progressive guide, how-to, source/evidence comparison, generated reference/site, executable fences, CI/release preparation | Delivered locally; hosted CI and publication not observed |
| TrackV | Draft independent-code and on-bike protocol; optional AeroSampleR export adapter | External R/IGOR/field/CFD evidence still pending |

A separate agent reviewed the increments without writing the solver or guide. Its report is `docs/developer/independent-review.md`. It independently ran 31 core tests and strict Clippy at its review checkpoint; the final primary run adds the independent unequal-three-tank regression, totaling 32 tests. The reviewer independently checked the three-unequal-tank quantiles against SciPy's matrix exponential plus Brent roots: maximum absolute error about 1.7e-13s against a predeclared 1e-10s tolerance. It also ran an additional 128-evaluation coarse MODE study with 8 accepted, cold-checked nondominated points; the main final campaign below is separate.

## Source and implementation decisions

1. **Contraction convention corrected.** Brockmann's standard-reference equation contains `2*Stk*(1-Ao/Ai)`; the PLC printed equation omits 2. The implementation chooses the standard-reference factor 2 and records `W_SOURCE_CONFLICT`. Stokes number uses upstream velocity and downstream diameter, not downstream velocity.
2. **Angles and Stokes definitions corrected.** Bend equations use radians and diameter-based Stokes number. Calm aspiration uses inlet velocity, moving aspiration uses freestream velocity. The physical/aerodynamic conversion preserves equal relaxation time and does not apply density twice.
3. **Wind/tilt geometry made explicit.** For horizontal wind, `cos(theta)=sin(phi)*cos(yaw)`. A vertical inlet sees 90 degrees. The slow-air velocity correction uses the orthogonality of horizontal wind and vertical settling, documented as a geometric extension of the scalar source. Downward-facing calm/slow apertures are rejected where the chosen formula cannot support them.
4. **No hidden continuity correction.** Empirical angle endpoints report their jumps; the 60–61degree gap is rejected. A1e-10degree endpoint classification tolerance handles floating-point rounding without filling the real gap. Zero wind uses calm; positive wind up to 1.5m/s uses the explicitly documented slow blend.
5. **Time response does not hide stiffness failure.** One/two tanks use analytic limits; general chains use bounded uniformization. A nonfinite CDF propagates to `E_RESPONSE`. Preflight work limits prevent the earlier possible NaN-to-finite-quantile error and stiff-chain hangs. Averaging is trailing, so its mean delay is half its interval.
6. **No invented buffer loss.** Buffer volume modelsV/Qonly. Bend arc diffusion/travel time is included, but unknown arc settling and coupled deposition are warned. Taper volume contributes travel time; taper-wall diffusion/settling is not invented. Ideal flow splits preserve concentration and partition particle flux.
7. **Budget and evidence semantics.** Ask/tell stops at the exact optimizer evaluation budget; baseline and cold checks are separately counted. Returned candidates are rebuilt/re-evaluated; cancellation/deadline also stops rechecks. A finite unsuccessful search never asserts global infeasibility. MODE uses 3 objectives and 1 aggregate constraint, not changing scalar weights against stale objective scores.
8. **Integration harness corrected.** SDK journals are checksummed envelopes containing `payload_json`; the checker now verifies checksums and parses the actual record before examining its outcome. Both normal and optimized Python runs pass.

No PLC software source was read, downloaded, ported or used as an oracle. The published paper, standard-reference lecture/notes and original-source metadata were consulted. Where primary full text or an independent published point was unavailable, that gap remains in the ledger.

## Bicycle study results

The baseline has a 4 mm inlet, 2 L/min total flow, 15 cm tube, 90 degree bend of 2 cm radius, 20 mL buffer and sensor tau 1 s with 1 s averaging. Speed scenarios are 10/15/20/25 km/h and crosswind 0/2m/s. The declared bimodal aerosol is an illustrative assumption, not measured site data.

At baseline, PM2.5 bias is approximately −0.35% to +1.20%; PM10 bias is approximately −27.3% to −5.37%. These are integrated model outputs with validity warnings, not validated sensor accuracy.

The main goal is PM10 within 15%, resolution below 15 m **at 18 km/h**, total flow at most 3 L/min, laminar flow. The maximum resolution over all speeds is also reported separately. A reference-speed field was added because imposing 15 m at 25 km/h would be a different and stronger requirement.

| Study | Evaluations | Outcome | Worst PM10 bias | Resolution at 18 km/h, m | Flow, L/min | Rechecked Pareto points |
|---|---:|---|---:|---:|---:|---:|
| full-distribution-seed42 | 512 | no_feasible_design_found | 14.594% | 11.808 | 3.000 | 0 |
| full-distribution-seed7 | 512 | no_feasible_design_found | 14.538% | 12.147 | 3.000 | 0 |
| full-distribution-seed123 | 512 | no_feasible_design_found | 14.423% | 14.712 | 3.000 | 0 |
| mode-full-distribution | 256 | no_feasible_design_found | 15.319% | 13.139 | 2.828 | 0 |
| coarse-monodisperse-sensitivity | 512 | feasible_design_found | 9.923% | 13.566 | 3.000 | 20 |
| mode-coarse-monodisperse-sensitivity | 256 | feasible_design_found | 10.044% | 12.191 | 2.939 | 25 |
| deliberately-impossible-zero-resolution | 64 | no_feasible_design_found | 10.954% | 15.183 | 2.871 | 0 |

The full-distribution DE runs achieve integrated bias/resolution numbers near or inside the requested numeric limits, but **none is accepted** because of the fine-tail nonphysical-extrapolation guard. For sub-isokinetic sampling, the extrapolated Liu transmission becomes slightly greater than 1 below a Stokes number of roughly 0.00388, outside the source's fitted range starting at 0.01. Even a tiny-mass fine tail can trigger that guard. The evaluator preserves the raw value and diagnostics; search refuses it. This is a correlation/acceptance limitation, **not evidence that the hardware requirements cannot be met**.

The coarse-only5 µm monodisperse sensitivity demonstrates accepted DE/MODE tradeoffs under a different explicitly stated distribution. It does not validate or recommend that design for realistic ambient aerosol. Source extrapolation warnings remain even for accepted coarse candidates. The zero-resolution study deliberately tests truthful search failure with a nonzero sensor time constant.

## Failed published-system comparison

Geometry was transcribed from von der Weiden 2009 Figure 7: 3 m/s freestream, 20 degree misalignment in the vertical plane, 7 m of 50 mm tube, 70 degree arc 0.3 m long, 0.5 m vertical tube, 90 degree arc 0.3 m long, 45 degree half-angle contraction 0.2 m long, then 5 m of 10 mm tube. Air 293.15K/101325Pa is assumed because the figure does not specify it. Approximate graph anchors and an absolute 0.02 loss tolerance were fixed before execution.

| Total flow, L/min | Diameter, µm | Approximate published loss | Computed loss | Check |
|---:|---:|---:|---:|---|
| 10 | 0.1 | 1.00% | 1.38% | PASS |
| 10 | 0.3 | 0.32% | 1.25% | PASS |
| 40 | 0.1 | 6.00% | 2.84% | FAIL |
| 40 | 0.3 | 5.00% | 2.14% | FAIL |
| 150 | 0.1 | 40.00% | 6.20% | FAIL |
| 150 | 0.3 | 40.00% | 5.54% | FAIL |

The reviewer rechecked the printed turbulent-deposition equation and the standard-reference wall-unit conversion; no missing printed exponent or justified coefficient correction was found. The 150L/min small-tube Re also exceeds the stated 15600 limit. Air-state uncertainty, omitted arc settling and taper-wall treatment are real limitations, but have not been shown to explain the large fine-particle discrepancy. The whole-system reproduction claim is therefore withheld. `scripts/compare_virtual.py` exits 1 on these mismatches; `--report-only` records them without falsely closing G1. CI uses that explicitly labeled reporting mode while retaining the failed rows.

Raw results: `results/plc-virtual-comparison.json` and `results/plc-virtual-10lpm.json`, `...40lpm.json`, `...150lpm.json`. No tolerance was loosened after seeing these results.

## Performance baseline

For 100 full bicycle evaluations through the installed native Python dict/JSON bridge, including serialization: median **14.721ms**, 95th percentile **16.264ms**, about **65.2 evaluations/s**. Process peak RSS was **12.36MiB** in this measured process. Each case has 8 scenarios, 4 requested sizes, 2 size modes and 48 quadrature bins per mode/cut. One process was used; no objective-level parallelism is claimed. This new project has no previous performance baseline, so no speedup factor is reported. Timings are environment-specific, not a prediction for the user's machines.

## Reproducibility and layout

- `crates/inlet-craft`: primitives, correlations, strict models, diagnostics, evaluator, response, verification and search.
- `crates/inlet-craft-cli`: CLI and actual SDK worker.
- `crates/inlet-craft-py`, `python/inlet_craft`, `pyproject.toml`: native Python binding and build configuration.
- `examples/`: stationary/bicycle/published-virtual cases, bicycle study and SDK job.
- `fixtures/`:33 reference fixtures and generated 20 entry source/validity catalog.
- `schemas/`: case/result/study/search/verification JSON Schemas generated from Rust.
- `scripts/`: fixture/catalog generation, integrations, installed-package comparison, bounded studies, published-system comparison, timing and executable-documentation checker.
- `docs/`, `site/index.html`: guide, architecture, comparison, evidence/protocols, traceability, independent review and self-contained static documentation.
- `results/`: raw fixture/oracle/study/virtual/integration/schema/timing outputs and final check logs.

Build/test commands are given in README.md. The source ZIP excludes target/ and virtual environments. Cargo registry dependencies require a connection on a first clean machine; their versions are locked. The tested wheel was built locally but is not a PyPI release. Rust compiler observed: `rustc 1.100.0 (0ef1bc386 2026-11-09)`; Python observed:`3.12.14`. The declared Rust minimum 1.88 was not separately installed/tested in this environment. Hosted GitHub CI is configured but was not run here.

Pinned ecosystem sources, retrieved from upstream HEAD on 2026-11-14:

| Repository | Revision |
|---|---|
|fcmaes-rust|`c398fef40d66853f0e892b775ae5eea4a8172620`|
|optimization-jobs|`1356ee6ab1335cf31fa3da77c7f67f7d9a2fba6d`|

Only inherited fcmaes Cargo package metadata was made standalone; optimizer/SDK algorithm source is unchanged. MIT licenses and per-dependency provenance files are included. AeroSampleR and aerosolpy source are not vendored.

## Remaining limitations and next scientific work

- Complete original-source and published-number coverage for each correlation; G0 is still incomplete.
- Resolve the published virtual-inlet mismatch before claiming whole-system agreement; compare declared cases through an independent IGOR user and installed AeroSampleR, not another transcription of the same formula.
- Obtain a justified treatment of small-Stokes transmission before claiming accepted full-spectrum bike designs; do not clip coefficients or ignore the failed rows.
- Add a verified buffer-wall model/arc-settling geometry only when literature and a user requirement support them.
- Obtain practitioner review of the study, then preregister and run actual flow/step/co-location measurements. No instrument/housing/humidity/wake accuracy is inferred from these equations.
- Compact search results retain the first 256 failed candidate rows plus an explicit omitted count and at most 256 archive points. Full fixture/published-comparison rows are retained. This is a documented bounded-memory departure from an unlimited failure ledger.
- The AeroSampleR export adapter is supplied but unexecuted because Rscript was absent. No R/PLC/experimental agreement is claimed.

## Input provenance

The supplied plan/design are preserved verbatim under docs/design. Their SHA-256 values:

| Input | SHA-256 |
|---|---|
| inlet-craft-incremental-plan-2026-11-14.md | `847bf06fe83ff59f6216eac0b4d39ce807be37654dc0ef22da41617579a02d5c` |
| inlet-craft-sensor-unit-design-2026-11-14.md | `b7f28c1093c68bdf703871d67c44efc99729f4ae2833a954d5b14541ed7fc3c6` |

Primary evidence links: [Brockmann notes](https://www.osti.gov/servlets/purl/1675151), [Brockmann slides](https://www.osti.gov/servlets/purl/1675150), [PLC paper](https://doi.org/10.5194/amt-2-479-2009), [Allen–Raabe](https://doi.org/10.1080/02786828508959055), [Muyshondt contraction](https://doi.org/10.1080/02786829608965364), [Lee–Gieseke](https://doi.org/10.1016/0021-8502(94)90011-6). The source/validity ledger gives formula-specific attribution and uncertainty.

## Source archive delivery check

ZIP integrity and every SHA-256 manifest entry were checked. The extracted project also passed `cargo check --workspace --locked --offline` from its extracted location, covering all three first-party crates and the relocated vendored dependencies. Only this log and delivery-check evidence were added after that code check.
