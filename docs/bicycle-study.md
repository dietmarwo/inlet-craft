# Bicycle design study: executed results

Date2026-11-14. Source defaults and random seeds are retained in examples/results. These are computed correlation outputs, not field measurements.

## Baseline at2L/min

| Speedkm/h | Crosswindm/s | PM2.5relativebias | PM10relativebias | Meanpositionoffsetm | Resolutionm |
|---:|---:|---:|---:|---:|---:|
|10.0|0.0|-0.06%|-19.32%|6.02|7.86|
|10.0|2.0|-0.35%|-27.30%|6.02|7.86|
|15.0|0.0|+0.26%|-16.59%|9.03|11.79|
|15.0|2.0|-0.02%|-17.71%|9.03|11.79|
|20.0|0.0|+0.69%|-13.25%|12.05|15.73|
|20.0|2.0|+0.19%|-11.39%|12.05|15.73|
|25.0|0.0|+1.20%|-9.23%|15.06|19.66|
|25.0|2.0|+0.38%|-5.37%|15.06|19.66|

## Budgeted search

Resolution goal15m at18km/h, PM10within15%, totalflow≤3L/min, laminar flow. All nonphysical extrapolations are hard rejected. The all-speeds maximum resolution is also retained in each raw candidate.

| Study | Budgetused | Accepted | PM10worstbias | Resolutionat18km/h | FlowL/min | Paretopoints |
|---|---:|---|---:|---:|---:|---:|
|full-distribution-seed42|512|no_feasible_design_found|0.14594|11.808|3.000|0|
|full-distribution-seed7|512|no_feasible_design_found|0.14538|12.147|3.000|0|
|full-distribution-seed123|512|no_feasible_design_found|0.14423|14.712|3.000|0|
|mode-full-distribution|256|no_feasible_design_found|0.15319|13.139|2.828|0|
|coarse-monodisperse-sensitivity|512|feasible_design_found|0.09923|13.566|3.000|20|
|mode-coarse-monodisperse-sensitivity|256|feasible_design_found|0.10044|12.191|2.939|25|
|deliberately-impossible-zero-resolution|64|no_feasible_design_found|0.10954|15.183|2.871|0|

## Interpretation

The full-lognormal study is limited by a source/extrapolation issue: for sufficiently small Stokes numbers and sub-isokinetic conditions, Liu transmission slightly exceeds1. This lies outside the source fitting range. Even very low-mass fine tails trigger the explicit nonphysical guard. The integrated PM10bias and resolution alone do not settle feasibility. These failed searches therefore give no evidence that the hardware goal is impossible. No constants or outputs were clipped to make them pass.

The coarse monodisperse sensitivity removes that fine-tail issue and tests a different declared aerosol. A feasible result there demonstrates search functionality and conditional tradeoffs only; it is not a recommended bike design or a validation for realistic multimodal aerosol. The deliberately impossible zero-resolution goal tests truthful finite-search failure reporting.

### Reported best-candidate violations

- **full-distribution-seed42**: nonphysical extrapolation cannot be accepted by search
- **full-distribution-seed7**: nonphysical extrapolation cannot be accepted by search
- **full-distribution-seed123**: nonphysical extrapolation cannot be accepted by search
- **mode-full-distribution**: PM10 relative deviation: 1.531902e-1 > 1.500000e-1; nonphysical extrapolation cannot be accepted by search
- **coarse-monodisperse-sensitivity**: no declared constraint violation; scientific coverage warnings remain.
- **mode-coarse-monodisperse-sensitivity**: no declared constraint violation; scientific coverage warnings remain.
- **deliberately-impossible-zero-resolution**: distance resolution m: 1.518254e1 > 0.000000e0

G2practitioner acceptance remains pending. Raw cases and failed rows are preserved in results; reference and oracle evidence are distinct from optimization.
