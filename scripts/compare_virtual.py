#!/usr/bin/env python3
"""Published qualitative system regression with explicit missing-geometry limits.
Default exits1 on mismatch. --report-only retains failures without claiming G1.
Tolerance:2percentagepoints absolute at approximate .1/.3um graph anchors,
fixed before running; no fitted constants. No PLC program code is consulted.
"""
import copy,json,subprocess,tempfile,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];CLI=ROOT/'target/release/inlet-craft'
def main():
    base=json.loads((ROOT/'examples/plc-virtual-inlet.json').read_text());rows=[]
    expected={10:[.01,.0032],40:[.06,.05],150:[.40,.40]}
    for flow in [10,40,150]:
        case=copy.deepcopy(base);case['flow']['total_m3_s']=flow/60000
        with tempfile.NamedTemporaryFile('w',suffix='.json') as f:
            json.dump(case,f);f.flush();p=subprocess.run([str(CLI),'evaluate',f.name],text=True,capture_output=True,timeout=20)
        if p.returncode:raise RuntimeError(p.stderr)
        r=json.loads(p.stdout);(ROOT/f'results/plc-virtual-{flow}lpm.json').write_text(json.dumps(r,indent=2)+'\n')
        for i,diam in enumerate([.1,.3]):
            actual=1-r['scenarios'][0]['sizes'][i]['overall_efficiency'];target=expected[flow][i]
            rows.append(dict(flow_l_min=flow,diameter_um=diam,published_approximate_loss=target,actual_loss=actual,absolute_tolerance=.02,passed=abs(actual-target)<=.02))
    report=dict(schema='inlet-craft.published-system-comparison/0.1',source='von der Weiden2009 Figure7 p492',source_url='https://doi.org/10.5194/amt-2-479-2009',evidence_kind='published_approximate_curve_with_incomplete_geometry',all_passed=all(r['passed'] for r in rows),rows=rows,limitations=['air state unspecified, assumed293.15K/101325Pa','bend arc diffusion/impaction included, arc settling orientation unspecified','taper wall losses unmodeled; conical volume only','primary Brockmann contraction factor2 differs from PLC printed Eq32','possible remaining convention/source discrepancy: not tuned or resolved'],gate_g1='open',experimental_validation=False)
    (ROOT/'results/plc-virtual-comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    text=['# Published virtual-inlet comparison','','The geometry was transcribed from Figure7 of von der Weiden2009. Approximate graph anchors and a2percentage-point absolute tolerance were fixed before running. Missing air state, bend orientation and taper-wall physics are recorded. Every result is retained.','','| FlowL/min | Diameterµm | Publishedloss | Computedloss | Pass |','|---:|---:|---:|---:|---|']
    for r in rows:text.append(f"|{r['flow_l_min']}|{r['diameter_um']}|{100*r['published_approximate_loss']:.2f}%|{100*r['actual_loss']:.2f}%|{r['passed']}|")
    text+=['',f"Overall anchor check: **{'passed' if report['all_passed'] else 'FAILED'}**. This comparison does not closeG1.",'','A failed anchor is evidence of an unresolved system/source discrepancy, not permission to fit a constant or copy PLC code. Potential omissions are named above, but are not demonstrated to explain a large discrepancy. The affected whole-system reproduction claim is withheld; the implementation remains a provisional literature-equation tool.']
    (ROOT/'docs/validation/virtual-inlet.md').write_text('\n'.join(text)+'\n')
    print(json.dumps(report,indent=2))
    if not report['all_passed'] and '--report-only' not in sys.argv:raise SystemExit(1)
if __name__=='__main__':main()
