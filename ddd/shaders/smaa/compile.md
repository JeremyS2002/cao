## Compile all shaders
```./compile.sh```
## Compile single shader
```glslc file -DPRESET -o PRESET/file.spv```
eg.
```glslc edge_detect.vert -DSMAA_PRESET_MEDIUM=1 -o SMAA_PRESET_MEDIUM/edge_detect.vert.spv```

where preset is one of
- SMAA_PRESET_LOW
- SMAA_PRESET_MEDIUM
- SMAA_PRESET_HIGH
- SMAA_PRESET_ULTRA
