use bevy::math::{
    Dir3, Vec3,
};
use bevy::shape::{Aabb3d, BoundingSphere, RayCast3d};
use oktree::prelude::*;

fn main() -> Result<(), TreeError> {
    let aabb = Aabb::new(TUVec3::splat(16), 16u8);
    let mut tree = Octree::from_aabb_with_capacity(aabb?, 10);

    let c1 = DummyCell::new(TUVec3::splat(1u8));
    let c2 = DummyCell::new(TUVec3::splat(8u8));

    let c1_id = tree.insert(c1)?;
    let c2_id = tree.insert(c2)?;

    // Searching by point
    assert_eq!(tree.find(&TUVec3::new(1, 1, 1)), Some(c1_id));
    assert_eq!(tree.find(&TUVec3::new(8, 8, 8)), Some(c2_id));
    assert_eq!(tree.find(&TUVec3::new(1, 2, 8)), None);
    assert_eq!(tree.find(&TUVec3::splat(100)), None);

    // Searching for the ray intersection
    let ray = RayCast3d::new(Vec3::new(1.5, 7.0, 1.9), Dir3::NEG_Y, 100.0);

    // Hit!
    assert_eq!(
        tree.ray_cast(&ray),
        HitResult {
            element: Some(ElementId(0)),
            distance: 5.0
        }
    );

    assert_eq!(tree.remove(ElementId(0)), Ok(()));

    // Miss!
    assert_eq!(
        tree.ray_cast(&ray),
        HitResult {
            element: None,
            distance: 0.0
        }
    );

    let c1 = DummyCell::new(TUVec3::splat(1u8));
    let c1_id = tree.insert(c1)?;

    // Aabb intersection
    let aabb = Aabb3d::new(Vec3::splat(2.0), Vec3::splat(2.0));
    assert_eq!(tree.intersect(&aabb), vec![c1_id]);

    // Sphere intersection
    let sphere = BoundingSphere::new(Vec3::splat(2.0), 2.0);
    assert_eq!(tree.intersect(&sphere), vec![c1_id]);

    Ok(())
}

struct DummyCell {
    position: TUVec3<u8>,
}

impl Position for DummyCell {
    type U = u8;
    fn position(&self) -> TUVec3<u8> {
        self.position
    }
}

impl DummyCell {
    fn new(position: TUVec3<u8>) -> Self {
        DummyCell { position }
    }
}
