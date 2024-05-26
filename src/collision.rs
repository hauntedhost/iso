use crate::{player::systems::PlayerTag, schedule::UpdateSet};
use bevy::{prelude::*, utils::HashSet};

#[derive(Clone, Debug)]
pub struct CollisionPlugin {}

impl Default for CollisionPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            collision_detection::<PlayerTag>.in_set(UpdateSet::AfterEffects),
        );
    }
}

fn collision_detection<T: Component>(
    mut query: Query<(Entity, &GlobalTransform, &mut Handle<StandardMaterial>), With<T>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut colliding_entities = HashSet::new();

    // First pass: detect collisions and mark colliding entities
    for (entity_a, transform_a, mat_handle_a) in query.iter() {
        for (entity_b, transform_b, mat_handle_b) in query.iter() {
            if entity_a != entity_b && aabb_collision(transform_a, transform_b) {
                colliding_entities.insert(entity_a);
                colliding_entities.insert(entity_b);

                if let Some(material_a) = materials.get_mut(&*mat_handle_a) {
                    material_a.base_color = Color::rgb(1.0, 0.0, 0.0);
                }

                if let Some(material_b) = materials.get_mut(&*mat_handle_b) {
                    material_b.base_color = Color::rgb(1.0, 0.0, 0.0);
                }
            }
        }
    }

    // Second pass: reset non-colliding entities to original color
    for (entity, _, mat_handle) in query.iter_mut() {
        if !colliding_entities.contains(&entity) {
            if let Some(material) = materials.get_mut(&*mat_handle) {
                material.base_color = Color::rgb(0.8, 0.7, 0.6);
            }
        }
    }
}

fn aabb_collision(transform_a: &GlobalTransform, transform_b: &GlobalTransform) -> bool {
    let (scale_a, _, translation_a) = transform_a.to_scale_rotation_translation();
    let (scale_b, _, translation_b) = transform_b.to_scale_rotation_translation();

    let half_size_a = scale_a / 2.0;
    let half_size_b = scale_b / 2.0;

    let min_a = translation_a - half_size_a;
    let max_a = translation_a + half_size_a;
    let min_b = translation_b - half_size_b;
    let max_b = translation_b + half_size_b;

    let overlap_x = min_a.x <= max_b.x && max_a.x >= min_b.x;
    let overlap_y = min_a.y <= max_b.y && max_a.y >= min_b.y;
    let overlap_z = min_a.z <= max_b.z && max_a.z >= min_b.z;

    overlap_x && overlap_y && overlap_z
}
