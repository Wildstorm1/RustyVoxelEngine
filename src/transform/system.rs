/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use std::sync::{ Arc, Mutex };
use bevy_ecs::entity::{ Entity };
use bevy_ecs::hierarchy::{ ChildOf, Children };
use bevy_ecs::query::{ With, Without };
use bevy_ecs::system::{ Query };
use bevy_ecs::world::{ Mut };
use glm::{ Matrix4 };
use crate::transform::{ Transform, GlobalTransform };

fn compute_transform(
    transform: &Transform,
) -> Matrix4<f32> {
    let t_x = transform.translation.x;
    let t_y = transform.translation.y;
    let t_z = transform.translation.z;

    let s_x = transform.scale;
    let s_y = transform.scale;
    let s_z = transform.scale;

    let r_x = transform.rotation.x;
    let r_y = transform.rotation.y;
    let r_z = transform.rotation.z;
    let r_w = transform.rotation.w;

    glm::mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        t_x, t_y, t_z, 1.0
    ) * glm::mat4(
        1.0 - 2.0 * r_y * r_y - 2.0 * r_z * r_z, 2.0 * (r_x * r_y + r_z * r_w), 2.0 * (r_x * r_z - r_y * r_w), 0.0,
        2.0 * (r_x * r_y - r_z * r_w), 1.0 - 2.0 * r_x * r_x - 2.0 * r_z * r_z, 2.0 * (r_y * r_z + r_x * r_w), 0.0,
        2.0 * (r_x * r_z + r_y * r_w), 2.0 * (r_y * r_z - r_x * r_w), 1.0 - 2.0 * r_x * r_x - 2.0 * r_y * r_y, 0.0,
        0.0, 0.0, 0.0, 1.0
    ) * glm::mat4(
        s_x, 0.0, 0.0, 0.0,
        0.0, s_y, 0.0, 0.0,
        0.0, 0.0, s_z, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

/*
 * Function is unsafe because there is no promise that the structure
 * of parents and children follow a strict DAG tree
 */
unsafe fn propagate_transform_helper(
    parent: &GlobalTransform,
    node: Entity,
    nodes: Arc<Mutex<Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<ChildOf>>>>,
    stack: &mut Vec<(Entity, GlobalTransform)>
) {
    let mut nodes_list = nodes.lock().unwrap();

    match nodes_list.get_mut(node) {
        Ok((transform, mut global, children)) => {
            global.p_world = parent.p_world * compute_transform(transform);

            match children {
                Some(children) => {
                    for child in children.iter() {
                        stack.push((*child, *global));
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
}

pub fn propagate_transform_system(
    mut roots: Query<(&Transform, &mut GlobalTransform, Option<&Children>), Without<ChildOf>>,
    nodes: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<ChildOf>>,
) {
    let sharable_nodes = Arc::new(Mutex::new(nodes));

    for (transform, mut global, children) in roots.iter_mut() {
        // Root node is already at its final position
        global.p_world = compute_transform(transform);

        let mut stack= Vec::new();

        match children {
            Some(children) => {
                for child in children.iter() {
                    stack.push((*child, *global));
                }
            },
            _ => {}
        }

        while !stack.is_empty() {
            match stack.pop() {
                Some((child, global)) => {
                    /*
                     * Promise in our game not to break the invariant
                     */
                    unsafe { propagate_transform_helper(&global, child, sharable_nodes.clone(), &mut stack) };
                },
                _ => {}
            }


        }
    }
}