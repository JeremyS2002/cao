#version 450

struct Foo {
    mat4 matrix;
};

//layout(set = 0, binding = 0) uniform FooData {
//    Foo data;
//} u_foo_data;

void main() {
    Foo f;
    f.matrix = mat4(0.0);
    //Foo f = u_foo_data.data;
}
