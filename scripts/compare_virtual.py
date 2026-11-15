#!/usr/bin/env python3
"""Published qualitative system regression with explicit missing-geometry limits.

Default exits 1 on mismatch. --report-only retains failures without claiming G1.
Tolerance: 2 percentage points absolute at approximate 0.1/0.3 um graph anchors,
fixed before running; no fitted constants. No PLC program code is consulted.
Each row also attributes the computed loss to mechanisms (share of -ln T), so a
later code-to-code comparison (track V1) can see where the difference sits.
"""
import copy
import json
import math
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / 'target/release/inlet-craft'
EXPECTED = {10: [.01, .0032], 40: [.06, .05], 150: [.40, .40]}
TOLERANCE = .02


def breakdown(size):
    """Share of the computed log-loss carried by each mechanism formula."""
    shares = defaultdict(float)
    shares['aspiration'] = -math.log(size['aspiration_efficiency'])
    for element in size['elements']:
        mechanisms = element['mechanisms'][1:] if element['kind'] == 'probe' else element['mechanisms']
        for m in mechanisms:
            shares[m['formula']] += -math.log(m['efficiency'])
    total = sum(shares.values())
    return {k: v / total for k, v in sorted(shares.items(), key=lambda kv: -kv[1]) if abs(v / total) >= .01}


def main():
    base = json.loads((ROOT / 'examples/plc-virtual-inlet.json').read_text())
    rows = []
    for flow in EXPECTED:
        case = copy.deepcopy(base)
        case['flow']['total_m3_s'] = flow / 60000
        with tempfile.NamedTemporaryFile('w', suffix='.json') as f:
            json.dump(case, f)
            f.flush()
            p = subprocess.run([str(CLI), 'evaluate', f.name], text=True, capture_output=True, timeout=20)
        if p.returncode:
            raise RuntimeError(p.stderr)
        r = json.loads(p.stdout)
        (ROOT / f'results/plc-virtual-{flow}lpm.json').write_text(json.dumps(r, indent=2) + '\n')
        for i, diam in enumerate([.1, .3]):
            size = r['scenarios'][0]['sizes'][i]
            actual = 1 - size['overall_efficiency']
            target = EXPECTED[flow][i]
            rows.append(dict(flow_l_min=flow, diameter_um=diam, published_approximate_loss=target,
                             actual_loss=actual, absolute_tolerance=TOLERANCE,
                             passed=abs(actual - target) <= TOLERANCE,
                             computed_loss_share_by_mechanism=breakdown(size)))
    # Flow scaling of log-loss between 40 and 150 L/min, computed vs anchors.
    def exponent(key, diam):
        a, b = (next(r for r in rows if r['flow_l_min'] == q and r['diameter_um'] == diam) for q in (40, 150))
        return math.log(math.log(1 - b[key]) / math.log(1 - a[key])) / math.log(150 / 40)
    scaling = {f'{d}um': dict(computed=exponent('actual_loss', d), published=exponent('published_approximate_loss', d))
               for d in (.1, .3)}
    report = dict(
        schema='inlet-craft.published-system-comparison/0.1', log_loss_flow_exponent_40_to_150=scaling,
        source='von der Weiden 2009 Figure 7 p492', source_url='https://doi.org/10.5194/amt-2-479-2009',
        evidence_kind='published_approximate_curve_with_incomplete_geometry',
        all_passed=all(r['passed'] for r in rows), rows=rows,
        limitations=['air state unspecified, assumed 293.15 K / 101325 Pa',
                     'bend arc diffusion/impaction included, arc settling orientation unspecified',
                     'taper wall losses unmodeled; conical volume only',
                     'primary Brockmann contraction factor 2 differs from PLC printed Eq 32',
                     '150 L/min: 10 mm tube Re about 21000 exceeds the Lee-Gieseke fitted range 4000-15600',
                     'orifice diameter not stated: Figure 7 labels the inlet super-isokinetic, which the transcribed 50 mm probe cannot give at these flows',
                     'possible remaining convention/source discrepancy: not tuned or resolved'],
        gate_g1='open', experimental_validation=False)
    (ROOT / 'results/plc-virtual-comparison.json').write_text(json.dumps(report, indent=2) + '\n')
    text = ['# Published virtual-inlet comparison', '',
            'The geometry was transcribed from Figure 7 of von der Weiden 2009. Approximate graph anchors and a '
            '2 percentage-point absolute tolerance were fixed before running. Missing air state, bend orientation '
            'and taper-wall physics are recorded. Every result is retained.', '',
            '| Flow, L/min | Diameter, µm | Published loss | Computed loss | Pass | Main computed mechanisms (share of log-loss) |',
            '|---:|---:|---:|---:|---|---|']
    for r in rows:
        main = ', '.join(f'{k} {100 * v:.0f}%' for k, v in list(r['computed_loss_share_by_mechanism'].items())[:3])
        text.append(f"| {r['flow_l_min']} | {r['diameter_um']} | {100 * r['published_approximate_loss']:.2f}% | "
                    f"{100 * r['actual_loss']:.2f}% | {r['passed']} | {main} |")
    text += ['', f"Overall anchor check: **{'passed' if report['all_passed'] else 'FAILED'}**. "
             'This comparison does not close G1.', '',
             'The anchors are confirmed by the paper text, not only read from the graph: case 2 (40 L/min) gives '
             '"at least 5%" for all sizes and case 3 (150 L/min) "at least 40% for all particle sizes". The flow '
             'regimes the paper names (case 2 laminar in sections 1 to 5 and turbulent in section 6; case 3 '
             'turbulent everywhere) match the computed Reynolds numbers. One geometry input is missing: Figure 7 '
             'labels the inlet "super-isokinetic sampling", which a 50 mm orifice cannot give at 10 to 150 L/min '
             'with U0 = 3 m/s. The program takes the orifice diameter as a separate input and the figure does not '
             'state it, so the transcribed 50 mm probe is an assumption. With the printed probe equations (eq 5 to '
             '19, vena-contracta term extrapolated), an orifice small enough to make the 10 L/min case '
             'super-isokinetic adds under 4% loss at 0.1 to 0.3 µm and 150 L/min for orifices of 5 mm or more, and '
             'about 10% for a near-sonic 3 mm orifice; that term also grows with particle size instead of staying '
             'flat. This gap therefore does not explain the 40% plateau either.', '',
             'Where the computed fine-particle loss comes from: at 40 and 150 L/min the largest share is the '
             'size-independent `2e-8*Re` term of the Lee–Gieseke turbulent deposition velocity in the 10 mm tube, '
             'whose Reynolds number at 150 L/min (about 21000) lies above the fitted range. The published curve '
             'implies a loss that grows much faster with flow: between 40 and 150 L/min the computed log-loss '
             'scales as Q^' + '/'.join(f"{v['computed']:.2f}" for v in scaling.values()) + ' (0.1/0.3 µm), the '
             'anchors imply Q^' + '/'.join(f"{v['published']:.2f}" for v in scaling.values()) + '; the size-independent '
             'term alone scales as Q^0.875 at fixed geometry. A mechanism with the published scaling is '
             'not among the implemented correlations, so the discrepancy is not a single-coefficient '
             'error that could be corrected here.', '',
             'A failed anchor is evidence of an unresolved system/source discrepancy, not permission to fit a '
             'constant or copy PLC code. Potential omissions are named above, but are not demonstrated to explain '
             'a large discrepancy. The affected whole-system reproduction claim is withheld; the implementation '
             'remains a provisional literature-equation tool.']
    (ROOT / 'docs/validation/virtual-inlet.md').write_text('\n'.join(text) + '\n')
    print(json.dumps(report, indent=2))
    if not report['all_passed'] and '--report-only' not in sys.argv:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
