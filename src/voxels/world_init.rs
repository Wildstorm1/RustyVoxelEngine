/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::camera::PlayerCamera;
use crate::render::{
    Voxel,
    UniformBufferObject,
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
use crate::voxels::{ Chunk, ChunkCache, Coord };
use bevy_ecs::system::Res;
use noise::{ NoiseFn, Perlin, Seedable };
use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct VoxelWorld {
    cache: ChunkCache,
    visibility: Vec<Coord>
}

impl VoxelWorld {
    pub fn new(width: usize, height: usize) -> Self {
        let (mut chunk_store, coords) = world_init(width, height);

        Self {
            cache: chunk_store,
            visibility: coords,
        }
    }

    pub fn get_cache(&self) -> &ChunkCache {
        &self.cache
    }

    pub fn get_visibility(&self) -> Vec<Coord> {
        self.visibility.clone()
    }

    pub fn clear_visibility(&mut self) {
        self.visibility.clear();
    }

    pub fn set_visible(&mut self, coord: Coord) {
        self.visibility.push(coord);
    }
}

pub fn world_init(width: usize, height: usize) -> (ChunkCache, Vec<Coord>) {
    let perlin = Perlin::new(1);
    let mut chunk_store = ChunkCache::new();
    let mut coords = Vec::new();

    // Pass 1 assigns "solidness"
    for cy in 0..height {
        for cz in 0..width {
            for cx in 0..width {
                let mut chunk = Chunk::new();

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let y_pos = cy * 16 + y;

                            let turbulence = 20.0f64 * perlin.get([
                                (cx * 16 + x) as f64 / 16.0f64,
                                y_pos as f64 / 16.0f64,
                                (cz * 16 + z) as f64 / 16.0f64
                            ]);

                            let plane_value = perlin.get([
                                ((cx * 16 + x) as f64 + turbulence) / 32.0f64,
                                ((cz * 16 + z) as f64 + turbulence) / 32.0f64
                            ]);

                            let noise = 0.25f64 * ((plane_value + 1.0f64) / 2.0f64);

                            if ((y_pos as f32 / (height * 16) as f32) + noise as f32) < 0.5f32 {
                                chunk.set_is_air(false);

                                chunk.set_voxel(x, y, z, Voxel {
                                    metadata: 0,
                                    data: 5,
                                });
                            }
                        }
                    }
                }

                let model = UniformBufferObject {
                    model: glm::ext::translate(
                        &glm::mat4(
                            1.0f32, 0.0f32, 0.0f32, 0.0f32,
                            0.0f32, 1.0f32, 0.0f32, 0.0f32,
                            0.0f32, 0.0f32, 1.0f32, 0.0f32,
                            0.0f32, 0.0f32, 0.0f32, 1.0f32
                        ),
                        glm::vec3((cx * 16) as f32, (cy * 16) as f32, (cz * 16) as f32)
                    )
                };

                chunk_store.cache(Coord::new(cx as i64, cy as i64, cz as i64), chunk, model);
            }
        }
    }

    // Pass 2 assigns block type and ambient occlusion
    for cy in 1..(height - 1) {
        for cz in 1..(width - 1) {
            for cx in 1..(width - 1) {
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
    for cy in 1..(height - 1) {
        for cz in 1..(width - 1) {
            for cx in 1..(width - 1) {
                let chunk_coordinate = Coord::new(cx as i64, cy as i64, cz as i64);
                coords.push(chunk_coordinate);

                for x in (cx * 16)..((cx + 1) * 16) {
                    for y in (cy * 16)..((cy + 1) * 16) {
                        for z in (cz * 16)..((cz + 1) * 16) {
                            let voxel_coord = Coord::new(x as i64, y as i64, z as i64);
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
                    }
                }
            }
        }
    }

    (chunk_store, coords)
}