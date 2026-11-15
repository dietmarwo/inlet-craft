# inlet-craft: incremental plan for an aerosol inlet and sampling-line design tool

Date: 2026-11-14
Status: proposed. Nothing exists yet; no repository is created. The name is a proposal (section 1). Companion document for users: `claude/inlet-craft-sensor-unit-design-2026-11-14.md`.

## 0. Why, in three sentences

Aerosol measurements from moving platforms need an inlet whose sampling efficiency is known by particle size and speed. The formulas are published, but the existing implementations are an IGOR Pro program with a restrictive license, an R package for still-air sampling lines and spreadsheets. An open library with strict units, validity checks, a system model with time response and a design search would fit the craft ecosystem and a real user: a project that measures fine dust from cargo bikes.

## 1. Name

**Proposed: `inlet-craft`** (repository `dietmarwo/inlet-craft`, Python package `inlet-craft`, crates `inlet-craft` and `inlet-craft-cli`).

- It says what is designed: the inlet system, from the opening to the sensor.
- It follows the family: CFD-craft, CRN-craft, SQP-craft, budel-craft.
- Checked on 14 November 2026: no package of this name on PyPI or crates.io, and a GitHub repository search for the name returns no project of this kind.

| Alternative | Why not |
|---|---|
| `sampler-craft` | "Sampler" means a random-number or MCMC sampler in an optimization ecosystem |
| `aerosol-craft` | Promises aerosol dynamics and chemistry, which the tool will not have |
| `probe-craft` | "Probe" is too general |
| `sniff-craft` | Informal; hides what the tool computes |

All four alternatives are also free on PyPI and crates.io.

## 2. Decision question and gates

**Question:** does an open, scriptable implementation of the published sampling and transport formulas let an agent design an inlet for a moving platform, with a result that an aerosol practitioner accepts and that tests on a bike confirm?

- **G0 (after I0):** a formula registry in which every formula has its original source, its validity range and at least one published number it must reproduce. The licence position is written down. The owner decides whether to create the repository.
- **G1 (after I3):** every mechanism reproduces its published numbers within tolerances declared beforehand, the limits hold (section 6), and overlapping mechanisms agree with an independent implementation. A mismatch is explained or the formula is withdrawn; no constant is fitted.
- **G2 (after I6):** a design study for a bicycle inlet gives a result a practitioner finds non-trivial, and states which requirements no design meets. Release 0.1.0 is then the owner's decision.
- **G3 (track V):** on-bike co-location at several speeds agrees with the predicted speed dependence within a tolerance fixed before the data are opened, or the disagreement is traced to a part the formulas do not cover. This is the only gate that carries an experimental claim.

## 3. Verified baseline facts

- **Particle Loss Calculator** (Max Planck Institute for Chemistry; paper in Atmos. Meas. Tech. 2009, Creative Commons Attribution 3.0). It implements, with literature sources and validity ranges:
  - aspiration in moving air facing the flow (Belyaev and Levin), at 0 to 60° (Durham and Lundgren) and at 61 to 90° (Hangal and Willeke); in calm and slow-moving air (Grinshpun and co-authors);
  - transmission in the probe (Liu and co-authors; Hangal and Willeke);
  - diffusion in laminar and turbulent tube flow; settling in horizontal, inclined and turbulent flow; turbulent inertial deposition; bends (Pui and co-authors); contractions (Muyshondt and co-authors).
  - Not included, each with a reason: enlargements, electrostatics, thermophoresis, diffusiophoresis, interception, coagulation, re-entrainment.
  - Limits stated by its authors: thin-walled probes of constant diameter; no formula for the transition between laminar and turbulent flow; extreme tube geometries are underpredicted.
  - The program needs IGOR Pro. Its license allows changes to the code only after consulting the licensor.
- **AeroSampleR** (R, on CRAN): probe in calm air, tubes and bends for sampling lines; one diameter per system.
- **aerosolpy** (Python, on PyPI): slip correction, diffusion coefficients, diffusional losses.
- **For the application:** at riding speeds of 10 to 25 km/h, 10 µm particles have Stokes numbers of 0.1 to 0.5 at inlets of 4 to 8 mm; 2.5 µm particles stay below 0.05. The inlet therefore matters for PM10 and hardly for PM2.5. The numbers are in the companion document.
- **Not verified:** one paper mentions "an open-source particle loss calculator" that I could not identify. I0 searches again before any code is written.

## 4. Scope and model

**In scope**

- Air properties, slip correction, relaxation time, settling velocity, diffusion coefficient, Stokes and Reynolds numbers.
- Inlet (probe): aspiration and transmission for a thin-walled tube in moving, slow and calm air at an angle.
- Elements: straight tube with inclination, bend, contraction, well-mixed buffer volume, flow split to a sensor.
- System: elements in series, efficiencies multiplied, per particle size, with the assumption stated.
- Moving platform: riding speed and crosswind give the apparent wind speed and its angle to the inlet axis; distributions of both give statistics of the efficiency.
- Time response: travel time, mixing time constants, sensor lag and sampling interval, as time and as distance at a speed; the position offset.
- Size distributions and the mass-weighted effect on PM2.5 and PM10, compared with an ideal inlet of the same nominal cut.
- Design search over stated parameters, objectives and constraints.

**Out of scope**

- Blunt or shrouded inlets and housing slots: no formula is implemented for them, so they are rejected with a diagnostic.
- Sensor optics, humidity response, fan behavior in wind.
- Flow around the platform.
- Gases beyond travel time and mixing.
- Enlargements, electrostatics, thermophoresis, re-entrainment (as in the Particle Loss Calculator), until a verified formula and a user need exist.

**Conventions**, as in the craft ecosystem

- Rust core with `forbid(unsafe_code)`, a command line, a Python binding; MIT license.
- One JSON case with a versioned schema, SI units in the field names, unknown fields rejected.
- Coded diagnostics with the path of the offending field. A formula used outside its validity range gives a coded warning that names the quantity, its value and the range; it never fails silently and never clamps.
- `describe` lists only what is implemented: every formula with its source and range.
- Evidence checks that behave the same under `python -O`; failed rows kept; no tags or publishing without the owner.
- Documentation from the first increment, with every code block run by a checker, as introduced in CFD-craft 0.1.1.

## 5. Increments

Effort is in agent days and is a rough guess. Each increment ends with a review by an agent that did not write it.

### I0 — Formula registry and licence position (about 1 day, no code)

- For every mechanism of section 3: obtain the formula from the original paper or from the standard reference chapter, not from another program. Record the equation, symbols, units, validity range and the assumptions.
- Where sources differ (constants, ranges, the definition of the Stokes number), record both and decide with a reason.
- For every formula, find at least one published number or curve point to reproduce, and one limit it must satisfy.
- Licence: formulas are implemented from the literature and cited. No code of the Particle Loss Calculator is read or ported. AeroSampleR and aerosolpy are used only as independent oracles in tests, run as installed packages; check their licenses for that use.
- Search once more for an existing open implementation for moving platforms. If one exists and is sound, the plan changes to contributing or wrapping.

**Gate G0.** Model: Fable, high (judgment on sources). Output: `docs/developer/formula-registry.md` as the first file of the future repository.

### I1 — Particle and air primitives (about 1 day)

- Air viscosity and mean free path as functions of temperature and pressure; slip correction; relaxation time; settling velocity with its validity check on the particle Reynolds number; diffusion coefficient; aerodynamic against physical diameter with density and shape factor.
- Fixtures: textbook tables for standard conditions; cross-check against aerosolpy on a grid; dimensional tests.

Model: Sonnet or Opus, medium.

### I2 — The inlet: aspiration and transmission (about 2 days)

- Aspiration for the three moving-air regimes by angle, and for calm and slow-moving air; transmission in the probe.
- The regime is chosen from the conditions and reported. Where two regimes meet, the jump between their formulas is computed and reported, not smoothed.
- Fixtures: the published points of I0; matched speeds give an efficiency of one; vanishing Stokes number gives one; very large Stokes number in facing flow tends to the ratio of wind to inlet speed; symmetry in the sign of the angle where the source states it.

Model: Opus, medium to high.

### I3 — Transport elements and the system (about 2 days)

- Tube: diffusion and settling in laminar and turbulent flow, turbulent inertial deposition; the flow regime from the Reynolds number, with the transition range reported as uncovered and an explicit option to extend the laminar formulas there with a warning.
- Bend, contraction, buffer volume (time constant, settling and diffusion in a stirred volume if I0 found a verified formula; otherwise time constant only).
- System in series; per-mechanism and per-element breakdown in the result.
- Fixtures: published points; zero length gives one; efficiencies between zero and one; longer tube never increases efficiency; code-to-code comparison with AeroSampleR on tubes and bends, with differences in chosen formulas explained.
- A regression case: the "virtual inlet" example of the Particle Loss Calculator paper at its three flow rates, compared with the losses stated there, as far as the paper gives the geometry.

**Gate G1.** Model: Opus, medium to high.

### I4 — Case schema, command line, Python binding (about 2 days)

- `inlet-craft.case/0.1` and the result schema; `describe`, `check`, `evaluate`, `verify`.
- `check` rejects designs with an uncovered part and explains why.
- Guide chapters 1 to 3: first evaluation, reading a result and its warnings, describing a unit. Reference pages generated from `describe`.

Model: Opus, medium.

### I5 — Moving platform, size distributions, time response (about 2 days)

- Apparent wind from riding speed, wind speed and wind direction relative to heading; statistics over declared distributions.
- Lognormal size modes; mass-weighted PM2.5 and PM10 against an ideal inlet; the result states the assumed distribution prominently.
- Time response chain and its distance equivalents; the position offset as a function of speed.
- `sweep` over speed, crosswind or one design parameter.
- Fixtures: zero riding speed reduces to the stationary case; a single-size distribution reproduces the size-resolved result; a chain of two mixed volumes against the analytic step response.

Model: Opus, medium.

### I6 — Design search and jobs (about 2 days)

- A study file: design variables with bounds, objectives (for example the worst-case PM10 deviation over the speed and crosswind distribution), constraints (distance resolution, total flow, laminar flow, all formulas inside their ranges or a stated number of warnings).
- Search with fcmaes-rust; a multi-objective variant for bias against resolution against flow.
- A worker for optimization-jobs, so an agent can run studies with budgets and cancellation.
- "No design meets the requirements" is a valid, reported outcome.
- A guide chapter with the bicycle study of the companion document, every number produced by running.

**Gate G2.** Model: Opus, medium; the study reviewed by Fable, high.

### I7 — Documentation, comparison, release (about 1.5 days)

- Guide ordered by difficulty; "How do I …?" page; a comparison page with the Particle Loss Calculator, AeroSampleR and aerosolpy that says where each is ahead; site; the documentation check in CI.
- Release preparation; the owner tags.

Models: as in the CFD-craft documentation playbook.

### Track V — validation, in parallel

- **V1, code to code:** a user with IGOR Pro runs a handful of declared cases in the Particle Loss Calculator; results are compared. This tests the implementation, not the physics.
- **V2, experiment:** a partner builds one designed inlet and rides it with a research-grade instrument at several speeds. The protocol, the quantity and the tolerance are fixed before the data are opened. **Gate G3.**
- **V3, optional:** one inlet in a body-fitted flow solver with particle tracking as a further independent check.

## 6. Verification protocol

1. **Primitives:** reference tables and an independent package.
2. **Each formula:** its published points, its limits, monotonic behavior where the physics requires it.
3. **System:** composition rules, conservation of the particle count through a flow split, code-to-code comparison.
4. **Search:** winners re-evaluated cold; constraints re-checked; warnings counted in the result.
5. **Experiment:** track V only. Agreement with another program or with published formula values is numerical agreement, not validation of an inlet on a bike.

## 7. Order and size

```
I0 → I1 → I2 → I3 → I4 → I5 → I6 → I7
               V1 after I3;  V2 after I6
```

About 13 agent days to release 0.1.0, of which the first four (I0 to I2) already answer the practical question for one forward-facing inlet.

## 8. Risks

- **The formulas do not fit a real low-cost sensor.** They hold for thin-walled tubes. A housing slot is outside them. The tool must refuse such designs, and the companion document says to add a defined inlet.
- **Formula sources disagree.** I0 exists for this. The registry records the choice.
- **False precision.** Empirical correlations carry scatter of their own. Results state the validity status of every formula and avoid more digits than the sources support.
- **Little novelty.** The value is engineering: automation, validity checks, moving platforms, design search. If I0 finds an existing open tool that does this, stop.
- **No experimental partner.** Without track V2 the tool stays "verified against published formulas".

## 9. Open decisions for the owner

1. Is `inlet-craft` the name?
2. Start I0 now? It writes no code and decides whether the project is worth a repository.
3. Create the repository after G0, or keep the work in a tutorial of another repository until G1?
4. Rust core as in the other crafts, or Python only for a first version? The formulas are cheap either way; Rust keeps the ecosystem uniform and the search fast.
5. Is there a partner for track V2: a bike, a reference instrument and a few rides?

## 10. First handoff

> Do I0 for inlet-craft: build the formula registry for aerosol aspiration, probe transmission and transport losses from the original papers and the standard reference chapter. For each formula record equation, units, validity range, assumptions, one published number to reproduce and one limit. Record where sources differ and decide with a reason.
>
> Do not read or port code of the Particle Loss Calculator. Check the licenses of AeroSampleR and aerosolpy for use as test oracles. Search once more for an existing open tool for moving-platform inlets, and stop and report if one exists. Write no solver code.

## 11. Sources

- [Particle Loss Calculator, paper (Atmos. Meas. Tech. 2009)](https://amt.copernicus.org/articles/2/479/2009) and [download page with license](https://www.mpic.de/4230607/particle-loss-calculator-plc)
- [Aerosol Transport in Sampling Lines and Inlets, chapter 6 of Aerosol Measurement](https://doi.org/10.1002/9781118001684.ch6)
- [Aerosol sampling lecture slides with the aspiration and transport formulas (OSTI)](https://www.osti.gov/servlets/purl/1675150)
- [AeroSampleR reference manual](https://archive.linux.duke.edu/cran/web/packages/AeroSampleR/refman/AeroSampleR.html)
- [aerosolpy](https://github.com/DominikStolzenburg/aerosolpy)
- [Modeling inertial deposition in complicated flow systems (mentions AeroCalc and the Particle Loss Calculator)](https://egusphere.copernicus.org/preprints/2024/egusphere-2024-1242/egusphere-2024-1242.pdf)
- Project documents: `claude/inlet-craft-sensor-unit-design-2026-11-14.md`, `claude/cfd-craft-for-schwarmmessrad-2026-11-14.md`, `claude/thermo-craft-incremental-plan-v2-2026-11-03.md` (format)
