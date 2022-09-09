# gpu
Low (ish) level wrapper of vulkan. Manages creation and destruction of vulkan objects as well as hiding some of the more unpleasant parts of the vulkan api.

# examples
run from inside `cao/gpu/` (to avoid path errors in loading files)

|Name          |Command                             |Description                                                                               |
|--------------|------------------------------------|------------------------------------------------------------------------------------------|
|basic         |`cargo run --example basic`         |Create a vulkan context and device then print infomation about them                       |
|clear         |`cargo run --example clear`         |Create a swapchain and clear the texture in a solid color each frame                      |
|triangle      |`cargo run --example triangle`      |Draw a triangle, introduces shaders, graphics pipelines and buffers                       |
|push_constants|`cargo run --example push_constants`|Builds off triangle example, uses push constants to color the triange                     |
|depth         |`cargo run --example depth`         |Introduces depth testing to draw a rotating cube                                          |
|compute       |`cargo run --example compute`       |Introduces compute shaders to compute the collatz conjecture for multiple values at once  |
