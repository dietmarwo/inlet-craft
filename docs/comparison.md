# Comparison and license boundaries

| Tool | Where it is ahead / purpose | inlet-craft's addition | Verified comparison here |
|---|---|---|---|
| Particle Loss Calculator | Broad established correlations, interactive geometry and published experimental comparisons | Rust API/CLI/Python, strict inputs, moving-platform scenarios, time response and design search | Paper-equation implementation plus approximate virtual example; no IGOR software comparison |
| AeroSampleR0.3.0 | Established R sampling-line workflow, reporting, Pui/Zhang bend options and calm-probe choices | Variable-diameter explicit contractions and mobile scenarios/search | Public API export adapter supplied; R/package comparison not run in this environment |
| aerosolpy | Aerosol mechanics and diffusional survival, broader aerosol kinetics package | Whole inlet/system composition, moving-air aspiration and design search | Installed package grid for slip, diffusion and tube diffusional survival; different correlations explained |
| AeroCalc | Many spreadsheet formulas and teaching utility | Versioned scriptable cases, diagnostics and reproducible studies | No spreadsheet oracle executed |

The comparison is about workflows, not an assertion that inlet-craft is a more accurate physical model. PLC's validation is its authors' evidence; it does not transfer to this reimplementation. AeroSampleR has mechanisms and reporting choices inlet-craft does not provide, and aerosolpy covers broader aerosol science.

PLC paper is CC BY 3.0; PLC software has a separate restrictive license and was not inspected. First-party code is MIT. Vendored ecosystem crates are MIT. aerosolpy(MIT) is only an installed comparison dependency; AeroSampleR(GPL-3) is optional and called externally. Neither oracle's code is copied into this library.

Primary links: [PLC paper](https://doi.org/10.5194/amt-2-479-2009), [Brockmann standard-reference notes](https://www.osti.gov/servlets/purl/1675151), [AeroSampleR CRAN manual](https://cran.r-project.org/web/packages/AeroSampleR/refman/AeroSampleR.html), [aerosolpy](https://github.com/DominikStolzenburg/aerosolpy). Source access and limits are recorded in the formula registry.
