files='
ambient.frag
environment.frag
'

for file in $files
do
    glslc $file -o $file.spv
done