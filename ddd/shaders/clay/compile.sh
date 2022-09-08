files='
flat.frag
flat.vert
smooth.frag
smooth.vert
solid.frag
solid.vert
'

for file in $files
do
    glslc $file -o $file.spv
done