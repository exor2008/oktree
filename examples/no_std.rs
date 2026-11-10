#![no_std]

use bevy::math::{Dir3, Vec3};
use bevy::shape::{Ray3d, RayCast3d};
use oktree::prelude::*;

extern crate alloc;

fn main() -> Result<(), TreeError> {
    let aabb = Aabb::new_unchecked(TUVec3::splat(8u8), 8);
    let mut tree = Octree::from_aabb(aabb);

    let e1_id = tree.insert(TUVec3u8::new(3, 3, 3))?;
    assert_eq!(e1_id, ElementId(0));

    let ray = Ray3d::new(Vec3::ONE, Dir3::X);
    let hit = tree.ray_cast(&RayCast3d::from_ray(ray, 10.0));
    assert_eq!(
        hit,
        HitResult {
            element: None,
            distance: 0.0
        }
    );
    Ok(())
}
