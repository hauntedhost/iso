#![allow(dead_code)]
use bevy::prelude::*;

pub fn round_to_two(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

pub fn round_vec3_to_two(vector: Vec3) -> Vec3 {
    Vec3::new(
        round_to_two(vector.x),
        round_to_two(vector.y),
        round_to_two(vector.z),
    )
}
