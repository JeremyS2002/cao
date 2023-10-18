
#[allow(dead_code)]
#[derive(Copy, Clone, Default, gfx::Vertex)]
struct MyVertex {
    #[alias(in_pos, pos)]
    position: [f32; 4],
}

unsafe impl bytemuck::Pod for MyVertex { }
unsafe impl bytemuck::Zeroable for MyVertex { }

fn main() {
    use gfx::Vertex;

    println!("position: {:?}", MyVertex::get("position"));
    println!("pos     : {:?}", MyVertex::get("pos"));
    println!("in_pos  : {:?}", MyVertex::get("in_pos"));
}