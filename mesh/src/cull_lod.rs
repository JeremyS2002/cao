
// pub fn decimate_indexed<V: crate::Vertex>(vertices: &Vec<V>, indices: &Vec<u32>, ratio: f32) -> (Vec<V>, Vec<u32>) {
//     todo!();
// }

// pub fn gen_lod_indexed<V: crate::Vertex>(vertices: &mut Vec<V>, indices: &mut Vec<u32>, lod_count: u32, first_instance: u32, instance_count: u32, ratio: f32) -> Vec<gpu::DrawIndexedIndirectCommand> {
//     let mut half = decimate_indexed(vertices, indices, ratio);

//     let mut draw = Vec::new();

//     for _ in 0..(lod_count - 1) {
//         draw.push(gpu::DrawIndexedIndirectCommand {
//             index_count: half.1.len() as _,
//             instance_count,
//             first_index: 0,
//             vertex_offset: vertices.len() as _,
//             first_instance,
//         });
//         vertices.extend(&half.0);
//         indices.extend(&half.1);
//         half = decimate_indexed(&half.0, &half.1, ratio);
//     }

//     vertices.extend(&half.0);
//     indices.extend(&half.1);

//     draw
// }

// pub fn decimate<V: crate::Vertex>(vertices: &Vec<V>, ratio: f32) -> Vec<V> {
//     todo!();
// }

// pub fn gen_lod<V: crate::Vertex>(vertices: &mut Vec<V>, lod_count: u32, first_instance: u32, instance_count: u32, ratio: f32) -> Vec<gpu::DrawIndirectCommand> {
//     let mut half = decimate(vertices, ratio);

//     let mut draw = Vec::new();

//     for _ in 0..(lod_count - 1) {
//         draw.push(gpu::DrawIndirectCommand {
//             vertex_count: half.len() as _,
//             instance_count,
//             first_vertex: vertices.len() as _,
//             first_instance,
//         });
//         vertices.extend(&half);
//         half = decimate(&half, ratio);
//     }

//     vertices.extend(&half);

//     draw
// }