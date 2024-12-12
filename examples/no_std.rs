#![no_std]
use oktree::prelude::*;

fn main() -> Result<(), TreeError> {
    let aabb = Aabb::new_unchecked(TUVec3::splat(8u8), 8);
    let mut tree = Octree::from_aabb(aabb);

    let e1_id = tree.insert(TUVec3u8::new(3, 3, 3))?;
    assert_eq!(e1_id, ElementId(0));
    Ok(())
}
