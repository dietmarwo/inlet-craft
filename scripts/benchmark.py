#!/usr/bin/env python3
"""Single-process end-to-end timing of Rust through native JSON/dict binding."""
import json,os,platform,resource,statistics,time
from pathlib import Path
import inlet_craft
ROOT=Path(__file__).resolve().parents[1]
case=json.loads((ROOT/'examples/bicycle.json').read_text());samples=[]
for _ in range(5):inlet_craft.evaluate(case)
for _ in range(100):
    start=time.perf_counter();inlet_craft.evaluate(case);samples.append(time.perf_counter()-start)
report=dict(schema='inlet-craft.benchmark/0.1',operation='full bicycle evaluate via Python dict/JSON bridge, including serialization',repetitions=len(samples),scenarios=8,requested_particle_sizes=4,distribution_modes=2,quadrature_bins_per_mode_per_cut=48,median_seconds=statistics.median(samples),p95_seconds=sorted(samples)[94],evaluations_per_second=1/statistics.mean(samples),peak_process_rss_mib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss/1024,host_logical_cpus=os.cpu_count(),platform=platform.platform(),parallelism='one process, no objective-level worker threads',comparative_speedup=None,note='new project; no older baseline or external solver speed comparison')
(ROOT/'results/benchmark.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
