use bevy::math::{
    Dir3, Vec3,
};
use bevy::shape::{Aabb3d, BoundingSphere, RayCast3d};
use oktree::prelude::*;

fn main() -> Result<(), TreeError> {
    let aabb = Aabb::new(TUVec3::splat(8), 8u8);
    let mut tree = Octree::from_aabb_with_capacity(aabb?, 10);

    let v1_volume = Aabb::new(TUVec3::new(9, 5, 4), 4).unwrap();
    let v1 = DummyVolume::new(v1_volume);
    let v1_id = tree.insert(v1)?;

    let v2_volume = Aabb::new(TUVec3::new(14, 14, 4), 4).unwrap();
    let v2 = DummyVolume::new(v2_volume);
    let v2_id = tree.insert(v2)?;

    let v3_volume = Aabb::new(TUVec3::new(7, 5, 4), 4).unwrap();
    let v3 = DummyVolume::new(v3_volume);
    assert!(tree.insert(v3).is_err());

    // Searching by point
    assert_eq!(tree.find(&TUVec3::new(9, 5, 4)), Some(v1_id));
    assert_eq!(tree.find(&TUVec3::new(16, 12, 2)), Some(v2_id));
    assert_eq!(tree.find(&TUVec3::new(1, 2, 8)), None);
    assert_eq!(tree.find(&TUVec3::splat(100)), None);

    // Searching for the ray intersection
    let ray = RayCast3d::new(Vec3::new(9.0, 15.0, 4.0), Dir3::NEG_Y, 100.0);

    // Hit!
    assert_eq!(
        tree.ray_cast(&ray),
        HitResult {
            element: Some(ElementId(0)),
            distance: 6.0
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

    let v1_volume = Aabb::new(TUVec3::new(9, 5, 4), 4).unwrap();
    let v1 = DummyVolume::new(v1_volume);
    let v1_id = tree.insert(v1)?;

    // Aabb intersection
    let aabb = Aabb3d::new(Vec3::splat(8.0), Vec3::splat(8.0));
    assert_eq!(tree.intersect(&aabb), vec![v1_id, v2_id]);

    // Sphere intersection
    let sphere = BoundingSphere::new(Vec3::new(15.0, 15.0, 0.0), 5.0);
    assert_eq!(tree.intersect(&sphere), vec![v2_id]);

    Ok(())
}

struct DummyVolume<U: Unsigned> {
    aabb: Aabb<U>,
}

impl<U: Unsigned> Volume for DummyVolume<U> {
    type U = U;
    fn volume(&self) -> Aabb<U> {
        self.aabb
    }
}

impl<U: Unsigned> DummyVolume<U> {
    fn new(aabb: Aabb<U>) -> Self {
        Self { aabb }
    }
}
