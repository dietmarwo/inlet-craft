# Designing a sensor unit for mobile air measurements, and how inlet-craft would help

Date: 14 November 2026. Written for a project that measures fine dust, ozone and asphalt temperature from cargo bikes (SchwarmMessRad, RWTH Aachen). `inlet-craft` is a proposed tool; nothing of it exists yet. Its plan is `claude/inlet-craft-incremental-plan-2026-11-14.md`.

## In short

- A reading from a moving sensor passes a chain: riding wind, inlet, tubing, a buffer volume, the sensor's optics or chemistry, its response time, and the position stamp. Each link can bias the value, and several links depend on speed.
- For fine particles (PM2.5) the inlet matters little. For coarse particles (up to PM10) it matters a lot at riding speed: a simple forward-facing tube can oversample 10 µm particles by a factor of two or more, and the factor changes with speed.
- The largest errors of low-cost optical sensors are not aerodynamic: humidity, a weak response to coarse particles, and riding wind disturbing the sensor's fan flow. Only co-location tests settle these.
- Published formulas predict inlet and tubing effects in minutes. The existing tools for them are an IGOR Pro program with a restrictive license, an R package for sampling lines in still air, and spreadsheets. None is built for moving platforms, automation or optimization.
- `inlet-craft` would be that tool: the published formulas as an open library with strict units and validity checks, plus a design search over the fleet's speeds and crosswinds.

## 1. The measurement chain

| Link | What can go wrong | Depends on speed | What settles it |
|---|---|---|---|
| Position on the bike | Wake of rider and box, spray and dust from the front wheel, heat from sun-lit surfaces | Yes | Smoke or tuft tests; a flow simulation only if in doubt |
| Inlet | Too many or too few coarse particles enter (aspiration); losses at the inlet wall | Strongly | Formulas (inlet-craft); wind tunnel |
| Tubing, bends | Coarse particles settle or impact; very small ones diffuse to the wall | Weakly | Formulas (inlet-craft) |
| Buffer volume | Smears the signal in time; some settling | No | Formulas (inlet-craft) |
| Sensor fan and housing | Riding wind changes the flow through the sensor | Yes | Shielding by design; tests |
| Detection | Optical response depends on particle size, material and humidity | No | Co-location with a reference; humidity correction |
| Time response | A slow chain blurs a hotspot over tens of meters | Yes | Formulas (inlet-craft); step tests |
| Position stamp | Delay between air entering and value logged; GNSS errors between buildings | Yes | Delay correction; tests |

## 2. Particulate matter

### 2.1 What low-cost optical sensors can and cannot do

- **Fine particles are their domain.** The common small sensors with a laser and a photodiode are reported as generally effective for PM2.5. Above about 0.5 µm they report nearly the same size distribution whatever the aerosol, and one evaluation calls them incapable of measuring the coarse mode from 2.5 to 10 µm. A detailed study of the Plantower PMS5003 found that it works like an imperfect particle counter and that its PM10 output does not respond to windblown dust.
- **Larger optical particle counters do better for PM10.** The Alphasense OPC-N3 has a much higher flow (0.28 L/min sample flow, 5.5 L/min in total) and has been used on a bicycle trailer in a published mobile campaign.
- **Humidity.** Water uptake changes particle size and refractive index. For hygroscopic aerosol, relative humidity is described as the largest source of uncertainty of optical sensors. Reference instruments measure dry; low-cost sensors measure at ambient humidity.

Consequence for design: decide early whether the project needs PM10 or only PM2.5. PM10 from a moving bike needs a better sensor and a designed inlet. PM2.5 needs mostly humidity handling and good co-location practice.

### 2.2 Riding wind and the sensor itself

The wind a moving sensor feels is called apparent wind. Published findings differ, which is itself the lesson:

- A laboratory study of three low-cost sensor models found different sensitivity to apparent wind, depending on how each sensor was positioned and exposed to the airflow.
- For the Snifferbike kit, each additional km/h of riding speed added about 0.03 to 0.04 µg/m³ to the standard deviation of PM2.5.
- The Smart-P bicycle monitor reported that its measurement was not sensitive to bicycle speed.

So the effect is real, differs by sensor and housing, and can be designed down. The design rule that follows: do not let riding wind reach the sensor's own inlet and outlet directly. Either shield both openings equally, or feed the sensor from a calm buffer volume (pattern B below).

### 2.3 Inlet physics at riding speed

Whether a particle follows the air into an inlet depends on its Stokes number: the particle's relaxation time times the air speed, divided by the inlet diameter. Below about 0.05 particles follow the air; near 1 they do not.

Stokes numbers for a forward-facing inlet (unit-density spheres, 20 °C):

| Inlet diameter | Speed | 2.5 µm | 5 µm | 10 µm |
|---|---|---:|---:|---:|
| 4 mm | 10 km/h | 0.014 | 0.055 | 0.22 |
| 4 mm | 20 km/h | 0.028 | 0.11 | 0.43 |
| 4 mm | 25 km/h | 0.036 | 0.14 | 0.54 |
| 8 mm | 10 km/h | 0.007 | 0.028 | 0.11 |
| 8 mm | 20 km/h | 0.014 | 0.055 | 0.22 |
| 8 mm | 25 km/h | 0.018 | 0.069 | 0.27 |

Particles of 2.5 µm stay below 0.05: the inlet hardly matters for PM2.5. Particles of 10 µm are in the range where it does.

If the air speed inside the inlet is lower than the riding speed, air streams around the inlet but heavy particles fly straight in: they are oversampled. The classic correlation for a thin-walled tube facing the flow (Belyaev and Levin) gives these ratios of sampled to true concentration:

| Inlet, sample flow | Speed | 2.5 µm | 5 µm | 10 µm |
|---|---|---:|---:|---:|
| 4 mm, 1 L/min | 10 km/h | 1.03 | 1.12 | 1.36 |
| 4 mm, 1 L/min | 20 km/h | 1.18 | 1.61 | 2.54 |
| 4 mm, 2 L/min | 10 km/h | 1.00 | 1.01 | 1.02 |
| 4 mm, 2 L/min | 20 km/h | 1.07 | 1.22 | 1.55 |
| 8 mm, 5 L/min | 10 km/h | 1.01 | 1.04 | 1.14 |
| 8 mm, 5 L/min | 20 km/h | 1.07 | 1.25 | 1.76 |

These numbers are an illustration, computed by hand from the published formula. Values at a Stokes number below 0.05, which includes all those for 2.5 µm, lie below the formula's stated validity range and are indicative only. Three things follow:

- **The bias changes with speed.** The same inlet reads 10 µm particles 36 % high at 10 km/h and 154 % high at 20 km/h. A hotspot on the map could be a downhill stretch.
- **Matching speeds is expensive.** A 4 mm inlet needs 2.1 L/min to match 10 km/h and 4.2 L/min to match 20 km/h; an 8 mm inlet needs four times as much. Small sensors draw far less.
- **The formulas assume a thin-walled tube.** The slot of a sensor housing is a blunt body, for which these formulas do not hold. A defined inlet makes the unit calculable.

### 2.4 Design patterns

| Pattern | How it works | For | Against |
|---|---|---|---|
| A. Sensor in a shielded housing | Openings behind baffles, both at the same pressure | Simple, light, fast response | Not calculable; coarse particles lost at baffles in an unknown way; needs tests at every speed |
| B. Forward-facing tube, bypass flow, buffer volume | A pump or fan draws a larger flow through a tube; the sensor samples from a small calm volume | Calculable end to end; the sensor sees still air | Needs a pump; aspiration bias varies with speed unless the flow is high; the volume slows the response |
| C. Vertical or omnidirectional inlet | Tube at right angles to the wind, with a cap against rain | Independent of wind direction | At riding speed coarse particles are undersampled; published formulas for this angle hold only up to a Stokes number of 0.2 |
| D. Speed-controlled flow | The pump follows the riding speed to keep the inlet matched | Smallest aspiration bias | Complex; flow range of a factor of three; the sensor still needs a constant flow behind it |

For PM2.5 only, pattern A with careful shielding and thorough tests is defensible. For PM10, pattern B is the one whose behavior can be predicted and optimized before anything is built.

### 2.5 Time response and spatial resolution

Every volume in the chain mixes the sample. A buffer volume V at flow Q has the time constant V/Q:

| Buffer volume | Flow | Time constant | Distance at 18 km/h |
|---|---|---:|---:|
| 20 mL | 1 L/min | 1.2 s | 6 m |
| 50 mL | 1 L/min | 3.0 s | 15 m |
| 50 mL | 2 L/min | 1.5 s | 8 m |
| 100 mL | 5 L/min | 1.2 s | 6 m |

Add the travel time through the tube, the sensor's own response time and its averaging interval. The sum decides whether a map can resolve a junction or only a street. The travel time is also a position offset: the value logged now belongs to air that entered some meters back, and the offset grows with speed. It can be corrected if it is known.

### 2.6 Where to mount

- Ahead of the rider and the cargo box, in air the bike has not disturbed.
- Away from the front wheel's spray and dust, and not directly above it.
- At a stated height, the same on every bike. About one meter is between exhaust-pipe and breathing height.
- Out of direct sun for the housing, and away from brakes and motor.
- Check with smoke or wool tufts at riding speed before anything else. A flow simulation of bike and rider is worth it only if candidate positions remain in doubt.

### 2.7 Humidity

- Measure relative humidity and temperature next to the particle sensor, in the same air.
- Published corrections use the hygroscopic growth of the particles; their parameters vary with season and place.
- A mild heater on the inlet lowers the humidity in the sample. It costs power and can evaporate part of the particles; decide it together with the reference method.
- Flag or drop values above a stated humidity; fog is not fine dust.

### 2.8 Calibration and quality assurance

- **Stationary co-location** of every unit next to a reference station, before, during and after the campaign.
- **On-bike co-location:** a research-grade instrument on the same bike for some rides, at several speeds. This is the only direct test of the speed dependence.
- **Bike against bike:** two units on one bike, and comparisons when bikes meet or pass a station.
- **Flow and zero checks:** flow through the unit at standstill and in wind; a filter on the inlet must give zero.
- **Unit-to-unit spread:** published values for one bicycle monitor are 6 to 9 % between units.

## 3. Ozone

- **Two sensors, not one.** Typical low-cost electrochemical ozone sensors respond to nitrogen dioxide as well. The usual solution pairs an oxidant sensor with a sensor for nitrogen dioxide only and takes the difference. The ozone estimate from such a pair is less accurate and less precise than the nitrogen dioxide one.
- **Next to traffic** nitrogen dioxide is high and variable, which is where the difference is weakest.
- **Temperature and humidity** change the signal; log both at the cell.
- **Response time** of the cells and of any tubing is seconds to tens of seconds; the same distance arithmetic as in 2.5 applies.
- **Flow over the cells** should be steady and shielded from riding wind, as for the particle sensor.

## 4. Asphalt surface temperature

This is radiometry, not fluid dynamics.

- **Emissivity.** An infrared thermometer converts radiation to temperature with an assumed emissivity. One street study used 0.97 for dry pavements and 0.98 for wet ones. Emissivity depends on the viewing direction and the sensor's spectral band; measurements on site are the careful route.
- **Reflected radiation.** The sensor also sees sky and buildings reflected by the surface. The correction needs a stated background temperature.
- **Spot size** is height times twice the tangent of half the field of view: a 10° sensor at 0.6 m sees a 10 cm spot, a 90° sensor a 1.2 m spot, which may include the bike's own wheel or shadow.
- **Shadow.** Aim ahead or to the side so that the spot is not in the shadow of bike and rider, and record sun or shade of the surface.
- **Sensor temperature.** The sensor body heats in the sun and cools in riding wind. Shade it and let it equilibrate; check against a contact thermometer.
- **Speed.** The response time of the sensor and the speed give the length over which the reading is averaged.

## 5. What all sensors share

- One clock for all channels, and a logged delay per channel.
- Speed, heading and position with every value; position quality drops between tall buildings.
- Vibration, rain and dust protection that does not block the inlet.
- Metadata that travels with the data: unit, sensor serials, inlet design, firmware, calibration version.
- Wind from a station that represents roof-level flow, to interpret the rides.

## 6. How inlet-craft would be used

The tool is planned, not built. The interface below is the intended one.

### 6.1 What it computes

For a described inlet system and operating conditions, by particle size:

- aspiration efficiency at the inlet, for the apparent wind speed and its angle to the inlet axis;
- transmission losses in the inlet;
- transport efficiency through tubes, bends, contractions and a buffer volume, by diffusion, settling and inertia;
- the overall efficiency curve, and what it does to PM2.5 and PM10 for a stated size distribution;
- travel time, time constant and the resulting distance resolution and position offset;
- for every formula used: its source, and whether the conditions are inside its stated validity range.

A result outside a validity range is still reported, with a coded warning that names the quantity and the range. A system with a part no formula covers, such as a blunt housing slot, is rejected with a diagnostic and not silently approximated.

### 6.2 Describing a unit

One JSON file, in the style of the other craft tools: units in the field names, unknown fields rejected. A sketch:

```json
{
  "schema": "inlet-craft.case/0.1",
  "air": {"temperature_k": 293.15, "pressure_pa": 101325.0},
  "platform": {"speed_m_s": [2.8, 4.2, 5.6, 6.9], "crosswind_m_s": [0.0, 2.0]},
  "particles": {"aerodynamic_diameter_m": [1e-6, 2.5e-6, 5e-6, 1e-5]},
  "elements": [
    {"kind": "probe", "inner_diameter_m": 0.004, "angle_to_heading_deg": 0.0},
    {"kind": "tube", "length_m": 0.15, "inner_diameter_m": 0.004, "inclination_deg": 0.0},
    {"kind": "bend", "angle_deg": 90.0, "radius_m": 0.02},
    {"kind": "buffer", "volume_m3": 2e-5}
  ],
  "flow": {"total_m3_s": 3.3e-5, "sensor_m3_s": 4.7e-6},
  "sensor": {"response_time_s": 1.0, "sampling_interval_s": 1.0}
}
```

### 6.3 Commands

| Command | Purpose |
|---|---|
| `inlet-craft describe` | Schemas, implemented formulas with sources and validity ranges, diagnostic codes |
| `inlet-craft check case.json` | Reject wrong or uncovered designs before computing |
| `inlet-craft evaluate case.json` | Efficiency by size and mechanism, time response, warnings |
| `inlet-craft sweep case.json` | The same over speeds, crosswinds or a design parameter |
| `inlet-craft optimize study.json` | Search design parameters for stated objectives and constraints |
| `inlet-craft verify` | Reproduce the published reference numbers the tool is tested against |

The same functions are available from Python.

### 6.4 A design study for the bike

1. **State the goal.** For example: PM10 within ±15 % of an ideal PM10 inlet between 10 and 25 km/h and up to 3 m/s crosswind; distance resolution under 15 m at 18 km/h; total flow under 3 L/min.
2. **Describe candidates** of patterns B and C as cases.
3. **Evaluate and sweep.** See which mechanism dominates for which size and speed, and which warnings appear.
4. **Optimize.** Let the search vary inlet diameter, total flow, tube length, bend radius and buffer volume. The objective is the worst case over the speed and crosswind distribution, not the value at one speed. If no design meets the goal, the result says so.
5. **Build one or two candidates** and test them: flow, zero, step response, then on-bike co-location at several speeds.
6. **Feed the tests back.** The measured speed dependence either confirms the model or shows a part of the unit the formulas do not cover.

### 6.5 What it will not tell

- How the sensor's optics respond to particle size, material and humidity.
- How riding wind acts on a sensor fan inside a housing.
- The flow around bike and rider.
- Anything about gases beyond travel time and mixing, or about infrared thermometry.

## 7. Existing tools, and why a new one

| Tool | Covers | Limits for this use |
|---|---|---|
| Particle Loss Calculator (Max Planck Institute for Chemistry) | Aspiration in moving and calm air, at an angle; diffusion, settling, turbulent deposition, bends, contractions; validated against tube experiments | Runs only in IGOR Pro; the license allows changes to the code only after consulting the licensor; interactive; thin-walled probes of constant diameter |
| AeroSampleR (R package) | Sampling lines: probe in calm air, tubes, bends | One tube diameter per system; no moving-air inlet; built for sampling lines in facilities |
| aerosolpy (Python package) | Basic aerosol mechanics: slip correction, diffusion coefficients, diffusional losses | No inlet or system model |
| AeroCalc (spreadsheet) | Many single formulas | Manual |

The formulas themselves are published and free to implement from the original papers. What is missing is an open, scriptable implementation for moving platforms that checks validity ranges, composes a whole unit, computes time response and searches designs. I found no such tool; one cited paper mentions "an open-source particle loss calculator" that I could not identify.

## 8. Sources

- [SchwarmMessRad, ISAC, RWTH Aachen](https://www.isac.rwth-aachen.de/cms/isac/forschung/projekte/verkehrstechnik-laufende-projekte/~bmgdfq/schwarmmessrad/?lidx=1)
- [Particle Loss Calculator, paper (Atmos. Meas. Tech. 2009)](https://amt.copernicus.org/articles/2/479/2009) and [download page with license](https://www.mpic.de/4230607/particle-loss-calculator-plc)
- [Aerosol Transport in Sampling Lines and Inlets, chapter 6 of Aerosol Measurement](https://doi.org/10.1002/9781118001684.ch6)
- [Aerosol sampling lecture slides with the aspiration and transport formulas (OSTI)](https://www.osti.gov/servlets/purl/1675150)
- [AeroSampleR reference manual](https://archive.linux.duke.edu/cran/web/packages/AeroSampleR/refman/AeroSampleR.html)
- [aerosolpy](https://github.com/DominikStolzenburg/aerosolpy)
- [Effect of apparent wind on low-cost PM sensors, laboratory study](https://iris.unica.it/retrieve/a00972f8-7268-4a45-924e-0ac5a94894d1/Metro_2024_Air_Quality_Wind%20post-print-cover.pdf)
- [Snifferbike: field calibration and uncertainty of citizen-operated mobile sensors (NILU)](https://nilu.com/?p=51149)
- [Smart-P, a low-cost portable PM2.5 monitor for bicycles (Sensors 2022)](https://resources.environment.yale.edu/xlee-downloads/files/chen_sensors_2022.pdf)
- [Physical-optical model of the Plantower PMS5003](https://pmc.ncbi.nlm.nih.gov/articles/PMC11236278/)
- [Alphasense OPC-N3 and Plantower PMS5003 for PM10](https://par.nsf.gov/servlets/purl/10422410)
- [Mobile PM monitoring with an OPC-N3 on a bicycle trailer (KIT)](https://publikationen.bibliothek.kit.edu/1000181048/181655668)
- [Towards a hygroscopic growth calibration for low-cost PM2.5 sensors](https://amt.copernicus.org/articles/17/1051/2024/)
- [Efficacy of paired electrochemical sensors for measuring ozone](https://stacks.cdc.gov/view/cdc/216345/cdc_216345_DS1.pdf)
- [Infrared pavement temperatures in a street, with emissivities](https://pdf.arxiv.org/pdf/1412.2165) and [in-situ emissivity measurement for road surfaces](https://meetingorganizer.copernicus.org/EGU2018/EGU2018-1520.pdf)
- [A bicycle-based field measurement system for thermal exposure](https://researchconnect.buffalo.edu/en/publications/a-bicycle-based-field-measurement-system-for-the-study-of-thermal/)
