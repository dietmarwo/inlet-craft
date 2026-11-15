"""Independent high-precision equation arithmetic, not measured validation.
Run to regenerate fixtures. Published approximate points are kept separate.
Requires mpmath only when regenerating; verify itself is pure Rust.
"""
import json
from pathlib import Path
import mpmath as m
m.mp.dps=60
rows=[]
def add(id,formula,args,value,kind='independent_equation_arithmetic',source='Brockmann notes SAND2010-7446P / formula-registry.md',atol=1e-13,rtol=1e-11):
    rows.append(dict(id=id,formula=formula,inputs=list(map(float,args)),expected=float(value),absolute_tolerance=atol,relative_tolerance=rtol,evidence_kind=kind,source=source))
def viscosity(t):return m.mpf('1.716e-5')*(t/m.mpf('273.15'))**m.mpf('1.5')*(m.mpf('273.15')+111)/(t+111)
t=m.mpf('293.15');mu=viscosity(t);lam=mu/101325*m.sqrt(m.pi*m.mpf('287.05')*t/2)
def slip(d):
    kn=2*lam/d
    return 1+kn*(m.mpf('1.142')+m.mpf('.558')*m.exp(-m.mpf('.999')/kn))
add('sutherland-standard','air.viscosity',[273.15],m.mpf('1.716e-5'),'reference_parameter','Sutherland reference viscosity at273.15K',1e-18)
add('mean-free-path','air.mean_free_path',[t,101325],lam,atol=1e-19)
for d in map(m.mpf,['1e-8','1e-7','1e-6','1e-5']):
    add('slip-'+str(d),'particle.slip',[d],slip(d))
    add('settling-'+str(d),'particle.settling',[d,1000],1000*d*d*slip(d)/(18*mu)*m.mpf('9.80665'))
    add('diffusion-'+str(d),'particle.diffusion',[d],m.mpf('1.380649e-23')*t*slip(d)/(3*m.pi*mu*d),atol=1e-20)
s=m.mpf('.1');r=m.mpf(2);k=2+m.mpf('.617')/r
add('belyaev-r2','aspiration.belyaev_levin',[s,r],1+(r-1)*(1-1/(1+k*s)))
add('isokinetic','aspiration.belyaev_levin',[.1,1],1,'analytic_identity')
st=s*m.exp(m.mpf('.022')*30);ratio=m.mpf('1.5');kk=2+m.mpf('.617')/ratio
add('durham-30','aspiration.durham_lundgren',[s,ratio,30],1+(ratio*m.cos(m.pi/6)-1)*(1-1/(1+kk*st))/(1-1/(1+m.mpf('2.617')*st))*(1-1/(1+m.mpf('.55')*st*m.exp(m.mpf('.25')*st))))
add('hangal-90','aspiration.hangal_willeke',[.1,1,90],m.mpf('.7'))
# Exponent R^-0.5 (primary eq 17-18): pins the R dependence that R=1 cannot.
add('hangal-90-r2','aspiration.hangal_willeke',[.1,2,90],1+(2*m.cos(m.pi/2)-1)*3*m.mpf('.1')**(1/m.sqrt(2)),source='Hangal & Willeke 1990a eq 18 (primary source), arithmetic')
q=m.mpf('.01');s=m.mpf('.1')
add('calm-horizontal','aspiration.grinshpun_calm',[s,q,90],m.exp(-4*s**(1+m.sqrt(q))/(1+2*s)))
add('liu-r2','transmission.facing',[s,2],(1+1/(1+m.mpf('2.66')*s**(-m.mpf(2)/3)))/(1+1/(1+m.mpf('.418')/s)))
add('hangal-rhalf','transmission.facing',[s,.5],m.exp(-75*(m.mpf('.09')*(s*(2-1))**m.mpf('.3'))**2))
angle=m.pi/6;alpha=12*((1-m.mpf(30)/90)-m.exp(-30))*m.pi/180
iv=m.mpf('.09')*(s*m.cos(angle)*(2-1))**m.mpf('.3');iw=s*m.sqrt(m.mpf('.5'))*m.sin(angle-alpha)*m.sin((angle-alpha)/2)
add('oblique-transmission','transmission.hangal_willeke_angle',[s,.5,30,0],m.exp(-75*(iv+iw)**2))
xi=m.mpf('.001');add('holman','tube.diffusion_holman',[xi],m.exp(-xi*(m.mpf('3.66')+m.mpf('.2672')/(xi+m.mpf('.10079')*xi**(m.mpf(1)/3)))))
add('friedlander','tube.diffusion_friedlander',[xi,8000,10000],m.exp(-xi*m.mpf('.0118')*8000**(m.mpf(7)/8)*10000**(m.mpf(1)/3)))
a=m.mpf('.2');add('heyder','tube.settling_heyder',[a],1-2/m.pi*(2*a*m.sqrt(1-a**(m.mpf(2)/3))-a**(m.mpf(1)/3)*m.sqrt(1-a**(m.mpf(2)/3))+m.asin(a**(m.mpf(1)/3))))
add('schwendiman','tube.settling_schwendiman',[.2],m.exp(-m.mpf('.8')/m.pi))
re=m.mpf(8000);u=m.mpf(10);diam=m.mpf('.01');length=m.mpf('.2');flow=m.pi*diam**2/4*u
vt=(m.mpf('.0006')*(m.mpf('.0395')*s*re**m.mpf('.75'))**2+m.mpf('2e-8')*re)*u/(m.mpf('5.03')*re**m.mpf('.125'))
add('lee-gieseke','tube.inertia_lee_gieseke',[s,re,u,diam,length,flow],m.exp(-m.pi*diam*length*vt/flow))
add('pui-half','bend.pui_laminar',[.171,m.pi/2],.5,'analytic_identity')
add('pui-turbulent','bend.pui_turbulent',[.1,m.pi/2],m.exp(-m.mpf('2.823')*m.mpf('.1')*m.pi/2))
add('muyshondt-factor2','contraction.muyshondt',[.1,.25,45],1/(1+(2*m.mpf('.1')*(1-m.mpf('.25'))/(m.mpf('3.14')*m.exp(-m.mpf('.0185')*45)))**m.mpf('1.24')))
for st,expected in [(.05,1),(.16,.5),(.32,.1)]:
    add('published-bend-guide-'+str(st),'bend.pui_laminar',[st,m.pi/2],expected,'published_approximate_adjacent_geometry','Brockmann SAND2010-7446P p17: approximate curvature8 guide; not exact Pui fit geometry',.06,0)
Path('fixtures/reference.json').write_text(json.dumps(rows,indent=2)+'\n')
print(f'{len(rows)} fixtures')
