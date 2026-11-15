# Generated interface reference

Generated from the current binary `describe`. Schemas are in schemas/. Registry is the authoritative source/validity ledger.

| Formula | Source and declared applicability |
|---|---|
|air.sutherland|Sutherland; dry ideal air, engineering range 200–350 K, 20000–120000 Pa|
|particle.allen_raabe|W p482; solid-sphere fit, original Kn .03–7.2; mean-free-path convention differs slightly|
|particle.stokes|B slide5; Stokes drag; settling Rep<.1|
|aspiration.belyaev_levin|B slide10; W eq5; St=tau*U0/d; 0.05<=St<=2.03, .17<=R<=5.6|
|aspiration.durham_lundgren|W eq6; 0<theta<=60deg, .02<=St<=4, .5<=R<=2|
|aspiration.hangal_willeke|W eq7 (prints exponent sqrt(U0/U)), Hangal & Willeke (1990a); implemented exponent sqrt(U/U0) is a declared source conflict (W_SOURCE_CONFLICT), see below; 61<=theta<=90deg, .02<=St<=.2, .5<=R<=2|
|aspiration.grinshpun_calm|B slide20, W eq8; phi angle to upward-facing vertical 0–90deg, .001<=q<=1, .001<=s<=100|
|aspiration.grinshpun_slow|W eq9–10; automatic switch at U0<=1.5m/s; extension of blend below.5m/s is an implementation policy, reported; underlying correlation ranges still checked|
|transmission.liu|B slide13, W eq12; 1<=R<=10, .01<=St<=100|
|transmission.hangal_willeke|B slide14, W eq13–14; .25<=R<=1, .02<=St<=4|
|transmission.hangal_willeke_angle|W eq15–19 (Iw transcription corrected 2026-11-15); .02<=St<=4,.25<=R<=4,0<theta<=90; documented downward/upward branch|
|tube.diffusion_holman|W eq21–22; laminar Re<2000|
|tube.diffusion_friedlander|W eq21,23; turbulent Re>4000|
|tube.settling_heyder|B slide26, W eq24–25; a>=1 gives complete deposition (analytic domain); abs(v*sin(i)/U)<<1 checked against stated .1 policy|
|tube.settling_schwendiman|B slide26, W eq26–27; turbulent flow|
|tube.inertia_lee_gieseke|W eq28–29, Re 4000–15600 (Lee/Gieseke original DOI 10.1016/0021-8502(94)90011-6); St=tau*U/d|
|bend.pui_laminar|B slide38, W eq30; St=tau*U/d, curvature radius/(d/2) 5–30; fit measured at Re1000, curvature 5.6–5.7; generic curvature envelope 5–30 is not additional fitted-condition evidence|
|bend.pui_turbulent|B slide38, W eq31; Re>4000, curvature ratio 5–30|
|contraction.muyshondt|B slide42 / notes eq40; W eq32 omits factor 2; C St=tau*U_upstream/d_downstream; .001<=x<=100, 12–90deg|
|response.series|mass conservation / linear systems, explicitly modeled assumptions|

Diagnostic codes: E_JSON, E_IO, E_USAGE, E_SCHEMA, E_POSITIVE, E_NONNEGATIVE, E_SHAPE, E_WEIGHTS, E_SAMPLES, E_RESOURCE, E_FINITE, E_WIND, E_PARTICLES, E_DIAMETER, E_FLOW, E_ELEMENTS, E_PROBE, E_ANGLE, E_UNCOVERED, E_TRANSITION, E_REGIME, E_ANGLE_UNCOVERED, E_NUMERIC, E_RESPONSE, E_DISTRIBUTION, E_BUDGET, E_VARIABLES, E_VARIABLE, E_BOUNDS, E_DIMENSION, E_CASE, E_EMPTY_PM, E_FIXTURE, W_RANGE, W_NONPHYSICAL, W_TRANSITION, W_REGIME_BOUNDARY, W_SLOW_POLICY, W_CALM_TRANSMISSION, W_WALL_ASSUMED, W_BEND_SETTLING, W_SOURCE_CONFLICT, W_CONTRACTION_TRANSPORT, W_BUFFER_LOSS_UNMODELED, W_IDEAL_SPLIT.
