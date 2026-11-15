# Validation protocol and open gates

**G0 incomplete:** original-source and source-published-number coverage is incomplete. Most formulas were checked against Brockmann's standard-reference notes and the published PLC compilation. Numerical arithmetic fixtures are labeled as arithmetic; approximate published bend guidance is labeled as adjacent geometry with an a priori absolute tolerance. No constants were tuned to obtain agreement.

**G1 incomplete:** Rust regression/property checks, the aerosolpy grid and the approximate PLC virtual regression are numerical verification. AeroSampleR runtime and an independent IGOR Pro execution remain pending. They cannot be replaced by re-evaluating the same formulas in another language.

**G2 conditional study produced, practitioner acceptance pending:** the bicycle campaign records successes/failures under explicit aerosol distributions, source extrapolation guards and resource budgets. A practitioner must assess whether the model covers the proposed inlet before hardware selection.

**G3 not attempted:** no on-bike data, laboratory flow data or particle-tracking CFD was supplied or generated. No experimental claim is made.

## V1 code-to-code

Freeze cases before opening external results: forward-facing probe at three speed ratios; yaw30/60/90deg; calm horizontal/vertical; tube at Re1000/8000;90deg bend at three Stokes numbers; a contraction. Use the source's exact angle, diameter and Stokes conventions. Export all PLC input values with outputs and software version. Declare per-case numerical tolerances before comparison. Explain formula choices rather than fit constants. The optional AeroSampleR exporter records a uniform-diameter tube/bend case and must be checked under an installed R package before claiming agreement.

The virtual PLC example is qualitative because its air state, bend settling orientation and taper-wall loss details are not fully specified. The primary contraction equation also differs from the PLC printed equation. The comparison script records every approximate anchor and uses a predeclared2percentage-point absolute tolerance; a mismatch stays failed. This limited comparison cannot close G1.

## V2 experiment to be registered before data

Build a defined conductive thin-walled inlet and measure actual total/sensor flow, including under wind. Freeze design, tubing, aerosol size distribution assumptions and reference-instrument settings. Co-locate on the same bike at0,10,15,20,25km/h, with repeated randomized route segments and at least three repeats per condition; log wind, humidity, temperature, speed and timestamps. First perform filter-zero, flow and step tests.

Primary quantity: ratio of size-resolved concentration, or properly defined coarse/fine mass ratio, against a reference of known inlet response. Separate inlet effects from optical humidity response. Register a tolerance such as absolute relative speed-dependence error≤.15 for the declared coarse fraction, with instrument uncertainty and interval method specified **before** opening measurements. The illustrative.15 is a draft protocol decision, not an agreed experimental acceptance criterion. Keep raw data and rejected runs. Analyze disagreements against omitted fan/housing/wake/buffer physics.

Only measured results can close G3. TrackV3 body-fitted flow/particle tracking would be a separate numerical check, not a substitute for measurement.
