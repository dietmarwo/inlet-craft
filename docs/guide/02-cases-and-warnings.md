# 2. Cases, units and diagnostics

`schema` must equal `inlet-craft.case/0.1`. Nested unknown fields are rejected. `particles` takes exactly one nonempty `aerodynamic_diameter_m` or `physical_diameter_m` list. Density defaults to 1000kg/m3 and dynamic shape factor to 1. Aerodynamic input is converted to physical diameter by a slip-corrected implicit solve; density is not applied twice. Diffusion uses physical size. Shape factor is used as an effective drag/mobility factor, an assumption for non-spherical particles.

| Element | Required fields and meaning |
|---|---|
| `probe` | First and unique; `inner_diameter_m`. Optional outer diameter must satisfy 1<=outer/inner<1.1; omission emits an assumption warning. `angle_to_heading_deg` is yaw, `angle_to_vertical_deg` defaults 90(horizontal), 0 is vertical. |
| `tube` | `length_m`, `inner_diameter_m`; `inclination_deg` defaults 0(horizontal). Diameter matches upstream. |
| `bend` | `angle_deg`, centerline `radius_m` (must exceed the tube radius); optional matching inner diameter. Arc diffusion and travel time included; settling along unknown arc orientation omitted with warning. |
| `contraction` | Smaller `outlet_diameter_m`, `half_angle_deg`; optional axial `length_m` for conical volume. Uses upstream velocity/downstream diameter and the reference's factor 2. |
| `buffer` | `volume_m3`; time constantV/Q only, particle wall losses unmodeled. |
| `split` | `sensor_m3_s` equals global sensor flow. Downstream elements carry that flow. Ideal mixed concentration preserved; count flux partitions. |

If there is no explicit split, the sensor samples the final stream. Flow fractions in results are relative to ambient concentration times **total inlet flow**; aspiration enrichment can make them larger than one. With downstream branch losses, branch plus bypass flux is reduced by the branch deposition.

Air is dry and ideal. The chosen Sutherland/kinetic mean-free-path convention differs slightly from the original Allen–Raabe fit; `W_RANGE` reports its Knudsen envelope. Coagulation, charge, wall temperature, evaporation, platform wakes, humidity and sensor optics are excluded. The independent-mechanism product assumes losses do not alter later deposition behavior.

`E_TRANSITION` covers Re 2000–4000 inclusively. `options.extend_laminar_transition=true` applies laminar equations with `W_TRANSITION`. This is an estimate, not a verified transition correlation. A source-provenance or omitted-process warning is different from a formula-range warning.

Warnings are grouped by code, element path, formula and quantity; minimum, maximum and occurrence counts cover both requested sizes and PM quadrature particles. `warning_count` counts groups; `warning_event_count` counts individual events and depends on quadrature size. `W_REGIME_BOUNDARY` reports empirical endpoint jumps as advisory information, including regimes not used by that scenario. The endpoint classification has a 1e-10deg floating-point tolerance; the true 60–61deg gap is never interpolated. `W_NONPHYSICAL` carries the offending efficiency and its formula. Efficiencies are never clipped; instead `nonphysical_transport_excess` (per size) and `nonphysical_excess` (per PM cut) state how much of a result rests on transport factors above one.

`upward_sampling` selects the source's upward-flow transmission branch; false is downward sampling/upward-facing aperture. The tilt field controls effective angle, while this branch flag records orientation convention. The Grinshpun calm fit covers an upward-facing aperture tilted 0–90deg; a downward-facing calm/slow aperture is outside this implementation's geometry. Arbitrary three-dimensional external wind is not supported.
