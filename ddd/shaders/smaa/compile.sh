#!/bin/bash

presets='SMAA_PRESET_LOW SMAA_PRESET_MEDIUM SMAA_PRESET_HIGH SMAA_PRESET_ULTRA'
files='
edge_detect.vert
depth_edge_detect.frag
luma_edge_detect.frag
color_edge_detect.frag
blending_weight.vert
blending_weight.frag
neighborhood_blending.vert
neighborhood_blending.frag
'

for preset in $presets
do
    for file in $files
    do
        glslc $file -D$preset=1 -o $preset/$file.spv
    done
done
