# Published virtual-inlet comparison

The geometry was transcribed from Figure7 of von der Weiden2009. Approximate graph anchors and a2percentage-point absolute tolerance were fixed before running. Missing air state, bend orientation and taper-wall physics are recorded. Every result is retained.

| FlowL/min | Diameterµm | Publishedloss | Computedloss | Pass |
|---:|---:|---:|---:|---|
|10|0.1|1.00%|1.38%|True|
|10|0.3|0.32%|1.25%|True|
|40|0.1|6.00%|2.84%|False|
|40|0.3|5.00%|2.14%|False|
|150|0.1|40.00%|6.20%|False|
|150|0.3|40.00%|5.54%|False|

Overall anchor check: **FAILED**. This comparison does not closeG1.

A failed anchor is evidence of an unresolved system/source discrepancy, not permission to fit a constant or copy PLC code. Potential omissions are named above, but are not demonstrated to explain a large discrepancy. The affected whole-system reproduction claim is withheld; the implementation remains a provisional literature-equation tool.
