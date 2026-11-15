# 1. First evaluation

Build and run the commands in the README. `examples/bicycle.json` describes a defined thin-walled inlet, not the slot of a sensor housing. Its total flow is 2 L/min, sensor branch 0.282 L/min, tube 15 cm long, bend90deg with2cm radius, and buffer20mL. Inputs remain SI in JSON: multiply L/min by1/60000 for m3/s, milliliters by1e-6 for m3.

Start with `check`. An error prevents evaluation: unknown fields, negative flow, a blunt wall, an unexplained diameter jump, backward-facing wind, the60–61deg correlation gap, or transition tube flow without explicit permission. A warning permits a numerical result and states what is extrapolated or unmodeled.

`evaluate` returns one scenario per positive-weight Cartesian combination. Each scenario has an efficiency curve, element/mechanism breakdown, the distribution-dependent PM response and the modeled time response. `size_statistics` averages the declared scenario probabilities; it is not a climatological confidence interval. Inspect warnings before using any curve.

The assumed distribution and model limitations travel in the result. A good-looking PM10 average can result from cancellation of over-aspiration and transport loss; changing the coarse mode can remove that cancellation. Compare size-resolved curves, worst-case scenarios and multiple distributions before choosing hardware.
