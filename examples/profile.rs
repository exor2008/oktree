const RANGE: usize = 65536;
const COUNT: usize = 65536 * 16;

use oktree::prelude::*;
use rand::RngExt;

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

fn main() {
    let mut tree = Octree::from_aabb_with_capacity(
        Aabb::new_unchecked(TUVec3::splat(RANGE / 2), RANGE / 2),
        COUNT,
    );

    let points = random_points();

    for p in points {
        let _ = tree.insert(p);
    }
}
