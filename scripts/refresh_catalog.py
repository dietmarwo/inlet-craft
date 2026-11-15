#!/usr/bin/env python3
"""Regenerate structured source/validity catalog before compiling describe."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
rows=[]
for line in (ROOT/'docs/developer/formula-registry.md').read_text().splitlines():
    if not line.startswith('|') or line.startswith('|---') or line.startswith('| ID'):continue
    cells=[c.strip() for c in line.strip('|').split('|')]
    if len(cells)==4:rows.append(dict(id=cells[0],equation=cells[1],source_and_validity=cells[2],verification_anchor_and_limit=cells[3]))
(ROOT/'fixtures/formula-catalog.json').write_text(json.dumps(rows,indent=2)+'\n')
print(f'{len(rows)} formula entries refreshed; rebuild binary before check_docs')
