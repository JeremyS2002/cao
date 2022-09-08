files='
single_base.frag
single_shadow.frag
single_subsurface.frag
depth_calc.comp
tile_assign.comp
tile_base.comp
'

for file in $files
do
    glslc $file -o $file.spv
done