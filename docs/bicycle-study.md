# Bicycle design study: executed results

Date 2026-11-15. Source defaults and random seeds are retained in examples/ and results/. These are computed correlation outputs, not field measurements.

## Baseline at 2 L/min

| Speed, km/h | Crosswind, m/s | PM2.5 relative bias | PM10 relative bias | Mean position offset, m | Resolution, m |
|---:|---:|---:|---:|---:|---:|
| 10.0 | 0.0 | -0.06% | -19.32% | 6.02 | 7.86 |
| 10.0 | 2.0 | -0.35% | -27.30% | 6.02 | 7.86 |
| 15.0 | 0.0 | +0.26% | -16.59% | 9.03 | 11.79 |
| 15.0 | 2.0 | -0.02% | -17.71% | 9.03 | 11.79 |
| 20.0 | 0.0 | +0.69% | -13.25% | 12.05 | 15.73 |
| 20.0 | 2.0 | +0.19% | -11.39% | 12.05 | 15.73 |
| 25.0 | 0.0 | +1.20% | -9.23% | 15.06 | 19.66 |
| 25.0 | 2.0 | +0.38% | -5.37% | 15.06 | 19.66 |

## Budgeted search

Goal: PM10 within 15% at every declared speed and crosswind, resolution below 15 m at 18 km/h, total flow at most 3 L/min, laminar flow. Transport efficiencies that an extrapolated correlation pushes above one are accepted only within the declared allowance `max_nonphysical_pm10_excess = 0.001` (the share of the PM10 ratio that rests on them); negative efficiencies are always rejected. The `strict-nonphysical-guard` row repeats seed 42 without the allowance. The all-speeds maximum resolution is also kept in each raw candidate.

| Study | Budget used | Status | Worst PM10 bias | Resolution at 18 km/h, m | Flow, L/min | Nonphysical PM10 excess | Pareto points |
|---|---:|---|---:|---:|---:|---:|---:|
| full-distribution-seed42 | 512 | feasible_design_found | 0.14433 | 11.806 | 3.000 | 5.25e-04 | 9 |
| full-distribution-seed7 | 512 | feasible_design_found | 0.14463 | 12.114 | 3.000 | 5.29e-04 | 8 |
| full-distribution-seed123 | 512 | feasible_design_found | 0.14430 | 11.977 | 2.998 | 5.25e-04 | 9 |
| mode-full-distribution | 256 | no_feasible_design_found | 0.15234 | 14.550 | 2.855 | 5.74e-04 | 0 |
| strict-nonphysical-guard | 512 | no_feasible_design_found | 0.14594 | 11.808 | 3.000 | 5.46e-04 | 0 |
| coarse-monodisperse-sensitivity | 512 | feasible_design_found | 0.09923 | 13.566 | 3.000 | 0.00e+00 | 20 |
| mode-coarse-monodisperse-sensitivity | 256 | feasible_design_found | 0.10044 | 12.191 | 2.939 | 0.00e+00 | 25 |
| deliberately-impossible-zero-resolution | 64 | no_feasible_design_found | 0.10954 | 15.183 | 2.871 | 0.00e+00 | 0 |

### Best full-distribution designs

| Study | Inlet, mm | Flow, L/min | Tube, cm | Bend radius, cm | Buffer, mL |
|---|---:|---:|---:|---:|---:|
| full-distribution-seed42 | 5.62 | 3 | 3 | 1.01 | 1 |
| full-distribution-seed7 | 5.64 | 3 | 4.18 | 2.89 | 10.3 |
| full-distribution-seed123 | 5.62 | 3 | 3 | 1 | 7.75 |

All full-distribution seeds put flow at its upper bound. Active bounds mark where the declared requirements push; widening a bound, for example a higher flow limit, is a design decision for the owner, not something the search should do silently.

## Interpretation

For sub-micron particles in the facing, sub-isokinetic scenarios, the Liu transmission correlation is evaluated below its fitted Stokes range and returns values slightly above one. The evaluator keeps those values unclipped and reports, per size and per PM cut, how much of the result rests on them (`nonphysical_transport_excess`, `nonphysical_excess`). For the bicycle study that share is of order 1e-4 to 1e-3 of the PM10 mass, two orders below the 15% tolerance, so the study accepts it explicitly. Without the allowance (`strict-nonphysical-guard`) every full-distribution candidate is rejected, as in the 0.1.0 candidate. Neither outcome says anything about built hardware.

The coarse monodisperse sensitivity tests a different declared aerosol; it demonstrates search behaviour and conditional tradeoffs only, and is not a recommended bike design. The deliberately impossible zero-resolution goal tests truthful finite-search failure reporting.

### Reported best-candidate violations

- **full-distribution-seed42**: no declared constraint violation; scientific coverage warnings remain.
- **full-distribution-seed7**: no declared constraint violation; scientific coverage warnings remain.
- **full-distribution-seed123**: no declared constraint violation; scientific coverage warnings remain.
- **mode-full-distribution**: PM10 relative deviation: 1.523399e-1 > 1.500000e-1
- **strict-nonphysical-guard**: nonphysical extrapolation cannot be accepted by search without constraints.max_nonphysical_pm10_excess
- **coarse-monodisperse-sensitivity**: no declared constraint violation; scientific coverage warnings remain.
- **mode-coarse-monodisperse-sensitivity**: no declared constraint violation; scientific coverage warnings remain.
- **deliberately-impossible-zero-resolution**: distance resolution m: 1.518254e1 > 0.000000e0

G2 practitioner acceptance remains pending. Raw cases and failed rows are preserved in results/; reference and oracle evidence are distinct from optimization.
