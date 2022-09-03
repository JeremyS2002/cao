files='
chain_blur.frag
split_gauss_blur.frag
full_gauss_blur.frag
ao_calc.frag
bloom_prefilter.frag
skybox.frag
tonemap_global.frag
tonemap_local.frag
'

for file in $files
do
    glslc $file -o $file.spv
done