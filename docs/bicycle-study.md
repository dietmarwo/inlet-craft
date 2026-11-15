# Bicycle design study: executed results

Date 2026-11-15. Source defaults and random seeds are retained in examples/ and results/. These are computed correlation outputs, not field measurements.

## Baseline at 2 L/min

| Speed, km/h | Crosswind, m/s | PM2.5 relative bias | PM10 relative bias | Mean position offset, m | Resolution, m |
|---:|---:|---:|---:|---:|---:|
| 10.0 | 0.0 | -0.06% | -19.32% | 6.02 | 7.86 |
| 10.0 | 2.0 | -0.07% | -20.49% | 6.02 | 7.86 |
| 15.0 | 0.0 | +0.26% | -16.59% | 9.03 | 11.79 |
| 15.0 | 2.0 | +0.05% | -15.84% | 9.03 | 11.79 |
| 20.0 | 0.0 | +0.69% | -13.25% | 12.05 | 15.73 |
| 20.0 | 2.0 | +0.20% | -10.93% | 12.05 | 15.73 |
| 25.0 | 0.0 | +1.20% | -9.23% | 15.06 | 19.66 |
| 25.0 | 2.0 | +0.38% | -5.29% | 15.06 | 19.66 |

## Budgeted search

Goal: PM10 within 15% at every declared speed and crosswind, resolution below 15 m at 18 km/h, total flow at most 3 L/min, laminar flow. Transport efficiencies that an extrapolated correlation pushes above one are accepted only within the declared allowance `max_nonphysical_pm10_excess = 0.001` (the share of the PM10 ratio that rests on them); negative efficiencies are always rejected. The `strict-nonphysical-guard` row repeats seed 42 without the allowance. The all-speeds maximum resolution is also kept in each raw candidate.

| Study | Budget used | Status | Worst PM10 bias | Resolution at 18 km/h, m | Flow, L/min | Nonphysical PM10 excess | Pareto points |
|---|---:|---|---:|---:|---:|---:|---:|
| full-distribution-seed42 | 512 | feasible_design_found | 0.08782 | 12.773 | 3.000 | 4.47e-04 | 29 |
| full-distribution-seed7 | 512 | feasible_design_found | 0.08817 | 11.806 | 3.000 | 4.49e-04 | 87 |
| full-distribution-seed123 | 512 | feasible_design_found | 0.08669 | 13.489 | 3.000 | 4.24e-04 | 15 |
| mode-full-distribution | 256 | feasible_design_found | 0.10175 | 13.067 | 2.834 | 4.28e-04 | 14 |
| strict-nonphysical-guard | 512 | no_feasible_design_found | 0.09039 | 13.190 | 3.000 | 4.67e-04 | 0 |
| coarse-monodisperse-sensitivity | 512 | feasible_design_found | 0.05880 | 14.434 | 3.000 | 0.00e+00 | 8 |
| mode-coarse-monodisperse-sensitivity | 256 | feasible_design_found | 0.06199 | 11.821 | 2.888 | 0.00e+00 | 11 |
| deliberately-impossible-zero-resolution | 64 | no_feasible_design_found | 0.11045 | 15.183 | 2.871 | 0.00e+00 | 0 |

### Best full-distribution designs

| Study | Inlet, mm | Flow, L/min | Tube, cm | Bend radius, cm | Buffer, mL |
|---|---:|---:|---:|---:|---:|
| full-distribution-seed42 | 5.36 | 3 | 13 | 2.59 | 18.5 |
| full-distribution-seed7 | 5.37 | 3 | 13.6 | 2.22 | 1 |
| full-distribution-seed123 | 5.28 | 3 | 3 | 1 | 24.9 |

All full-distribution seeds put flow at its upper bound. Active bounds mark where the declared requirements push; widening a bound, for example a higher flow limit, is a design decision for the owner, not something the search should do silently.

## Interpretation

For sub-micron particles in the facing, sub-isokinetic scenarios, the Liu transmission correlation is evaluated below its fitted Stokes range and returns values slightly above one. The evaluator keeps those values unclipped and reports, per size and per PM cut, how much of the result rests on them (`nonphysical_transport_excess`, `nonphysical_excess`). For the bicycle study that share is of order 1e-4 to 1e-3 of the PM10 mass, two orders below the 15% tolerance, so the study accepts it explicitly. Without the allowance (`strict-nonphysical-guard`) every full-distribution candidate is rejected, as in the 0.1.0 candidate. Neither outcome says anything about built hardware.

The coarse monodisperse sensitivity tests a different declared aerosol; it demonstrates search behaviour and conditional tradeoffs only, and is not a recommended bike design. The deliberately impossible zero-resolution goal tests truthful finite-search failure reporting.

### Reported best-candidate violations

- **full-distribution-seed42**: no declared constraint violation; scientific coverage warnings remain.
- **full-distribution-seed7**: no declared constraint violation; scientific coverage warnings remain.
- **full-distribution-seed123**: no declared constraint violation; scientific coverage warnings remain.
- **mode-full-distribution**: no declared constraint violation; scientific coverage warnings remain.
- **strict-nonphysical-guard**: nonphysical extrapolation cannot be accepted by search without constraints.max_nonphysical_pm10_excess
- **coarse-monodisperse-sensitivity**: no declared constraint violation; scientific coverage warnings remain.
- **mode-coarse-monodisperse-sensitivity**: no declared constraint violation; scientific coverage warnings remain.
- **deliberately-impossible-zero-resolution**: distance resolution m: 1.518254e1 > 0.000000e0

G2 practitioner acceptance remains pending. Raw cases and failed rows are preserved in results/; reference and oracle evidence are distinct from optimization.
