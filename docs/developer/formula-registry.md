# Formula registry and evidence ledger

Date: 2026-11-14. Implementation from published equations, **not PLC program code**.

## Sources and decisions

- B: John E. Brockmann, *Aerosol Sampling and Transport*, Sandia lecture, OSTI 1675150, https://www.osti.gov/servlets/purl/1675150 (standard-reference author; slides 5,10,13,14,20,26,30,34,38,42). Download and equation images inspected.
- W: von der Weiden, Drewnick & Borrmann (2009), https://doi.org/10.5194/amt-2-479-2009, pp.482–485,492. This is a published secondary compilation; original references are attributed below. The software was not downloaded or inspected.
- C: Muyshondt, McFarland & Anand (1996), https://doi.org/10.1080/02786829608965364. Abstract specifies upstream velocity / downstream diameter for contraction Stokes number.
- Allen & Raabe (1985), https://doi.org/10.1080/02786828508959055, slip correlation as reproduced in W.
- Sutherland viscosity: 1.716e-5 Pa s at 273.15 K, S=111 K; ideal dry air R=287.05 J/(kg K); kinetic mean free path lambda=(mu/p)*sqrt(pi*R*T/2). This choice differs slightly from fixed mean-free-path tables and is declared.

## Implemented equations

All lengths m, velocities m/s, flow m3/s, temperatures K, pressures Pa; radians in trigonometry unless explicitly marked degrees. Physical diameter dp, density rho_p and dynamic shape chi determine tau. Aerodynamic diameter da solves da² Cc(da) = (rho_p/1000/chi) dp² Cc(dp). Do not multiply density twice for aerodynamic input.

| ID / attribution | Equation / definition | Source / validity / assumptions | Verification anchor and limit |
|---|---|---|---|
| air.sutherland | mu=mu0*(T/T0)^1.5*(T0+S)/(T+S), rho=p/(R*T) | Sutherland; dry ideal air, engineering range 200–350 K, 20000–120000 Pa | reference mu(T0)=1.716e-5; p doubles rho, halves lambda |
| particle.allen_raabe | Kn=2lambda/dp; Cc=1+Kn*(1.142+.558*exp(-.999/Kn)) | W p482; solid-sphere fit, original Kn .03–7.2; mean-free-path convention differs slightly | Cc tends to 1 as Kn tends to zero |
| particle.stokes | tau=rho_p*dp²*Cc/(18mu*chi); v=tau*g; D=kB*T*Cc/(3*pi*mu*dp*chi) | B slide5; Stokes drag; settling Rep<.1 | standard 1um unit sphere ~3.5e-5 m/s; tau*g=v |
| aspiration.belyaev_levin | A=1+(R-1)*(k*St/(1+k*St)), k=2+.617/R | B slide10; W eq5; St=tau*U0/d; 0.05<=St<=2.03, .17<=R<=5.6 | arithmetic fixture R=2,St=.1; A=1 at R=1, A tends R at infinite St |
| aspiration.durham_lundgren | s=St*exp(.022*theta_deg); A=1+(R*cos(theta)-1)*[(1-1/(1+(2+.617/R)*s))/(1-1/(1+2.617*s))]*(1-1/(1+.55*s*exp(.25*s))) | W eq6; 0<theta<=60deg, .02<=St<=4, .5<=R<=2 | arithmetic point and zero-St limit; empirical jump at 0deg is reported |
| aspiration.hangal_willeke | A=1+(R*cos(theta)-1)*3*St^sqrt(1/R) | W eq7, Hangal & Willeke (1990a); exponent sqrt(U/U0) corrected 2026-11-15, see below; 61<=theta<=90deg, .02<=St<=.2, .5<=R<=2 | theta=90,R=1,St=.1 gives .7; R=2 arithmetic point; at 90deg A falls as R rises; 60–61deg interval is uncovered |
| aspiration.grinshpun_calm | q=v/U; s=tau*U/d; A=q*cos(phi)+exp(-4*s^(1+sqrt(q))/(1+2*s)) | B slide20, W eq8; phi angle to upward-facing vertical 0–90deg, .001<=q<=1, .001<=s<=100 | explicit use of inlet-speed St; small particle limit 1 |
| aspiration.grinshpun_slow | f=exp(-v/U0); A=A_moving*sqrt(1+delta)*f+A_calm*(1-f); delta=(v/U0)^2 for horizontal wind orthogonal to vertical settling | W eq9–10; automatic switch at U0<=1.5m/s; extension of blend below.5m/s is an implementation policy, reported; underlying correlation ranges still checked | exactly U0=0 uses calm formula; no divide-by-zero |
| transmission.liu | T=(1+(R-1)/(1+2.66*St^(-2/3)))/(1+(R-1)/(1+.418/St)) | B slide13, W eq12; 1<=R<=10, .01<=St<=100 | at R=1 and St=0 T=1; approach 1 at infinite St |
| transmission.hangal_willeke | Iv=.09*(St*(1/R-1))^.3 for super-isokinetic, else 0; T=exp(-75*Iv²) | B slide14, W eq13–14; .25<=R<=1, .02<=St<=4 | matched speed 1; .6075 coefficient retained, lecture rounds .61 |
| transmission.hangal_willeke_angle | Iv=.09*(St*cos(theta)*(1/R-1))^.3 if R<=1; alpha_deg=12*((1-theta_deg/90)-exp(-theta_deg)); Iw=sqrt(St*R)*sin(theta±alpha)*sin((theta±alpha)/2); T=exp(-75*(Iv+Iw)²) | W eq15–19; .02<=St<=4,.25<=R<=4,0<theta<=90; documented downward/upward branch | zero-St 1, signs explicit; gravitational deposition is in transport only |
| tube.diffusion_holman | xi=pi*D*L/Q; Sh=3.66+.2672/(xi+.10079*xi^(1/3)); T=exp(-xi*Sh) | W eq21–22; laminar Re<2000 | L=0 T=1; decreasing in length |
| tube.diffusion_friedlander | Sh=.0118*Re^(7/8)*Sc^(1/3); T=exp(-xi*Sh) | W eq21,23; turbulent Re>4000 | L=0 T=1 |
| tube.settling_heyder | a=.75*L*v*cos(inclination)/(d*U); T=1-(2/pi)*(2*a*sqrt(1-a^(2/3))-a^(1/3)*sqrt(1-a^(2/3))+asin(a^(1/3))) | B slide26, W eq24–25; a>=1 gives complete deposition (analytic domain); abs(v*sin(i)/U)<<1 checked against stated .1 policy | horizontal/vertical limits; saturation is analytic, not efficiency clamping |
| tube.settling_schwendiman | T=exp(-d*L*v*cos(inclination)/Q) | B slide26, W eq26–27; turbulent flow | zero length 1 |
| tube.inertia_lee_gieseke | vp=(.0006*(.0395*St*Re^.75)^2+2e-8*Re)*U/(5.03*Re^.125); T=exp(-pi*d*L*vp/Q) | W eq28–29, Re 4000–15600 (Lee/Gieseke original DOI 10.1016/0021-8502(94)90011-6); St=tau*U/d | L=0 T=1; use W version, not B's different capped Liu-Agarwal model |
| bend.pui_laminar | T=(1+(St/.171)^(.452*St/.171+2.242))^(-2*theta_rad/pi) | B slide38, W eq30; St=tau*U/d, curvature radius/(d/2) 5–30; fit measured at Re1000, curvature 5.6–5.7; generic curvature envelope 5–30 is not additional fitted-condition evidence | St=.171,theta=pi/2 gives .5; angle radians despite W's degrees sentence |
| bend.pui_turbulent | T=exp(-2.823*St*theta_rad) | B slide38, W eq31; Re>4000, curvature ratio 5–30 | zero angle 1; St=.1,theta=pi/2 arithmetic fixture |
| contraction.muyshondt | x=2*St*(1-Ao/Ai); T=1-1/(1+(x/(3.14*exp(-.0185*halfangle_deg)))^(-1.24)) | B slide42 / notes eq40; W eq32 omits factor 2; C St=tau*U_upstream/d_downstream; .001<=x<=100, 12–90deg | no contraction limit 1; preserves area/velocity convention |
| response.series | plug delay=sum(Vtube/Q); mixed tau=V/Q; sensor first-order tau; trailing boxcar T/2 mean, T²/12 variance | mass conservation / linear systems, explicitly modeled assumptions | two equal tanks y=1-exp(-t/tau)*(1+t/tau); split keeps concentration, partitions count |

## Conflicts resolved and uncovered parts

The B lecture gives a wider Belyaev range (.005–10, .2–5) than W; use W's narrower St range and retain diagnostics. B's Gormley–Kennedy diffusion is an independent alternative, not the chosen Holman formula. Bend angle uses radians (a 90deg bend at St=.171 has .5 penetration, not almost zero). Contraction St does **not** use downstream velocity. Calm St does **not** use wind speed. Angle 60–61deg and backward-facing wind are rejected. No smoothing at correlation boundaries. Apparent wind is horizontal. Effective misalignment satisfies cos(theta)=sin(phi)*cos(yaw), so a vertical inlet has theta 90deg. Slow-air delta uses the horizontal-wind/vertical-settling vector dot product (zero), an explicit geometric generalization of the scalar coplanar source. Arbitrary vertical wind and experimental validation of 3-D correlations are not claimed. The original scalar delta=(v/U0)*(v/U0+2*cos(theta+phi)) is correct only where those angles are coplanar.

Buffers have only residence time; a warning states that particle losses in them are unmodeled. Ideal well-mixed splits preserve concentration and track sensor/bypass particle-count fractions; no branch aspiration correlation is implied. Enlargements, blunt slots, shrouds, non-conductive electrostatics and thermophoresis have no implemented model.

**Correction 2026-11-15, aspiration.hangal_willeke.** The 0.1.0 candidate used the Stokes exponent sqrt(R) with R=U0/U. At 90deg the bracket R*cos(theta)-1 equals -1, so R enters only through that exponent, and sqrt(R) made aspiration of inertial particles *rise* when wind outruns the inlet velocity. That contradicts the inertial argument and the experimental literature, in which right-angle aspiration falls with St*R^(1/2) (Brixey, Evans & Vincent 2005, J. Aerosol Sci. 36:1144, https://stacks.cdc.gov/view/cdc/190534, summarizing the 1980s data; at 90deg sampling efficiency is approximately a function of St*sqrt(R)). The exponent sqrt(1/R)=sqrt(U/U0) restores that trend and halves the summed 60/61deg boundary jump against Durham–Lundgren over St .02–.2, R .5–2 (1.87 to 0.89). The printed W eq7 and the original article could not be re-read in this review (both were unreachable from the review environment), so the transcription source of sqrt(R) is unresolved; re-check against Hangal & Willeke (1990a) remains an open G0 item. A test pins the monotonic trend.

Brockmann written notes SAND2010-7446P, https://www.osti.gov/servlets/purl/1675151, independently confirm the equations and give approximate bend guide points (St=.05/.16/.32, penetration about 1/.5/.1, adjacent curvature 8 geometry). These are checked at a predeclared absolute .06 tolerance and labeled approximate, not precision validation. Contraction uses the primary standard-reference factor 2, unlike W. Validity diagnostic x is defined using the same factor 2; the exact original validation interval remains a provenance gap.

## Gate G0 and evidence honesty

**G0 is incomplete.** Not every original article was available in full text and not every equation has a source-published numerical point. `verify` distinguishes arithmetic/limit fixtures from genuine published curve/text comparisons. No calculated number is labeled a published measurement. W and B make a provisional literature-equation implementation possible; they do not satisfy the stronger original-source-only and every-formula-number conditions of G0. This gap is carried into G1. No release or experimental validation is claimed. Track V needs external data.

## License and existing-tool search

New implementation MIT; published mathematical equations are reimplemented with attribution. W paper CC BY 3.0. PLC software license is separate; no PLC code read, linked, copied or translated. aerosolpy is MIT and AeroSampleR GPL-3; optional installed-package subprocess oracle comparisons do not copy their code into this library. No legal opinion on redistribution of those packages is implied.

Search on 2026-11-14 found references calling PLC “open source”, but the citing article points back to von der Weiden (2009), not to a separately identified moving-platform library. AeroCalc is a spreadsheet; AeroSampleR and aerosolpy overlap but do not provide this system/search interface. This search is not proof of absence of all tools. Proceed with provisional implementation, not with a novelty claim.
