/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use std::f32;

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ ChildOf };
use bevy_ecs::system::{ Commands, Query, Res, ResMut, Single };
use glm::Vector3;
use noise::{ NoiseFn, Perlin };
use crate::camera::PlayerCamera;
use crate::input::{MouseButtonInput, MouseCode};
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
use crate::timer::Timer;
use crate::transform::Transform;
use crate::transform::component::GlobalTransform;
use crate::voxels::{ Chunk, ChunkCache, ChunkWrapper, Coord, Grid, Model, Size, VoxelMap, VoxelMapSettings };

fn fill_map_chunk(
    voxels: &Res<VoxelMapSettings>,
    noise: &Perlin,
    coordinate: Vector3<f32>
) -> Chunk {
    let world_depth = (voxels.depth() * voxels.chunk_size()) as f32;

    let mut chunk = Chunk::new(Size::new(
        voxels.chunk_size(),
        voxels.chunk_size(),
        voxels.chunk_size()
    ));

    for x in 0..voxels.chunk_size() {
        for z in 0..voxels.chunk_size() {
            for y in 0..voxels.chunk_size() {
                let vox_pos = coordinate + glm::vec3(x as f32, y as f32, z as f32);

                let turbulence = 20.0f64 * noise.get([
                    vox_pos.x as f64 / voxels.chunk_size() as f64,
                    vox_pos.y as f64 / voxels.chunk_size() as f64,
                    vox_pos.z as f64 / voxels.chunk_size() as f64
                ]);

                let plane_value = noise.get([
                    (vox_pos.x as f64 / voxels.chunk_size() as f64 + turbulence) / 32.0f64,
                    (vox_pos.z as f64 / voxels.chunk_size() as f64 + turbulence) / 32.0f64
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

/*
 * Assumes all necessary chunk data exists in the store
 */
fn set_voxel_flags<T: Grid>(
    voxel_coord: Coord,
    chunk_store: &mut T
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
    mut voxel_map: ResMut<VoxelMap>,
    mut commands: Commands
) {
    let perlin = Perlin::new(1);
    let mut chunk_store = ChunkCache::new(Size::new(
        voxels.chunk_size(),
        voxels.chunk_size(),
        voxels.chunk_size()
    ));
    let mut coords = Vec::new();

    // Pass 1 assigns "solidness"
    for cy in 0..voxels.depth() {
        for cz in 0..voxels.width() {
            for cx in 0..voxels.width() {
                let position = glm::vec3(
                    (cx * voxels.chunk_size()) as f32,
                    (cy * voxels.chunk_size()) as f32,
                    (cz * voxels.chunk_size()) as f32
                );

                let chunk = fill_map_chunk(
                    &voxels,
                    &perlin,
                    position
                );

                chunk_store.cache(Coord::new(cx as i64, cy as i64, cz as i64), chunk);
            }
        }
    }

    // Pass 2 assigns block type and ambient occlusion
    for cy in 1..(voxels.depth() - 1) {
        for cz in 1..(voxels.width() - 1) {
            for cx in 1..(voxels.width() - 1) {
                for x in (cx * voxels.chunk_size())..((cx + 1) * voxels.chunk_size()) {
                    for y in (cy * voxels.chunk_size())..((cy + 1) * voxels.chunk_size()) {
                        for z in (cz * voxels.chunk_size())..((cz + 1) * voxels.chunk_size()) {
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

                for x in (cx * voxels.chunk_size())..((cx + 1) * voxels.chunk_size()) {
                    for y in (cy * voxels.chunk_size())..((cy + 1) * voxels.chunk_size()) {
                        for z in (cz * voxels.chunk_size())..((cz + 1) * voxels.chunk_size()) {
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

        let entity = commands.spawn((
            chunk,
            Transform {
                translation: glm::vec3(
                    (coord.x() * voxels.chunk_size() as i64) as f32,
                    (coord.y() * voxels.chunk_size() as i64) as f32,
                    (coord.z() * voxels.chunk_size() as i64) as f32
                ),
                rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
                scale: 1.0,
            },
            GlobalTransform::IDENTITY,
        )).id();

        voxel_map.insert(coord, entity);
    }
}

fn mesh_free_floating_chunk(
    chunk: Chunk
) -> Chunk {
    let size = chunk.size();
    let mut wrapper = ChunkWrapper::new(chunk);

    for y in 0..size.y() {
        for z in 0..size.z() {
            for x in 0..size.x() {
                let voxel_coord = Coord::new(x as i64, y as i64, z as i64);
                set_voxel_flags(voxel_coord, &mut wrapper);
            }
        }
    }

    wrapper.to_owned()
}

pub fn generate_hermit_crab_chunks(
    mut commands: Commands
) {
    let hermit_model = commands.spawn((
        Model {},
        Transform {
            translation: glm::vec3(100.5, 49.18, 100.5),
            rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
            scale: 1.0 / 16.0,
        },
        GlobalTransform::IDENTITY,
    )).id();

    let mut claw = Chunk::new(Size::new(3, 2, 2));
    claw.set_is_air(false);

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..3 {
                let mut id = 15;

                if y == 1 && x == 1 && z == 0 {
                    id = 14;
                }

                claw.set_voxel(x, y, z, Voxel {
                    metadata: 0,
                    data: id,
                });
            }
        }
    }

    commands.spawn((
        mesh_free_floating_chunk(claw),
        Transform {
            translation: glm::vec3(3.0, -1.0, -3.5),
            rotation: glm::vec4(-0.0789899, 0.4210101, -0.0368336, 0.902859),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut crusher = Chunk::new(Size::new(3, 2, 2));
    crusher.set_is_air(false);

    for y in 0..2 {
        for z in 0..2 {
            for x in 0..3 {
                let mut id = 15;

                if y == 1 && x == 1 && z == 1 {
                    id = 14;
                }

                crusher.set_voxel(x, y, z, Voxel {
                    metadata: 0,
                    data: id,
                });
            }
        }
    }

    commands.spawn((
        mesh_free_floating_chunk(crusher),
        Transform {
            translation: glm::vec3(-3.0, -1.0, -3.5),
            rotation: glm::vec4(-0.029809, 0.9361168, -0.0818996, 0.3407187),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut shell = Chunk::new(Size::new(5, 5, 5));
    shell.set_is_air(false);

    for y in 0..5 {
        for z in 0..5 {
            for x in 0..5 {
                shell.set_voxel(x, y, z, Voxel {
                    metadata: 0,
                    data: 12,
                });
            }
        }
    }

    // -x face
    shell.set_voxel(0, 0, 0, Voxel {
        metadata: 0,
        data: 9,
    });
    shell.set_voxel(0, 0, 1, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(0, 0, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(0, 0, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 0, 4, Voxel {
        metadata: 0,
        data: 11,
    });

    shell.set_voxel(0, 1, 0, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(0, 1, 1, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 1, 2, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 1, 3, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(0, 1, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 2, 0, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(0, 2, 1, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(0, 2, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(0, 2, 3, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(0, 2, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 3, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 3, 1, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(0, 3, 2, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(0, 3, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 3, 4, Voxel {
        metadata: 0,
        data: 11,
    });

    shell.set_voxel(0, 4, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 4, 1, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 4, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(0, 4, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(0, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    // +x face
    shell.set_voxel(4, 0, 0, Voxel {
        metadata: 0,
        data: 9,
    });
    shell.set_voxel(4, 0, 1, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 0, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(4, 0, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 0, 4, Voxel {
        metadata: 0,
        data: 11,
    });

    shell.set_voxel(4, 1, 0, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 1, 1, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 1, 2, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 1, 3, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 1, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(4, 2, 0, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(4, 2, 1, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 2, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(4, 2, 3, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 2, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(4, 3, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 3, 1, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 3, 2, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 3, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 3, 4, Voxel {
        metadata: 0,
        data: 11,
    });

    shell.set_voxel(4, 4, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 1, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(4, 4, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    // -z face
    shell.set_voxel(0, 0, 0, Voxel {
        metadata: 0,
        data: 9,
    });
    shell.set_voxel(1, 0, 0, Voxel {
        metadata: 0,
        data: 9,
    });
    shell.set_voxel(2, 0, 0, Voxel {
        metadata: 0,
        data: 9,
    });
    shell.set_voxel(3, 0, 0, Voxel {
        metadata: 0,
        data: 9,
    });
    shell.set_voxel(4, 0, 0, Voxel {
        metadata: 0,
        data: 9,
    });

    shell.set_voxel(0, 1, 0, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(1, 1, 0, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(2, 1, 0, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(3, 1, 0, Voxel {
        metadata: 0,
        data: 10,
    });
    shell.set_voxel(4, 1, 0, Voxel {
        metadata: 0,
        data: 10,
    });

    shell.set_voxel(0, 2, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 2, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(2, 2, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(3, 2, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 2, 0, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 3, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 3, 0, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(2, 3, 0, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(3, 3, 0, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(4, 3, 0, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 4, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 4, 0, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(2, 4, 0, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(3, 4, 0, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 0, Voxel {
        metadata: 0,
        data: 12,
    });

    // +y face
    shell.set_voxel(0, 4, 1, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 4, 1, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(2, 4, 1, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(3, 4, 1, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 1, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 4, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(1, 4, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(2, 4, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(3, 4, 2, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(4, 4, 2, Voxel {
        metadata: 0,
        data: 11,
    });

    shell.set_voxel(0, 4, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 4, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(2, 4, 3, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(3, 4, 3, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 3, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(2, 4, 4, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(3, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    // +z face
    shell.set_voxel(0, 0, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(1, 0, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(2, 0, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(3, 0, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(4, 0, 4, Voxel {
        metadata: 0,
        data: 11,
    });

    shell.set_voxel(0, 1, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 1, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(2, 1, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(3, 1, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 1, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 2, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 2, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(2, 2, 4, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(3, 2, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 2, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    shell.set_voxel(0, 3, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(1, 3, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(2, 3, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(3, 3, 4, Voxel {
        metadata: 0,
        data: 11,
    });
    shell.set_voxel(4, 3, 4, Voxel {
        metadata: 0,
        data: 11,
    });

    shell.set_voxel(0, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(1, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(2, 4, 4, Voxel {
        metadata: 0,
        data: 13,
    });
    shell.set_voxel(3, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });
    shell.set_voxel(4, 4, 4, Voxel {
        metadata: 0,
        data: 12,
    });

    commands.spawn((
        mesh_free_floating_chunk(shell),
        Transform {
            translation: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut body = Chunk::new(Size::new(3, 2, 4));
    body.set_is_air(false);

    for y in 0..2 {
        for z in 0..4 {
            for x in 0..3 {
                body.set_voxel(x, y, z, Voxel {
                    metadata: 0,
                    data: 15,
                });
            }
        }
    }

    commands.spawn((
        mesh_free_floating_chunk(body),
        Transform {
            translation: glm::vec3(0.0, -2.0, -1.0),
            rotation: glm::vec4(-0.1305262, 0.0, 0.0, 0.9914449),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut eyes = Chunk::new(Size::new(3, 2, 1));
    eyes.set_is_air(false);
    eyes.set_voxel(0, 0, 0, Voxel {
        metadata: 0,
        data: 17,
    });
    eyes.set_voxel(0, 1, 0, Voxel {
        metadata: 0,
        data: 16,
    });
    eyes.set_voxel(2, 0, 0, Voxel {
        metadata: 0,
        data: 17,
    });
    eyes.set_voxel(2, 1, 0, Voxel {
        metadata: 0,
        data: 16,
    });

    commands.spawn((
        mesh_free_floating_chunk(eyes),
        Transform {
            translation: glm::vec3(0.0, 0.0, -3.5),
            rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut front_left_foot = Chunk::new(Size::new(1, 1, 1));
    front_left_foot.set_is_air(false);
    front_left_foot.set_voxel(0, 0, 0, Voxel {
        metadata: 0,
        data: 15,
    });
    commands.spawn((
        mesh_free_floating_chunk(front_left_foot),
        Transform {
            translation: glm::vec3(-1.5, -3.0, -1.5),
            rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut front_right_foot = Chunk::new(Size::new(1, 1, 1));
    front_right_foot.set_is_air(false);
    front_right_foot.set_voxel(0, 0, 0, Voxel {
        metadata: 0,
        data: 15,
    });
    commands.spawn((
        front_right_foot,
        Transform {
            translation: glm::vec3(1.5, -3.0, -1.5),
            rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut back_left_foot = Chunk::new(Size::new(1, 1, 1));
    back_left_foot.set_is_air(false);
    back_left_foot.set_voxel(0, 0, 0, Voxel {
        metadata: 0,
        data: 15,
    });
    commands.spawn((
        mesh_free_floating_chunk(back_left_foot),
        Transform {
            translation: glm::vec3(-1.0, -3.0, 0.5),
            rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));

    let mut back_right_foot = Chunk::new(Size::new(1, 1, 1));
    back_right_foot.set_is_air(false);
    back_right_foot.set_voxel(0, 0, 0, Voxel {
        metadata: 0,
        data: 15,
    });
    commands.spawn((
        mesh_free_floating_chunk(back_right_foot),
        Transform {
            translation: glm::vec3(1.0, -3.0, 0.5),
            rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
            scale: 1.0,
        },
        GlobalTransform::IDENTITY,
        ChildOf(hermit_model),
    ));
}

pub fn rotate_chunks(
    timer: Res<Timer>,
    mut chunks: Query<(&mut Transform, &Model)>
) {
    for (mut transform, model) in chunks.iter_mut() {
        transform.rotation = glm::vec4(
            0.0,
            (timer.total_time().as_secs_f32() * 0.25 * f32::consts::PI).sin(),
            0.0,
            (timer.total_time().as_secs_f32() * 0.25 * f32::consts::PI).cos()
        );
    }
}

pub fn block_manipulation(
    voxels: Res<VoxelMapSettings>,
    voxel_map: ResMut<VoxelMap>,
    camera_data: Res<PlayerCamera>,
    mouse: ResMut<MouseButtonInput>,
    mut chunks: Query<(Entity, &mut Chunk)>
) {
    if !mouse.is_pressed(MouseCode::LeftKey) {
        return;
    }

    let ray_direction = glm::normalize(camera_data.look_direction()) * 0.1;
    let mut position = camera_data.position();

    for i in 0..100 {
        position = position + ray_direction;

        // NOTE: The mesh center is offset so that data voxel 0,0,0 is at negative coordinates
        let chunk_coordinate = Coord::new(
            ((position.x + 8.0) / 16.0).floor() as i64,
            ((position.y + 8.0) / 16.0).floor() as i64,
            ((position.z + 8.0) / 16.0).floor() as i64
        );

        let voxel_offset = Coord::new(
            ((position.x + 8.0).floor() % voxels.chunk_size() as f32) as i64,
            ((position.y + 8.0).floor() % voxels.chunk_size() as f32) as i64,
            ((position.z + 8.0).floor() % voxels.chunk_size() as f32) as i64
        );

        let chunk_entity = voxel_map.get(chunk_coordinate);

        if chunk_entity.is_none() {
            continue;
        }

        let chunk_entity = chunk_entity.unwrap();
        let chunk = chunks.get_mut(chunk_entity);

        if chunk.is_err() {
            continue;
        }

        let (chunk_entity, mut chunk) = chunk.unwrap();
        let mut voxel = chunk.get_voxel(
            voxel_offset.x() as usize,
            voxel_offset.y() as usize,
            voxel_offset.z() as usize
        );

        if chunk.is_air() || voxel.data == 0 {
            continue;
        }

        voxel.metadata = 0;
        voxel.data = 0;

        chunk.set_voxel(
            voxel_offset.x() as usize,
            voxel_offset.y() as usize,
            voxel_offset.z() as usize,
            voxel
        );

        break;
    }
}