# Published virtual-inlet comparison

The geometry was transcribed from Figure 7 of von der Weiden 2009. Approximate graph anchors and a 2 percentage-point absolute tolerance were fixed before running. Missing air state, bend orientation and taper-wall physics are recorded. Every result is retained.

| Flow, L/min | Diameter, µm | Published loss | Computed loss | Pass | Main computed mechanisms (share of log-loss) |
|---:|---:|---:|---:|---|---|
| 10 | 0.1 | 1.00% | 1.38% | True | tube.diffusion_holman 87%, tube.settling_heyder 14% |
| 10 | 0.3 | 0.32% | 1.25% | True | tube.settling_heyder 74%, tube.diffusion_holman 30%, aspiration -4% |
| 40 | 0.1 | 6.00% | 2.84% | False | tube.inertia_lee_gieseke 53%, tube.diffusion_friedlander 35%, tube.diffusion_holman 11% |
| 40 | 0.3 | 5.00% | 2.14% | False | tube.inertia_lee_gieseke 71%, tube.diffusion_friedlander 15%, tube.settling_heyder 9% |
| 150 | 0.1 | 40.00% | 6.20% | False | tube.inertia_lee_gieseke 81%, tube.diffusion_friedlander 19% |
| 150 | 0.3 | 40.00% | 5.54% | False | tube.inertia_lee_gieseke 92%, tube.diffusion_friedlander 7%, tube.settling_schwendiman 1% |

Overall anchor check: **FAILED**. This comparison does not close G1.

Where the computed fine-particle loss comes from: at 40 and 150 L/min the largest share is the size-independent `2e-8*Re` term of the Lee–Gieseke turbulent deposition velocity in the 10 mm tube, whose Reynolds number at 150 L/min (about 21000) lies above the fitted range. The published curve implies a loss that grows much faster with flow: between 40 and 150 L/min the computed log-loss scales as Q^0.61/0.73 (0.1/0.3 µm), the anchors imply Q^1.60/1.74; the size-independent term alone scales as Q^0.875 at fixed geometry. A mechanism with the published scaling is not among the implemented correlations, so the discrepancy is not a single-coefficient error that could be corrected here.

A failed anchor is evidence of an unresolved system/source discrepancy, not permission to fit a constant or copy PLC code. Potential omissions are named above, but are not demonstrated to explain a large discrepancy. The affected whole-system reproduction claim is withheld; the implementation remains a provisional literature-equation tool.
