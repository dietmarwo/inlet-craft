# Optional installed AeroSampleR0.3.0 oracle export; GPL-3 package not bundled.
# Run with Rscript scripts/export_aerosampler.R OUTPUT.csv.
suppressPackageStartupMessages(library(AeroSampleR))
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=1) stop('one output CSV path required')
df <- particle_dist(discrete_vals=c(.1,.3,1,2.5,5,10))
params <- set_params_1(D_tube=.4,Q_lpm=2,T_C=20,P_kPa=101.325)
df <- set_params_2(df,params)
df <- tube_eff(df,params,L=.2,angle_to_horiz=0,elnum=1)
df <- bend_eff(df,params,method='Pui',bend_angle=90,bend_radius=.02,elnum=2)
write.csv(df,args[1],row.names=FALSE)
