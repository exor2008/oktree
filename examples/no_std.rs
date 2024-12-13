#![no_std]

use bevy::math::{bounding::RayCast3d, Dir3, Ray3d, Vec3};
use oktree::prelude::*;

extern crate alloc;
extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
