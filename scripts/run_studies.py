#!/usr/bin/env python3
"""Bounded reproducible design campaign and generated numerical guide."""
import copy,json,subprocess,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];CLI=ROOT/'target/release/inlet-craft'
def run(command,value,timeout=90):
    with tempfile.NamedTemporaryFile('w',suffix='.json') as f:
        json.dump(value,f);f.flush();p=subprocess.run([str(CLI),command,f.name],capture_output=True,text=True,timeout=timeout)
    if p.returncode:raise RuntimeError(p.stderr)
    return json.loads(p.stdout)
def main():
    study=json.loads((ROOT/'examples/bicycle-study.json').read_text());baseline=run('evaluate',study['case']);(ROOT/'results/bicycle-baseline.json').write_text(json.dumps(baseline,indent=2)+'\n')
    rows=[]
    variants=[]
    for seed in [42,7,123]:
        s=copy.deepcopy(study);s.update(seed=seed,max_evaluations=512,wall_time_seconds=45.);variants.append((f'full-distribution-seed{seed}',s))
    s=copy.deepcopy(study);s.update(seed=42,max_evaluations=256,multi_objective=True,wall_time_seconds=45.);variants.append(('mode-full-distribution',s))
    s=copy.deepcopy(study);s['case']['particles']={'aerodynamic_diameter_m':[5e-6]};s['case']['distribution']['modes']=[dict(number_concentration_m3=1e6,geometric_mean_diameter_m=5e-6,geometric_std_dev=1.)];s.update(max_evaluations=512,wall_time_seconds=45.);variants.append(('coarse-monodisperse-sensitivity',s))
    coarse_mode=copy.deepcopy(s);coarse_mode.update(multi_objective=True,max_evaluations=256);variants.append(('mode-coarse-monodisperse-sensitivity',coarse_mode))
    s=copy.deepcopy(s);s['constraints']['max_distance_resolution_m']=0.;s['max_evaluations']=64;variants.append(('deliberately-impossible-zero-resolution',s))
    for label,s in variants:
        result=run('optimize',s,timeout=60);(ROOT/f'results/{label}.json').write_text(json.dumps(result,indent=2)+'\n');b=result['best'];rows.append(dict(study=label,status=result['status'],evaluations=result['evaluations'],failed_evaluations=result['failed_evaluations'],elapsed_seconds=result['elapsed_seconds'],worst_pm10_bias=b['maximum_pm10_relative_deviation'] if b else None,resolution_at_reference_m=b['distance_resolution_objective_m'] if b else None,flow_l_min=b['total_flow_l_min'] if b else None,warning_count=b['warning_count'] if b else None,violations=b['violations'] if b else [],pareto_points=len(result['pareto'])))
        if b:(ROOT/f'results/{label}-best-case.json').write_text(json.dumps(b['case'],indent=2)+'\n')
    summary=dict(schema='inlet-craft.study-campaign/0.1',date='2026-11-14',distribution='explicit example assumptions, not measured site data',rows=rows,experimental_validation=False)
    (ROOT/'results/study-campaign.json').write_text(json.dumps(summary,indent=2)+'\n')
    out=['# Bicycle design study: executed results','','Date2026-11-14. Source defaults and random seeds are retained in examples/results. These are computed correlation outputs, not field measurements.','','## Baseline at2L/min','','| Speedkm/h | Crosswindm/s | PM2.5relativebias | PM10relativebias | Meanpositionoffsetm | Resolutionm |','|---:|---:|---:|---:|---:|---:|']
    for s in baseline['scenarios']:
        p=s['pm'];out.append(f"|{s['conditions']['riding_speed_m_s']*3.6:.1f}|{s['conditions']['crosswind_m_s']:.1f}|{100*(p[0]['efficiency']-1):+.2f}%|{100*(p[1]['efficiency']-1):+.2f}%|{s['mean_position_offset_m']:.2f}|{s['distance_resolution_10_90_m']:.2f}|")
    out+=['','## Budgeted search','','Resolution goal15m at18km/h, PM10within15%, totalflow≤3L/min, laminar flow. All nonphysical extrapolations are hard rejected. The all-speeds maximum resolution is also retained in each raw candidate.','','| Study | Budgetused | Accepted | PM10worstbias | Resolutionat18km/h | FlowL/min | Paretopoints |','|---|---:|---|---:|---:|---:|---:|']
    for row in rows:
        out.append(f"|{row['study']}|{row['evaluations']}|{row['status']}|{row['worst_pm10_bias']:.5f}|{row['resolution_at_reference_m']:.3f}|{row['flow_l_min']:.3f}|{row['pareto_points']}|")
    out+=['','## Interpretation','','The full-lognormal study is limited by a source/extrapolation issue: for sufficiently small Stokes numbers and sub-isokinetic conditions, Liu transmission slightly exceeds1. This lies outside the source fitting range. Even very low-mass fine tails trigger the explicit nonphysical guard. The integrated PM10bias and resolution alone do not settle feasibility. These failed searches therefore give no evidence that the hardware goal is impossible. No constants or outputs were clipped to make them pass.','','The coarse monodisperse sensitivity removes that fine-tail issue and tests a different declared aerosol. A feasible result there demonstrates search functionality and conditional tradeoffs only; it is not a recommended bike design or a validation for realistic multimodal aerosol. The deliberately impossible zero-resolution goal tests truthful finite-search failure reporting.','','### Reported best-candidate violations','']
    for row in rows:out.append(f"- **{row['study']}**: "+('; '.join(row['violations']) if row['violations'] else 'no declared constraint violation; scientific coverage warnings remain.'))
    out+=['','G2practitioner acceptance remains pending. Raw cases and failed rows are preserved in results; reference and oracle evidence are distinct from optimization.']
    (ROOT/'docs/bicycle-study.md').write_text('\n'.join(out)+'\n')
    print(json.dumps(summary,indent=2))
if __name__=='__main__':main()
