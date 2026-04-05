use std::time::Duration;

use bevy::math::{
    bounding::{Aabb3d, BoundingSphere, RayCast3d},
    Dir3A, Vec3A,
};
use criterion::{criterion_group, criterion_main, Criterion};
use oktree::prelude::*;
use rand::RngExt;

const RANGE: usize = 4096;
const COUNT: usize = 65536;
const RAY_COUNT: usize = 4096;
const VOLUME_SIZE: f32 = 100.0;

#[derive(Clone, Copy)]
struct DummyCell<U: Unsigned> {
    position: TUVec3<U>,
}

impl<U: Unsigned> Position for DummyCell<U> {
    type U = U;
    fn position(&self) -> TUVec3<U> {
        self.position
    }
}

impl<U: Unsigned> DummyCell<U> {
    fn new(position: TUVec3<U>) -> Self {
        DummyCell { position }
    }
}

fn random_points() -> Vec<DummyCell<usize>> {
    let mut points = Vec::with_capacity(COUNT);
    let mut rnd = rand::rng();

    for _ in 0..COUNT {
        let x = rnd.random_range(0..=RANGE);
        let y = rnd.random_range(0..=RANGE);
        let z = rnd.random_range(0..=RANGE);
        let position = TUVec3::new(x, y, z);
        let cell = DummyCell::new(position);

        points.push(cell);
    }

    points
}

fn random_rays() -> Vec<RayCast3d> {
    let mut rays = Vec::with_capacity(RAY_COUNT);
    let mut rnd = rand::rng();

    for _ in 0..RAY_COUNT {
        let x = rnd.random_range(0.0..=RANGE as f32);
        let y = rnd.random_range(0.0..=RANGE as f32);
        let z = rnd.random_range(0.0..=RANGE as f32);
        let origin = Vec3A::new(x, y, z);

        let x_dir = rnd.random_range(0.0..=1.0);
        let y_dir = rnd.random_range(0.0..=1.0);
        let z_dir = rnd.random_range(0.0..=1.0);
        let direction = Vec3A::new(x_dir, y_dir, z_dir);
        let direction = Dir3A::new(direction).unwrap();

        let ray = RayCast3d::new(origin, direction, RANGE as f32);
        rays.push(ray);
    }

    rays
}

fn random_spheres() -> Vec<BoundingSphere> {
    let mut spheres = Vec::with_capacity(RAY_COUNT);
    let mut rnd = rand::rng();

    for _ in 0..RAY_COUNT {
        let x = rnd.random_range(0.0..=RANGE as f32);
        let y = rnd.random_range(0.0..=RANGE as f32);
        let z = rnd.random_range(0.0..=RANGE as f32);
        let position = Vec3A::new(x, y, z);
        let radius = rnd.random_range(0.0..VOLUME_SIZE);
        let sphere = BoundingSphere::new(position, radius);

        spheres.push(sphere);
    }

    spheres
}

fn random_aabbs() -> Vec<Aabb3d> {
    let mut aabbs = Vec::with_capacity(RAY_COUNT);
    let mut rnd = rand::rng();

    for _ in 0..RAY_COUNT {
        let x = rnd.random_range(0.0..=RANGE as f32);
        let y = rnd.random_range(0.0..=RANGE as f32);
        let z = rnd.random_range(0.0..=RANGE as f32);

        let x_size = rnd.random_range(0.0..=VOLUME_SIZE);
        let y_size = rnd.random_range(0.0..=VOLUME_SIZE);
        let z_size = rnd.random_range(0.0..=VOLUME_SIZE);

        let position = Vec3A::new(x, y, z);
        let size = Vec3A::new(x_size, y_size, z_size);

        let aabb = Aabb3d::new(position, size);

        aabbs.push(aabb);
    }

    aabbs
}

fn octree_insert(points: &[DummyCell<usize>]) -> Octree<usize, DummyCell<usize>> {
    let mut tree = Octree::from_aabb_with_capacity(
        Aabb::new_unchecked(TUVec3::splat(RANGE / 2), RANGE / 2),
        COUNT,
    );

    for p in points {
        let _ = tree.insert(*p);
    }

    tree
}

fn octree_insert_using_clear(
    tree: &mut Octree<usize, DummyCell<usize>>,
    points: &[DummyCell<usize>],
) {
    tree.clear();
    for p in points {
        let _ = tree.insert(*p);
    }
}

fn octree_remove(tree: &mut Octree<usize, DummyCell<usize>>) {
    tree.restore_garbage().unwrap();
    for element in 0..tree.len() {
        let _ = tree.remove(element.into());
    }
}

fn octree_find(tree: &Octree<usize, DummyCell<usize>>, points: &[DummyCell<usize>]) {
    for p in points {
        let _ = tree.find(&p.position);
    }
}

fn octree_ray_cast(tree: &Octree<usize, DummyCell<usize>>, rays: &[RayCast3d]) {
    for ray in rays {
        let _ = tree.ray_cast(ray);
    }
}

fn octree_sphere_intersect(tree: &Octree<usize, DummyCell<usize>>, spheres: &[BoundingSphere]) {
    for sphere in spheres {
        let _ = tree.intersect(sphere);
    }
}

fn octree_aabb_intersect(tree: &Octree<usize, DummyCell<usize>>, aabbs: &[Aabb3d]) {
    for aabb in aabbs {
        let _ = tree.intersect(aabb);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("main");
    group.measurement_time(Duration::from_secs(10));
    let points = random_points();
    let rays = random_rays();
    let spheres = random_spheres();
    let aabbs = random_aabbs();

    let mut tree = octree_insert(&points);
    group.bench_function("octree insert", |b| {
        b.iter(|| octree_insert_using_clear(&mut tree, &points))
    });

    let mut tree = octree_insert(&points);
    group.bench_function("octree remove", |b| b.iter(|| octree_remove(&mut tree)));

    let tree = octree_insert(&points);
    group.bench_function("octree find", |b| b.iter(|| octree_find(&tree, &points)));

    let tree = octree_insert(&points);
    group.bench_function("octree ray cast", |b| {
        b.iter(|| octree_ray_cast(&tree, &rays))
    });

    let tree = octree_insert(&points);
    group.bench_function("octree sphere intersect", |b| {
        b.iter(|| octree_sphere_intersect(&tree, &spheres))
    });

    let tree = octree_insert(&points);
    group.bench_function("octree aabb intersect", |b| {
        b.iter(|| octree_aabb_intersect(&tree, &aabbs))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
