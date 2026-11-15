# Changelog

## Unreleased — 0.1.0 source candidate

Initial literature-equation implementation: SI primitives, moving/calm/slow aspiration, probe transmission, tubes/bends/contractions, explicit uncovered-process diagnostics, strict JSON/schema CLI, native Python binding, scenario/PM/time-response evaluation, fcmaes DE/MODE design studies, optimization-jobs worker, evidence fixtures, cancellation/integration/oracle checks, and authored documentation/site source. Scientific gates and external validation remain open as recorded in the evidence ledger.

### Review fixes, 2026-11-15

- **Correctness: right-angle aspiration.** `aspiration.hangal_willeke` now uses the Stokes exponent sqrt(1/R) = sqrt(U/U0). The candidate's sqrt(R) made 90-degree aspiration of inertial particles rise as wind outran the inlet velocity, the opposite of the inertial argument and the experimental trend. The change affects inlets seen at 61–90 degrees (vertical inlets, strong crosswind), not the forward-facing examples. The registry records the evidence and the remaining G0 re-check; a new fixture and a monotonicity test pin it.
- **Design search: quantified extrapolation allowance.** Transport correlations extrapolated above one (Liu transmission below its fitted Stokes range) used to veto every full-distribution bicycle candidate, although they move the PM10 ratio by about 5e-4. Results now report `SizeResult.nonphysical_transport_excess` and `PmResult.nonphysical_excess`; candidates report `nonphysical_pm10_excess`. A study may declare `constraints.max_nonphysical_pm10_excess`; without it the strict rejection is unchanged, and negative efficiencies are always rejected. Nothing is clipped. `W_NONPHYSICAL` now carries the value and formula. The bicycle study declares 0.001 and now finds accepted full-distribution designs; a strict variant keeps the previous outcome as evidence.
- **Search robustness.** A failed cold recheck no longer aborts the whole search; the candidate is kept, infeasible, with the error. A best candidate whose recheck was cut off by the deadline or cancellation is returned with `cold_rechecked: false` instead of being dropped, and the status requires a completed recheck. The worker reports checks only for rechecked candidates.
- **Validation.** A bend whose centerline radius does not exceed the tube radius is rejected with `E_UNCOVERED`.
- **Python.** `check` returns the same report as the CLI; the integration check asserts it.
- **Tooling.** Strict Clippy passes on current stable (needless range loop). The virtual-inlet comparison attributes the computed loss to mechanisms and states the flow scaling; its anchors still fail (4 of 6), unchanged.
- **Documentation.** Restored missing spaces between words and numbers throughout the authored and generated documentation; updated the design-search guide, how-to, registry and the generated study report.
