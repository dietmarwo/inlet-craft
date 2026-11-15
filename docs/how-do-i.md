# How do I…?

| Goal | Starting point |
|---|---|
| Compute a bicycle inlet | `examples/bicycle.json`; guide1–3 |
| Use a known non-unit particle density | Physical diameter plus density/shape, or aerodynamic diameter with the same properties; inspect physical conversion |
| Understand a warning | Its code/path/formula/range in result; formula registry and guide2 |
| Model crosswind or tailwind | Component arrays or wind samples; guide3 |
| Correct position lag | Mean age and actual trajectory history; guide3 |
| Reduce mixing blur | Smaller buffer, more flow, shorter sensor tau; include resulting aspiration/transport changes |
| Search designs | `examples/bicycle-study.json`; guide4 |
| Find a Pareto tradeoff | Study `multi_objective=true`; inspect cold-rechecked MODE archive |
| Reject extrapolation | Constrain warning groups and inspect mechanisms; a nonphysical extrapolation is always rejected by search |
| Model a housing slot | Obtain a verified model or measurements first; the thin-wall correlations do not cover it |
| Run independent seeded jobs | `examples/bicycle-job.json`; guide5 |
| Add a new correlation | Extend provenance/fixtures before code; keep source conflicts and failure evidence |
