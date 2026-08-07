/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::{ Voxel, UniformBufferObject };
use crate::voxels::{ Chunk, ChunkCache, ChunkCoord };
use noise::{ NoiseFn, Perlin, Seedable };

pub fn world_init(width: usize, height: usize) -> (ChunkCache, Vec<ChunkCoord>) {
    let perlin = Perlin::new(1);
    let mut chunk_store = ChunkCache::new();
    let mut coords = Vec::new();

    for cy in 0..height {
        for cz in 0..width {
            for cx in 0..width {
                let mut chunk = Chunk::new();

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let value = perlin.get([
                                (cx * 16 + x) as f64 / 16.0f64,
                                (cy * 16 + y) as f64 / 16.0f64,
                                (cz * 16 + z) as f64 / 16.0f64
                            ]);

                            if value > 0.0 {
                                chunk.set_voxel(x, y, z, Voxel {
                                    metadata: 0,
                                    data: 1,
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

                chunk_store.cache(ChunkCoord::new(cx, cy, cz), chunk, model);
            }
        }
    }

    for cy in 1..(height - 1) {
        for cz in 1..(width - 1) {
            for cx in 1..(width - 1) {
                let coordinate = ChunkCoord::new(cx, cy, cz);
                coords.push(coordinate);

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();
                            let mut voxel = chunk.get_voxel(x, y, z);

                            // Right face: +z
                            if z < 15 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y, z + 1).data != 0 {
                                    voxel.metadata |= 0x10000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy, cz + 1)).unwrap();

                                if neighbor_chunk.get_voxel(x, y, 0).data != 0 {
                                    voxel.metadata |= 0x10000000;
                                }
                            }

                            // Left face: -z
                            if z > 0 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y, z - 1).data != 0 {
                                    voxel.metadata |= 0x20000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy, cz - 1)).unwrap();

                                if neighbor_chunk.get_voxel(x, y, 15).data != 0 {
                                    voxel.metadata |= 0x20000000;
                                }
                            }

                            // Front face: +x
                            if x < 15 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x + 1, y, z).data != 0 {
                                    voxel.metadata |= 0x80000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx + 1, cy, cz)).unwrap();

                                if neighbor_chunk.get_voxel(0, y, z).data != 0 {
                                    voxel.metadata |= 0x80000000;
                                }
                            }

                            // Back face: -x
                            if x > 0 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x - 1, y, z).data != 0 {
                                    voxel.metadata |= 0x40000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx - 1, cy, cz)).unwrap();

                                if neighbor_chunk.get_voxel(15, y, z).data != 0 {
                                    voxel.metadata |= 0x40000000;
                                }
                            }

                            // Top face: +y
                            if y < 15 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y + 1, z).data != 0 {
                                    voxel.metadata |= 0x08000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy + 1, cz)).unwrap();

                                if neighbor_chunk.get_voxel(x, 0, z).data != 0 {
                                    voxel.metadata |= 0x08000000;
                                }
                            }

                            // Bottom face: -y
                            if y > 0 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y - 1, z).data != 0 {
                                    voxel.metadata |= 0x04000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy - 1, cz)).unwrap();

                                if neighbor_chunk.get_voxel(x, 15, z).data != 0 {
                                    voxel.metadata |= 0x04000000;
                                }
                            }

                            let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();
                            chunk.set_voxel(x, y, z, voxel);
                        }
                    }
                }
            }
        }
    }

    (chunk_store, coords)
}