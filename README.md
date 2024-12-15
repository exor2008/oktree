# Oktree

[![Crates.io](https://img.shields.io/crates/v/oktree.svg)](https://crates.io/crates/oktree)
[![Docs.rs](https://docs.rs/oktree/badge.svg)](https://docs.rs/oktree)

Fast octree implementation.

![Example](/assets/example.gif)

Able to operate with [`Position`](https://docs.rs/oktree/latest/oktree/trait.Position) or [`Volume`](https://docs.rs/oktree/latest/oktree/trait.Volume) data.

Could be used with the Bevy game engine or as a standalone tree.

## Available methods:

- ### Unsigned operations

  - [`Insertion`](https://docs.rs/oktree/latest/oktree/tree/struct.Octree.html#method.insert)
  - [`Removing`](https://docs.rs/oktree/latest/oktree/tree/struct.Octree.html#method.remove)
  - [`Searching`](https://docs.rs/oktree/latest/oktree/tree/struct.Octree.html#method.find)

- ### Floating point operations (Bevy integration)

  - [`Ray casting`](https://docs.rs/oktree/latest/oktree/tree/struct.Octree.html#method.ray_cast)
  - [`Bouning sphere and bounding box intersection`](https://docs.rs/oktree/latest/oktree/tree/struct.Octree.html#method.intersect)

To enable bevy integrations:

```toml
[dependencies]
oktree = { version = "0.4.1", features = ["bevy"] }
```

### Optimizations:

- `Unsigned` arithmetics, bitwise operations.
- Tree structure is represented by flat, reusable pools. Removed data is marked only.
- Few memory allocations. [`Smallvec`](https://docs.rs/smallvec/) and [`Heapless`](https://docs.rs/heapless/) structures are used.
- No smart pointers (`Rc`, `RefCell` e.t.c)

Compensation for the inconvenience is perfomance.

## Benchmark

Octree dimensions: `4096x4096x4096`

| Operation           | Quantity                         | Time   |
| ------------------- | -------------------------------- | ------ |
| insertion           | 65536 cells                      | 21 ms  |
| removing            | 65536 cells                      | 1.5 ms |
| find                | 65536 searches in 65536 cells    | 12 ms  |
| ray intersection    | 4096 rays against 65536 cells    | 37 ms  |
| sphere intersection | 4096 spheres against 65536 cells | 8 ms   |
| box intersection    | 4096 boxes against 65536 cells   | 7 ms   |

Run benchmark:

```sh
cargo bench --all-features
```

## Examples

### Simple

You have to specify the type for the internal tree structure.

It must be any `Unsigned` type (`u8`, `u16`, `u32`, `u64`, `u128` or `usize`).

Implement [`Position`](https://docs.rs/oktree/latest/oktree/trait.Position) or [`Volume`](https://docs.rs/oktree/latest/oktree/trait.Volume) for the handled type, so that it can return it's spatial coordinates.

```rust
use bevy::math::{
    bounding::{Aabb3d, BoundingSphere, RayCast3d},
    Dir3, Vec3,
};

use oktree::prelude::*;

fn main() -> Result<(), TreeError> {
    let aabb = Aabb::new(TUVec3::splat(16), 16u8);
    let mut tree = Octree::from_aabb_with_capacity(aabb?, 10);

    let c1 = DummyCell::new(TUVec3::splat(1u8));
    let c2 = DummyCell::new(TUVec3::splat(8u8));

    let c1_id = tree.insert(c1)?;
    let c2_id = tree.insert(c2)?;

    // Searching by position
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
```

### Bevy

Run bevy visual example:

```sh
cargo run --release --example bevy_tree --all-features
```

### no_std

`no_std` is supported, but you steel need to specify a global allocator.

See [example](https://github.com/exor2008/oktree/blob/main/examples/no_std.rs) with a custom allocator.

Run `no_std` example

```sh
cargo run --no-default-features --features bevy --example no_std
```

## Contribution guide

Feature and pull requests are welcomed.

1. Clone the Oktree [repo](https://github.com/exor2008/oktree)

```sh
git clone https://github.com/exor2008/oktree
```

2. Implement awesome feature

3. Run tests

```sh
cargo test --all-targets --all-features --release
```

4. Make sure clippy is happy

```sh
cargo clippy --all-targets --all-features
```

5. Run examples

```sh
cargo run --all-features --example simple
cargo run --all-features --example bevy_tree
cargo run --no-default-features --features bevy --example no_std
```

6. Run benchmark

```sh
cargo bench --all-features
```

7. Check the docs

```sh
cargo doc --no-deps --open --all-features
```

8. Start pull request
