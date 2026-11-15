"""Rust-powered aerosol inlet analysis. Inputs and outputs are plain dicts."""
import copy
import json
from . import _inlet_craft as _native

def evaluate(case):
    return json.loads(_native.evaluate_json(json.dumps(case, allow_nan=False)))

def check(case):
    return json.loads(_native.check_json(json.dumps(case, allow_nan=False)))

def describe():
    return json.loads(_native.describe_json())

def verify():
    return json.loads(_native.verify_json())

def optimize(study):
    return json.loads(_native.optimize_json(json.dumps(study, allow_nan=False)))

def sweep(case, pointer=None, values=None):
    if pointer is None:
        return {"schema":"inlet-craft.sweep/0.1", "evaluation":evaluate(case)}
    if not isinstance(pointer, str) or not pointer.startswith('/'):
        raise ValueError('pointer must be an absolute JSON pointer')
    rows=[]
    for value in values:
        c=copy.deepcopy(case)
        parts=[p.replace('~1','/').replace('~0','~') for p in pointer[1:].split('/')]
        target=c
        for part in parts[:-1]:
            target=target[int(part)] if isinstance(target,list) else target[part]
        key=int(parts[-1]) if isinstance(target,list) else parts[-1]
        if isinstance(target,dict) and key not in target:
            raise ValueError('missing JSON pointer')
        target[key]=value
        try:
            rows.append({"value":value,"evaluation":evaluate(c)})
        except ValueError as exc:
            try:
                error=json.loads(str(exc)).get('error')
            except json.JSONDecodeError:
                error={"code":"E_PYTHON","message":str(exc)}
            rows.append({"value":value,"error":error})
    return {"schema":"inlet-craft.sweep/0.1","axis":pointer,"rows":rows}
