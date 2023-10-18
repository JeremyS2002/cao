
use glam::Vec3;

#[allow(dead_code)]
#[derive(Copy, Clone, Default, gfx::Vertex)]
struct MyVertex {
    position: [f32; 4],
    depth: f32,
    distance: Vec3,
    color: glam::Vec3,
}

unsafe impl bytemuck::Pod for MyVertex { }
unsafe impl bytemuck::Zeroable for MyVertex { }

fn main() {
    use gfx::Vertex;

    println!("position: {:?}", MyVertex::get("position"));
    println!("depth   : {:?}", MyVertex::get("depth"));
    println!("distance: {:?}", MyVertex::get("distance"));
    println!("color   : {:?}", MyVertex::get("color"));
}