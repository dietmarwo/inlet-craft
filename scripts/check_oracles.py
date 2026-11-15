#!/usr/bin/env python3
"""Installed-package numerical comparisons, never software ports.
Predeclared acceptance: slip5%, diffusion5%, penetration abs.02. Aerosolpy
uses different slip/mean-free-path and Gormley-Kennedy, so equality is not expected.
"""
import json,subprocess,tempfile,importlib.metadata,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CLI=ROOT/'target/release/inlet-craft'
def require(ok,message):
    if not ok:raise RuntimeError(message)
def main():
    import aerosolpy,numpy as np
    rows=[]
    for temp in [273.15,293.15,313.15]:
        for pressure in [80000.,101325.]:
            dp=np.geomspace(10.,10000.,25)
            case=dict(schema='inlet-craft.case/0.1',air=dict(temperature_k=temp,pressure_pa=pressure),platform=dict(speed_m_s=[5.]),particles=dict(physical_diameter_m=(dp*1e-9).tolist()),elements=[dict(kind='probe',inner_diameter_m=.004),dict(kind='tube',inner_diameter_m=.004,length_m=.2)],flow=dict(total_m3_s=2/60000,sensor_m3_s=1/60000))
            oracle=aerosolpy.AerosolMechanics(temp_kelvin=temp,pres_hpa=pressure/100.)
            with tempfile.NamedTemporaryFile('w',suffix='.json') as f:
                json.dump(case,f);f.flush();r=subprocess.run([str(CLI),'evaluate',f.name],capture_output=True,text=True,timeout=20)
                require(r.returncode==0,r.stderr);data=json.loads(r.stdout)
            for i,particle in enumerate(data['scenarios'][0]['sizes']):
                slip=float(oracle.slipcorr(dp[i]));diff=float(oracle.diff_coeff_p(dp[i]));penetration=float(oracle.diff_loss(dp[i],.2/(2/60000)))
                rust_penetration=particle['elements'][1]['mechanisms'][0]['efficiency']
                row=dict(temperature_k=temp,pressure_pa=pressure,diameter_nm=float(dp[i]),slip_relative_error=abs(particle['slip_correction']/slip-1),diffusion_relative_error=abs(particle['diffusion_coefficient_m2_s']/diff-1),penetration_absolute_error=abs(rust_penetration-penetration))
                row['passed']=row['slip_relative_error']<=.05 and row['diffusion_relative_error']<=.05 and row['penetration_absolute_error']<=.02
                rows.append(row)
    report=dict(schema='inlet-craft.oracle/0.1',aerosolpy_version=importlib.metadata.version('aerosolpy'),aerosolpy_license='MIT',predeclared_tolerances=dict(slip_relative=.05,diffusion_relative=.05,penetration_absolute=.02),all_passed=all(r['passed'] for r in rows),rows=rows,AeroSampleR=dict(status='not_run' if shutil.which('Rscript') is None else 'adapter_available_run_separately',reason='Rscript not installed in execution environment' if shutil.which('Rscript') is None else 'use scripts/export_aerosampler.R; fixtures not claimed until compared'),PLC=dict(status='not_run',reason='requires external IGOR Pro user; no PLC software inspected'),experimental_validation=False)
    for field in ['slip_relative_error','diffusion_relative_error','penetration_absolute_error']:
        report['maximum_'+field]=max(r[field] for r in rows)
    (ROOT/'results/oracle-comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='rows'},indent=2))
    require(report['all_passed'],'independent package comparison exceeds a predeclared tolerance; failed rows retained')
if __name__=='__main__':main()
