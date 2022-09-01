files='
point.vert
shadow.frag
'

for file in $files
do
    glslc $file -o $file.spv
done