# 5. Python and durable jobs

Build a native wheel using `maturin build --release --locked`, then install the resulting wheel from `target/wheels/`. The binding uses the same Rust parser and evaluator as the CLI and returns plain Python dictionaries. A JSON-coded `ValueError` carries Rust diagnostics. The wheel built in this campaign is for the execution environment's CPython/Linux, not Android.

```python
import json
import inlet_craft
with open('examples/bicycle.json') as f:
    case=json.load(f)
result=inlet_craft.evaluate(case)
if len(result['scenarios']) != 8:
    raise RuntimeError('unexpected scenario count')
if not inlet_craft.verify()['all_passed']:
    raise RuntimeError('reference verification failed')
```

Python functions are `check`, `evaluate`, `describe`, `verify`, `sweep`, and `optimize`. The native functions exchange JSON internally; the Python sweep helper copies the input for each value and retains row failures. Evidence checks do not depend on Python assert statements and are repeated under `python -O`.

`inlet-craft-worker` uses the actual optimization-jobs SDK, profile`inlet-craft.optimize/1`, worker ID`inlet-craft`, protocol`optimization.worker/0.2`. `examples/bicycle-job.json` embeds the study in a standard job envelope. Use the worker's `validate SPEC` output's `resolved` object as `ATTEMPT/resolved.json`, then execute `inlet-craft-worker execute ATTEMPT`. The SDK writes checked result journals and run manifests. A `cancel.request` file cancels the task. The integration checker exercises validation, execution, pre-cancel and in-flight cancellation.

One study is one task, with `inner_threads=1`. For parallel exploration submit independent seeded jobs to optimization-jobs/experiment-craft. This worker has no distributed orchestration implementation and makes no claim about multicore speedup. The SDK's claim is `best_found` only for a feasible cold-checked point, otherwise `none`; an unsuccessful finite search never reports a global `infeasible` claim.
