#!/usr/bin/env python3
"""Bounded reproducible design campaign and generated numerical guide."""
import copy
import datetime
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / 'target/release/inlet-craft'
DATE = datetime.date.today().isoformat()
DESIGN_COLUMNS = [('/elements/0/inner_diameter_m', 'Inlet, mm', 1e3), ('/flow/total_m3_s', 'Flow, L/min', 6e4),
                  ('/elements/1/length_m', 'Tube, cm', 1e2), ('/elements/2/radius_m', 'Bend radius, cm', 1e2),
                  ('/elements/3/volume_m3', 'Buffer, mL', 1e6)]


def run(command, value, timeout=90):
    with tempfile.NamedTemporaryFile('w', suffix='.json') as f:
        json.dump(value, f)
        f.flush()
        p = subprocess.run([str(CLI), command, f.name], capture_output=True, text=True, timeout=timeout)
    if p.returncode:
        raise RuntimeError(p.stderr)
    return json.loads(p.stdout)


def variants(study):
    out = []
    for seed in [42, 7, 123]:
        s = copy.deepcopy(study)
        s.update(seed=seed, max_evaluations=512, wall_time_seconds=45.)
        out.append((f'full-distribution-seed{seed}', s))
    s = copy.deepcopy(study)
    s.update(seed=42, max_evaluations=256, multi_objective=True, wall_time_seconds=45.)
    out.append(('mode-full-distribution', s))
    # Same as seed 42 without the explicit allowance: the 0.1.0 behaviour.
    s = copy.deepcopy(study)
    s['constraints'].pop('max_nonphysical_pm10_excess', None)
    s.update(seed=42, max_evaluations=512, wall_time_seconds=45.)
    out.append(('strict-nonphysical-guard', s))
    s = copy.deepcopy(study)
    s['case']['particles'] = {'aerodynamic_diameter_m': [5e-6]}
    s['case']['distribution']['modes'] = [dict(number_concentration_m3=1e6, geometric_mean_diameter_m=5e-6,
                                               geometric_std_dev=1.)]
    s.update(max_evaluations=512, wall_time_seconds=45.)
    out.append(('coarse-monodisperse-sensitivity', s))
    coarse_mode = copy.deepcopy(s)
    coarse_mode.update(multi_objective=True, max_evaluations=256)
    out.append(('mode-coarse-monodisperse-sensitivity', coarse_mode))
    s = copy.deepcopy(s)
    s['constraints']['max_distance_resolution_m'] = 0.
    s['max_evaluations'] = 64
    out.append(('deliberately-impossible-zero-resolution', s))
    return out


def main():
    study = json.loads((ROOT / 'examples/bicycle-study.json').read_text())
    allowance = study['constraints'].get('max_nonphysical_pm10_excess')
    baseline = run('evaluate', study['case'])
    (ROOT / 'results/bicycle-baseline.json').write_text(json.dumps(baseline, indent=2) + '\n')
    rows = []
    for label, s in variants(study):
        result = run('optimize', s, timeout=60)
        (ROOT / f'results/{label}.json').write_text(json.dumps(result, indent=2) + '\n')
        b = result['best']
        rows.append(dict(
            study=label, status=result['status'], evaluations=result['evaluations'],
            failed_evaluations=result['failed_evaluations'], elapsed_seconds=result['elapsed_seconds'],
            worst_pm10_bias=b and b['maximum_pm10_relative_deviation'],
            resolution_at_reference_m=b and b['distance_resolution_objective_m'],
            flow_l_min=b and b['total_flow_l_min'], warning_count=b and b['warning_count'],
            nonphysical_pm10_excess=b and b['nonphysical_pm10_excess'],
            design=b and {path: x for (path, _, _), x in zip(DESIGN_COLUMNS, b['x'])} if len(s['variables']) == 5 else None,
            violations=b['violations'] if b else [], pareto_points=len(result['pareto'])))
        if b:
            (ROOT / f'results/{label}-best-case.json').write_text(json.dumps(b['case'], indent=2) + '\n')
    summary = dict(schema='inlet-craft.study-campaign/0.1', date=DATE,
                   distribution='explicit example assumptions, not measured site data',
                   max_nonphysical_pm10_excess=allowance, rows=rows, experimental_validation=False)
    (ROOT / 'results/study-campaign.json').write_text(json.dumps(summary, indent=2) + '\n')

    out = ['# Bicycle design study: executed results', '',
           f'Date {DATE}. Source defaults and random seeds are retained in examples/ and results/. These are computed '
           'correlation outputs, not field measurements.', '',
           '## Baseline at 2 L/min', '',
           '| Speed, km/h | Crosswind, m/s | PM2.5 relative bias | PM10 relative bias | Mean position offset, m | Resolution, m |',
           '|---:|---:|---:|---:|---:|---:|']
    for s in baseline['scenarios']:
        p = s['pm']
        out.append(f"| {s['conditions']['riding_speed_m_s'] * 3.6:.1f} | {s['conditions']['crosswind_m_s']:.1f} | "
                   f"{100 * (p[0]['efficiency'] - 1):+.2f}% | {100 * (p[1]['efficiency'] - 1):+.2f}% | "
                   f"{s['mean_position_offset_m']:.2f} | {s['distance_resolution_10_90_m']:.2f} |")
    out += ['', '## Budgeted search', '',
            'Goal: PM10 within 15% at every declared speed and crosswind, resolution below 15 m at 18 km/h, total '
            'flow at most 3 L/min, laminar flow. Transport efficiencies that an extrapolated correlation pushes '
            f'above one are accepted only within the declared allowance `max_nonphysical_pm10_excess = {allowance}` '
            '(the share of the PM10 ratio that rests on them); negative efficiencies are always rejected. The '
            '`strict-nonphysical-guard` row repeats seed 42 without the allowance. The all-speeds maximum '
            'resolution is also kept in each raw candidate.', '',
            '| Study | Budget used | Status | Worst PM10 bias | Resolution at 18 km/h, m | Flow, L/min | Nonphysical PM10 excess | Pareto points |',
            '|---|---:|---|---:|---:|---:|---:|---:|']
    for row in rows:
        out.append(f"| {row['study']} | {row['evaluations']} | {row['status']} | {row['worst_pm10_bias']:.5f} | "
                   f"{row['resolution_at_reference_m']:.3f} | {row['flow_l_min']:.3f} | "
                   f"{row['nonphysical_pm10_excess']:.2e} | {row['pareto_points']} |")
    designs = [r for r in rows if r['design'] and r['study'].startswith('full-distribution')]
    out += ['', '### Best full-distribution designs', '',
            '| Study | ' + ' | '.join(name for _, name, _ in DESIGN_COLUMNS) + ' |',
            '|---|' + '---:|' * len(DESIGN_COLUMNS)]
    for r in designs:
        out.append(f"| {r['study']} | " + ' | '.join(f"{r['design'][p] * k:.3g}" for p, _, k in DESIGN_COLUMNS) + ' |')
    bounds = {v['path']: (v['lower'], v['upper']) for v in study['variables']}
    at_bound = []
    for p, name, _ in DESIGN_COLUMNS:
        lo, hi = bounds[p]
        if all(abs(r['design'][p] - hi) <= 1e-3 * hi for r in designs):
            at_bound.append(f'{name.split(",")[0].lower()} at its upper bound')
        elif all(abs(r['design'][p] - lo) <= 1e-3 * hi for r in designs):
            at_bound.append(f'{name.split(",")[0].lower()} at its lower bound')
    if at_bound:
        out += ['', 'All full-distribution seeds put ' + ', '.join(at_bound) + '. Active bounds mark where the '
                'declared requirements push; widening a bound, for example a higher flow limit, is a design decision '
                'for the owner, not something the search should do silently.']
    out += ['', '## Interpretation', '',
            'For sub-micron particles in the facing, sub-isokinetic scenarios, the Liu transmission correlation '
            'is evaluated below its fitted Stokes range and returns values slightly above one. The evaluator keeps '
            'those values unclipped and reports, per size and per PM cut, how much of the result rests on them '
            '(`nonphysical_transport_excess`, `nonphysical_excess`). For the bicycle study that share is of order '
            '1e-4 to 1e-3 of the PM10 mass, two orders below the 15% tolerance, so the study accepts it explicitly. '
            'Without the allowance (`strict-nonphysical-guard`) every full-distribution candidate is rejected, as '
            'in the 0.1.0 candidate. Neither outcome says anything about built hardware.', '',
            'The coarse monodisperse sensitivity tests a different declared aerosol; it demonstrates search '
            'behaviour and conditional tradeoffs only, and is not a recommended bike design. The deliberately '
            'impossible zero-resolution goal tests truthful finite-search failure reporting.', '',
            '### Reported best-candidate violations', '']
    for row in rows:
        out.append(f"- **{row['study']}**: " + ('; '.join(row['violations']) if row['violations']
                                               else 'no declared constraint violation; scientific coverage warnings remain.'))
    out += ['', 'G2 practitioner acceptance remains pending. Raw cases and failed rows are preserved in results/; '
            'reference and oracle evidence are distinct from optimization.']
    (ROOT / 'docs/bicycle-study.md').write_text('\n'.join(out) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
