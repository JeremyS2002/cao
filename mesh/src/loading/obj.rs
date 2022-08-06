
use crate::Vertex;

use std::convert::TryFrom;
use super::LoadError;
use std::path::Path;

pub fn load_meshes_from_obj<P: AsRef<Path> + std::fmt::Debug, V: Vertex>(
    encoder: &mut gfx::CommandEncoder<'_>,
    device: &gpu::Device,
    gen_tangents: bool,
    path: P,
    name: Option<&str>,
) -> Result<Vec<gfx::IndexedMesh<V>>, LoadError> {
    let result = tobj::load_obj(
        path, 
        &tobj::GPU_LOAD_OPTIONS,
    );

    let (models, _) = match result {
        Ok((models, materials)) => (models, materials),
        Err(e) => return Err(LoadError::Tobj(e)),
    };

    let mut meshes = Vec::with_capacity(models.len());

    for model in models {
        if model.mesh.normals.is_empty() {
            return Err(LoadError::MissingNormals(model.name))
        }

        if model.mesh.texcoords.is_empty() {
            return Err(LoadError::MissingUvs(model.name))
        }

        let vertices = model.mesh
            .positions
            .chunks(3)
            .zip(model.mesh.normals.chunks(3))
            .zip(model.mesh.texcoords.chunks(2))
            .map(|((position, normal), uv)| {
                V::new(
                    <[f32; 3]>::try_from(position).unwrap().into(),
                    <[f32; 2]>::try_from(uv).unwrap().into(),
                    <[f32; 3]>::try_from(normal).unwrap().into(),
                    None,
                    None,
                )
            })
            .collect::<Vec<_>>();

        if gen_tangents {
            // crate::utils::gen_tangents(&mut vertices);
        }

        let indices = &*model.mesh.indices;

        let mesh = match gfx::IndexedMesh::new(
            encoder,
            device,
            &vertices,
            &indices,
            name.map(|n| format!("{}_{}", n, model.name)),
        ) {
            Ok(m) => m,
            Err(e) => return Err(LoadError::Gpu(model.name, e))
        };

        meshes.push(mesh);
    }

    Ok(meshes)
}