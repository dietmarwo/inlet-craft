# Independent implementation review

Date: 2026-11-14. Reviewer: a separate agent that did not write the solver, bindings, worker, tests, or guides. This review covers implementation increments I0–I7; it is not practitioner acceptance or experimental validation.

## Disposition

The project is a useful **provisional software candidate** with unusually explicit evidence limitations. It must not be described as an experimentally validated inlet designer or as having passed all of the plan's scientific gates. The current README, guides, result metadata and validation protocol make that distinction correctly.

The core equations and conventions were reviewed against the published Particle Loss Calculator paper and Brockmann's Sandia tutorial slides and written notes. No PLC program code was consulted. The original Allen–Raabe DOI was independently verified against its publisher. Most original empirical papers and precise published numerical anchors remain outstanding, as the formula registry states.

## Increment review

| Increment | Independent review and disposition |
|---|---|
| I0 | Reviewed source provenance, license boundaries, equation registry, source conflicts and explicit incomplete G0 declaration. Original-source/published-point coverage remains incomplete. |
| I1 | Inspected SI primitives and slip-corrected aerodynamic inversion. Density is not applied twice. Diameter/shape/density roundtrip and dimensional tests pass. Allen–Raabe coefficients are correctly attributed to solid spheres; the mean-free-path convention difference is declared. |
| I2 | Inspected moving/calm Stokes definitions, angle branches, probe transmission, source jumps, horizontal-wind geometry and slow interpolation. The vertical-inlet angle and floating-point endpoint regressions are tested. The vector generalization is explicitly identified as an assumption. |
| I3 | Inspected mechanism multiplication, tube regime handling, bend/contraction conventions and concentration/count-flux split accounting. Buffer wall loss, bend settling and taper transport omissions are reported rather than silently modeled. |
| I4 | Reviewed strict nested serde parsing, versioned schemas, CLI error/verification behavior, native Python bridge and failed-row preservation. Generated interface and integration checks supplement the core tests. |
| I5 | Inspected distribution mass weighting, ideal aerodynamic cuts, apparent-wind geometry, mean age, averaging and response quantiles. Independently checked a three-tank unequal-pole response against SciPy. Stiff/expensive chains have explicit numerical support limits. |
| I6 | Inspected actual DE/MODE integration, aggregate constraints, linked geometry, exact optimizer evaluation budgets, cancellation-aware cold checks and finite-search status claims. The 18 km/h resolution reference is separate from the reported maximum across speeds. |
| I7 | Reviewed README, five guide chapters, architecture, comparison, release preparation and validation protocol. They distinguish software verification, partial external comparisons and future physical validation. |

## Findings corrected during review

- The primary Brockmann contraction expression contains a factor of two relative to the printed PLC expression. The implementation keeps that source choice, upstream velocity/downstream diameter, and an explicit source-conflict diagnostic. The fitted coefficient `3.14` is retained as published; it is not replaced with pi.
- Calm-air Stokes number uses inlet velocity, while moving-air aspiration uses freestream velocity. Bend angle uses radians internally and the selected standard-reference convention uses tube diameter.
- A nonfinite general-chain CDF previously could produce a finite, false quantile through the bisection comparison. Quantiles now propagate the failure, and response preflight bounds reject expensive/stiff cases before solving.
- Optimization now uses MODE for multiple objectives, propagates contraction diameters, checks cancellation before baseline/cold work, and drops points that cannot be cold-checked before termination.
- Distance at 18 km/h is explicitly selectable instead of inadvertently imposing the same distance requirement at 25 km/h.
- Probe tilt and yaw now use a vector dot product for effective horizontal-wind misalignment. A small stated endpoint tolerance avoids rejecting a nominal 60-degree angle solely because of floating-point roundoff.

## Independently executed checks

The reviewer ran `cargo test --offline --locked -p inlet-craft`: **31 tests passed**, including strict parsing, reference-row handling, limiting behavior, diameter conversion, transport bounds/monotonicity, flow transitions, split conservation, PM convergence, geometry, response, deterministic search, MODE, cold checks and cancellation. The final rerun includes rejection of an unsupported downward-facing aperture in both calm and slow air.

An additional independent check used three well-mixed buffers with time constants 0.3, 0.7 and 1.5 seconds, without plug delay, sensor lag or averaging. The CLI quantiles were compared with `scipy.linalg.expm` for an absorbing continuous-time Markov chain and a root solve. A priori absolute tolerance was 1e-10 seconds:

| Quantity | Rust CLI | Independent SciPy | Absolute error |
|---|---:|---:|---:|
| t10, seconds | 0.7986830031575964 | 0.7986830031577664 | 1.70e-13 |
| t90, seconds | 4.704048480781098 | 4.704048480781246 | 1.48e-13 |

The recorded aerosolpy report was inspected: all 150 rows passed its declared tolerances. Maximum relative slip and diffusion differences are approximately 1.318% and 1.065%; maximum absolute penetration difference is 0.003918. This is partial numerical comparison, not validation of aspiration, bends, contractions or a built inlet.

The executed study campaign was inspected. Three full-distribution DE searches found raw integrated PM10 deviations around 14.4–14.6% and resolution below 15 m at 18 km/h, but all were rejected by the small-Stokes nonphysical extrapolation guard. The coarse-monodisperse sensitivity found a conditionally accepted point with 9.92% worst PM10 deviation, 13.57 m reference-speed resolution and 3 L/min flow. The zero-resolution case was correctly unsuccessful. These are distribution-dependent computational outcomes, not evidence of hardware performance.

The reconstructed PLC virtual system has a **material unresolved mismatch**: only two of six approximate anchors meet the predeclared two-percentage-point absolute tolerance. At 40 L/min, predicted losses around 2–3% differ from roughly 5–6% in the paper; at 150 L/min, about 5–6% differs from roughly 40%. The reviewer independently inspected the printed turbulent expression and its Reynolds exponent; no missing printed exponent was found. The standard-reference dimensionless-to-dimensional conversion agrees with the implementation. The original Lee–Gieseke full text was not available, and the final 10 mm section at 150 L/min also exceeds the reported Re<15600 fit range. The cause has not been established and must not be attributed solely to unspecified geometry or air state. All failed anchors remain evidence. No coefficient was adjusted to obtain agreement.

The reviewer also independently ran `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`: **passed**. The empirical contraction coefficient is preserved under a documented, narrowly scoped lint exception. The implementation author records the final complete workspace/integration/docs checks in the implementation log.

To avoid validating MODE only on an empty accepted archive, the reviewer independently ran a 128-evaluation, seed-42 MODE study using the example constraints and a monodisperse 5-micrometer aerosol. It returned eight feasible, cold-checked Pareto points. Every returned pair was checked for strict dominance; none was dominated. Their worst-case PM10 deviations ranged from approximately 10.20% to 13.50%. This verifies nonempty multi-objective operation for that hypothetical aerosol, not general design suitability.

## Accepted limitations and remaining gates

The full-distribution bicycle search is particularly sensitive to a genuine source-coverage issue: Liu transmission extrapolated below its fitted Stokes range can exceed one slightly. Evaluation preserves and diagnoses that result; design search rejects it. A failed strict search therefore cannot establish that the hardware goals are physically unattainable. PM bias, resolution and this extrapolation barrier must be reported separately. The guides do so.

The finite-search result retains the first 256 failed candidate rows and an omitted count, rather than retaining arbitrarily many failures in memory. The Pareto archive is also bounded. This is a documented departure from the plan's unrestricted failed-row retention; individual fixture verification retains every row.

G0 and G1 remain incomplete. The optional AeroSampleR runtime comparison and an external IGOR Pro run were not performed. Approximate published bend guidance and the partial virtual-inlet reconstruction cannot substitute for precise original numerical evidence. G2 has computed studies but still needs an aerosol practitioner's assessment. G3 requires actual on-bike measurements under a protocol fixed before opening the data. No release, optimality proof, physical infeasibility proof or experimental agreement is established by this review.

## Source links

- [Published PLC paper](https://doi.org/10.5194/amt-2-479-2009)
- [Brockmann Sandia slides, SAND2010-7445P](https://www.osti.gov/servlets/purl/1675150)
- [Brockmann Sandia written notes, SAND2010-7446P](https://www.osti.gov/servlets/purl/1675151)
- [Original Allen–Raabe solid-sphere slip measurements](https://doi.org/10.1080/02786828508959055)
- [Original Muyshondt contraction paper](https://doi.org/10.1080/02786829608965364)
