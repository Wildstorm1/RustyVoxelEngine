/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::system::{ Commands, Res };
use glm::Vector3;
use noise::{ NoiseFn, Perlin };
use crate::render::{
    Voxel,
    RIGHT_FACE_METADATA_BIT,
    LEFT_FACE_METADATA_BIT,
    FRONT_FACE_METADATA_BIT,
    BACK_FACE_METADATA_BIT,
    TOP_FACE_METADATA_BIT,
    BOTTOM_FACE_METADATA_BIT,
    VOXEL_P1_00_P1,
    VOXEL_00_P1_P1,
    VOXEL_P1_P1_P1,
    VOXEL_N1_00_P1,
    VOXEL_N1_P1_P1,
    VOXEL_00_N1_P1,
    VOXEL_P1_N1_P1,
    VOXEL_N1_N1_P1,
    VOXEL_P1_P1_00,
    VOXEL_P1_N1_00,
    VOXEL_P1_00_N1,
    VOXEL_P1_P1_N1,
    VOXEL_P1_N1_N1,
    VOXEL_00_P1_N1,
    VOXEL_00_N1_N1,
    VOXEL_N1_00_N1,
    VOXEL_N1_P1_N1,
    VOXEL_N1_N1_N1,
    VOXEL_N1_P1_00,
    VOXEL_N1_N1_00
};
use crate::transform::Transform;
use crate::voxels::{Chunk, ChunkCache, Coord, VoxelMapSettings};

fn fill_map_chunk(
    noise: &Perlin,
    coordinate: Vector3<f32>,
    world_depth: f32,
    chunk_size: usize
) -> Chunk {
    let mut chunk = Chunk::new();

    for x in 0..chunk_size {
        for z in 0..chunk_size {
            for y in 0..chunk_size {
                let vox_pos = coordinate + glm::vec3(x as f32, y as f32, z as f32);

                let turbulence = 20.0f64 * noise.get([
                    vox_pos.x as f64 / 16.0,
                    vox_pos.y as f64 / 16.0,
                    vox_pos.z as f64 / 16.0
                ]);

                let plane_value = noise.get([
                    (vox_pos.x as f64 / 16.0 + turbulence) / 32.0f64,
                    (vox_pos.z as f64 / 16.0 + turbulence) / 32.0f64
                ]);

                let noise = 0.25f64 * ((plane_value + 1.0f64) / 2.0f64);

                if ((vox_pos.y / world_depth) + noise as f32) < 0.5f32 {
                    chunk.set_is_air(false);

                    chunk.set_voxel(x, y, z, Voxel {
                        metadata: 0,
                        data: 5,
                    });
                }
            }
        }
    }

    chunk
}

fn set_voxel_flags(
    voxel_coord: Coord,
    chunk_store: &mut ChunkCache
) {
    let mut voxel = chunk_store.get_voxel(voxel_coord).unwrap();

    // Right face: +z
    if chunk_store.get_voxel(voxel_coord + Coord::unit_z()).unwrap().data != 0 {
        voxel.metadata |= RIGHT_FACE_METADATA_BIT;
    }

    // Left face: -z
    if chunk_store.get_voxel(voxel_coord - Coord::unit_z()).unwrap().data != 0 {
        voxel.metadata |= LEFT_FACE_METADATA_BIT;
    }

    // Front face: +x
    if chunk_store.get_voxel(voxel_coord + Coord::unit_x()).unwrap().data != 0 {
        voxel.metadata |= FRONT_FACE_METADATA_BIT;
    }

    // Back face: -x
    if chunk_store.get_voxel(voxel_coord - Coord::unit_x()).unwrap().data != 0 {
        voxel.metadata |= BACK_FACE_METADATA_BIT;
    }

    // Top face: +y
    if chunk_store.get_voxel(voxel_coord + Coord::unit_y()).unwrap().data != 0 {
        voxel.metadata |= TOP_FACE_METADATA_BIT;
    }

    // Bottom face: -y
    if chunk_store.get_voxel(voxel_coord - Coord::unit_y()).unwrap().data != 0 {
        voxel.metadata |= BOTTOM_FACE_METADATA_BIT;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, 0, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_00_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(0, 1, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_00_P1_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, 1, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_P1_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, 0, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_00_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, 1, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_P1_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(0, -1, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_00_N1_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, -1, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_N1_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, -1, 1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_N1_P1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, 1, 0)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_P1_00;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, -1, 0)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_N1_00;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, 0, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_00_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, 1, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_P1_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(1, -1, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_P1_N1_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(0, 1, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_00_P1_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(0, -1, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_00_N1_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, 0, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_00_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, 1, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_P1_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, -1, -1)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_N1_N1;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, 1, 0)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_P1_00;
    }

    if chunk_store.get_voxel(voxel_coord + Coord::new(-1, -1, 0)).unwrap().data != 0 {
        voxel.metadata |= VOXEL_N1_N1_00;
    }

    chunk_store.set_voxel(voxel_coord, voxel).unwrap();
}

pub fn generate_initial_chunks(
    voxels: Res<VoxelMapSettings>,
    mut commands: Commands
) {
    let perlin = Perlin::new(1);
    let mut chunk_store = ChunkCache::new();
    let mut coords = Vec::new();

    // Pass 1 assigns "solidness"
    for cy in 0..voxels.depth() {
        for cz in 0..voxels.width() {
            for cx in 0..voxels.width() {
                // TODO: hard-coded chunk size
                let position = glm::vec3((cx * 16) as f32, (cy * 16) as f32, (cz * 16) as f32);

                let chunk = fill_map_chunk(
                    &perlin,
                    position,
                    (voxels.depth() * 16) as f32,
                    16
                );

                chunk_store.cache(Coord::new(cx as i64, cy as i64, cz as i64), chunk);
            }
        }
    }

    // Pass 2 assigns block type and ambient occlusion
    for cy in 1..(voxels.depth() - 1) {
        for cz in 1..(voxels.width() - 1) {
            for cx in 1..(voxels.width() - 1) {
                for x in (cx * 16)..((cx + 1) * 16) {
                    for y in (cy * 16)..((cy + 1) * 16) {
                        for z in (cz * 16)..((cz + 1) * 16) {
                            let voxel_coord = Coord::new(x as i64, y as i64, z as i64);
                            let up_voxel = chunk_store.get_voxel(voxel_coord + Coord::unit_y()).unwrap();
                            let voxel = chunk_store.get_voxel(voxel_coord).unwrap();

                            // If upper voxel is open, assign grass
                            if voxel.data != 0 && up_voxel.data == 0 {
                                chunk_store.set_voxel(voxel_coord, Voxel {
                                    metadata: 0,
                                    data: 1,
                                }).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    // Pass 3 assigns metadata about neighbors
    for cy in 1..(voxels.depth() - 1) {
        for cz in 1..(voxels.width() - 1) {
            for cx in 1..(voxels.width() - 1) {
                let chunk_coordinate = Coord::new(cx as i64, cy as i64, cz as i64);
                coords.push(chunk_coordinate);

                for x in (cx * 16)..((cx + 1) * 16) {
                    for y in (cy * 16)..((cy + 1) * 16) {
                        for z in (cz * 16)..((cz + 1) * 16) {
                            let voxel_coord = Coord::new(x as i64, y as i64, z as i64);
                            set_voxel_flags(voxel_coord, &mut chunk_store);
                        }
                    }
                }
            }
        }
    }

    for coord in coords {
        let chunk = chunk_store.remove_chunk(coord).unwrap();

        commands.spawn((
            chunk,
            Transform {
                translation: glm::vec3(
                    (coord.x() * 16) as f32,
                    (coord.y() * 16) as f32,
                    (coord.z() * 16) as f32
                ),
                rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
                scale: glm::vec3(1.0, 1.0, 1.0)
            }
        ));
    }
}