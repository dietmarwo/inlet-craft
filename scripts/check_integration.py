#!/usr/bin/env python3
"""CLI/schema/native Python/job integration checks; assertions survive python -O."""
import copy,json,os,subprocess,tempfile,time,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CLI=ROOT/'target/release/inlet-craft'
WORKER=ROOT/'target/release/inlet-craft-worker'
def require(ok,message):
    if not ok: raise RuntimeError(message)
def run(args,expected=0,timeout=30):
    p=subprocess.run(list(map(str,args)),cwd=ROOT,text=True,capture_output=True,timeout=timeout)
    require(p.returncode==expected,f'{args}: exit{p.returncode}; expected{expected}: {p.stderr[:1000]}')
    return json.loads(p.stdout if p.stdout.strip() else p.stderr)
def read_records(path):
    records=[]
    for line in path.read_text().splitlines():
        envelope=json.loads(line)
        require(envelope['schema_version']==1,'journal envelope schema')
        payload=envelope['payload_json']
        require(hashlib.sha256(payload.encode()).hexdigest()==envelope['checksum'],'journal checksum')
        records.append(json.loads(payload))
    return records
def main():
    checks=[]
    desc=run([CLI,'describe'])
    case=json.loads((ROOT/'examples/bicycle.json').read_text())
    result=run([CLI,'evaluate',ROOT/'examples/bicycle.json'])
    import jsonschema
    jsonschema.Draft7Validator(desc['schemas']['case']).validate(case)
    jsonschema.Draft7Validator(desc['schemas']['result']).validate(result)
    require(len(result['scenarios'])==8,'Cartesian scenario count')
    checks.append('CLI case/result JSON schemas')
    with tempfile.TemporaryDirectory() as td:
        td=Path(td)
        bad=copy.deepcopy(case);bad['flow']['total_lpm']=3
        path=td/'bad.json';path.write_text(json.dumps(bad))
        e=run([CLI,'check',path],2);require(e['error']['code']=='E_JSON' and 'flow' in e['error']['path'],'nested strict field diagnostic')
        checks.append('nested unknown fields rejected with path')
        f=json.loads((ROOT/'fixtures/reference.json').read_text());f[0]['expected']=9
        path=td/'failed-fixture.json';path.write_text(json.dumps(f));v=run([CLI,'verify',path],1)
        require(not v['all_passed'] and len(v['rows'])==len(f) and not v['rows'][0]['passed'],'failed fixture rows retained')
        checks.append('verify nonzero status and all failed/passed rows retained')
        sweep=run([CLI,'sweep',ROOT/'examples/bicycle.json','/elements/3/volume_m3','0,-1,0.00001'])
        require(len(sweep['rows'])==3 and 'error' in sweep['rows'][1],'sweep failed row retained')
        checks.append('sweep mixed valid/invalid values')
        spec=json.loads((ROOT/'examples/bicycle-job.json').read_text());spec['payload']['max_evaluations']=12;spec['payload']['case']['platform']['speed_m_s']=[5.];spec['payload']['case']['platform']['crosswind_m_s']=[0.];spec['payload']['case']['distribution']['quadrature_bins']=16
        path=td/'job.json';path.write_text(json.dumps(spec))
        description=run([WORKER,'describe']);require(description['protocol']=='optimization.worker/0.2','SDK protocol')
        resolved=run([WORKER,'validate',path])['resolved'];require(resolved['task_count']==1,'task count')
        attempt=td/'attempt';attempt.mkdir();(attempt/'resolved.json').write_text(json.dumps(resolved));run([WORKER,'execute',attempt])
        records=read_records(attempt/'results.jsonl')
        require(len(records)==1 and (attempt/'run.json').exists(),'job journal committed')
        require(records[0]['outcome']['claim']!='infeasible','finite search cannot assert infeasibility')
        (ROOT/'results/worker-smoke-record.json').write_text(json.dumps(records[0],indent=2)+'\n')
        checks.append('optimization-jobs validate/execute/checksummed-journal/evidence')
        cancelled=td/'cancelled';cancelled.mkdir();(cancelled/'resolved.json').write_text(json.dumps(resolved));(cancelled/'cancel.request').write_text('test')
        run([WORKER,'execute',cancelled]);require((cancelled/'results.jsonl').read_text()=='','pre-cancel does no task')
        checks.append('pre-existing job cancellation')
        longspec=copy.deepcopy(spec);longspec['payload']['max_evaluations']=1000000
        path.write_text(json.dumps(longspec));longresolved=run([WORKER,'validate',path])['resolved']
        live=td/'live';live.mkdir();(live/'resolved.json').write_text(json.dumps(longresolved));began=time.monotonic()
        p=subprocess.Popen([str(WORKER),'execute',str(live)],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        time.sleep(.10);(live/'cancel.request').write_text('test cooperative stop')
        stdout,stderr=p.communicate(timeout=10)
        require(p.returncode==0,f'active cancellation failed: {stderr}')
        require(time.monotonic()-began<10,'active cancel timeout')
        records=read_records(live/'results.jsonl')
        if records:
            require(records[0]['outcome']['termination']=='cancelled','in-flight solver cancellation')
        checks.append('in-flight cooperative job cancellation <10s')
    import inlet_craft
    py=inlet_craft.evaluate(case)
    require(py==result,'Python and CLI must return identical numerical reports')
    require(inlet_craft.verify()['all_passed'],'native verify')
    require(inlet_craft.check(case)==run([CLI,'check',ROOT/'examples/bicycle.json']),'native check must equal CLI check')
    require(len(inlet_craft.sweep(case,'/elements/3/volume_m3',[0.,1e-5])['rows'])==2,'native sweep')
    checks.append('installed native Python check/evaluate/sweep/verify parity')
    report=dict(schema='inlet-craft.integration/0.1',passed=True,checks=checks,python_optimized=not __debug__)
    (ROOT/'results/integration.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
