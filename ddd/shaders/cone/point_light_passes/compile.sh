files='
base.frag
shadow.frag
subsurface.frag
'

for file in $files
do
    glslc $file -o $file.spv
done