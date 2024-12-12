use std::time::Duration;

use bevy::{color::palettes::css::RED, math::Vec3A, prelude::*};
use oktree::prelude::*;
use rand::Rng;

const RANGE: u32 = 256;
const SIZE: u32 = 16;
const COUNTER: usize = 1024;
//const COUNTER: usize = 3;
const SPAWN_VOLUME_FREQUENCY: f64 = 0.05;
const SPAWN_FREQUENCY: Duration = Duration::from_millis(10);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, spawn_points)
        .add_systems(Update, (draw_nodes, draw_elements))
        .run();
}

fn setup(mut commands: Commands) {
    let tree = Octree::from_aabb(Aabb::new_unchecked(TUVec3::splat(RANGE / 2), RANGE / 2));
    commands.insert_resource(Tree(tree));

    commands.insert_resource(SpawnTimer {
        timer: Timer::new(SPAWN_FREQUENCY, TimerMode::Repeating),
    });

    commands.insert_resource(Mode::Insert);

    commands.insert_resource(Counter(0));

    let position = Transform::from_xyz(-(RANGE as f32), 0.0, (RANGE / 2) as f32)
        .looking_at(Vec3::splat((RANGE / 2) as f32), Vec3::Z);

    commands.spawn((Name::new("Camera"), Camera3d::default(), position));
}

fn draw_nodes(mut gizmos: Gizmos, tree: Res<Tree>) {
    for node in tree.0.iter_nodes() {
        let scale = node.aabb.size() as f32;
        let transform =
            Transform::from_translation(node.aabb.center().into()).with_scale(Vec3::splat(scale));

        match node.ntype {
            NodeType::Empty => gizmos.cuboid(transform, Color::srgb(0.7, 0.7, 0.7)),
            NodeType::Leaf(_) => gizmos.cuboid(transform, Color::srgb(0.9, 0.45, 0.0)),
            NodeType::Branch(_) => (),
        };
    }
}

fn draw_elements(mut gizmos: Gizmos, tree: Res<Tree>) {
    for element in tree.0.iter() {
        let t: Vec3A = element.volume().center().into();

        gizmos.sphere(
            Isometry3d::from_translation(t),
            element.volume().size() as f32,
            RED,
        );
    }
}

fn spawn_points(
    mut tree: ResMut<Tree>,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut mode: ResMut<Mode>,
    mut counter: ResMut<Counter>,
) {
    timer.timer.tick(time.delta());

    if timer.timer.finished() {
        match *mode {
            Mode::Insert => {
                let mut rnd = rand::thread_rng();
                let position = TUVec3 {
                    x: rnd.gen_range(0..RANGE),
                    y: rnd.gen_range(0..RANGE),
                    z: rnd.gen_range(0..RANGE),
                };
                if rnd.gen_bool(SPAWN_VOLUME_FREQUENCY) {
                    let c = DummyVolume::new(position, rnd.gen_range(0..SIZE));
                    tree.0.insert(c).ok();
                } else {
                    let c = DummyCell::new(position);
                    tree.0
                        .insert(DummyVolume {
                            aabb: c.position().unit_aabb(),
                        })
                        .ok();
                }
                counter.0 += 1;
                if counter.0 >= COUNTER {
                    *mode = Mode::Remove;
                }
            }
            Mode::Remove => {
                let next = tree.0.iter_elements().next();
                match next {
                    Some(e) => {
                        let e = e.0;
                        tree.0.remove(e).ok();
                    }
                    None => {
                        counter.0 = 0;
                        *mode = Mode::Insert;
                    }
                }
            }
        }
    }
}

#[derive(Resource)]
struct Tree(Octree<u32, DummyVolume<u32>>);

#[derive(Resource)]
struct SpawnTimer {
    timer: Timer,
}

#[derive(Resource)]
enum Mode {
    Insert,
    Remove,
}

#[derive(Resource)]
struct Counter(usize);

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

struct DummyVolume<U: Unsigned> {
    aabb: Aabb<U>,
}

impl<U: Unsigned> Volume for DummyVolume<U> {
    type U = U;
    fn volume(&self) -> Aabb<Self::U> {
        self.aabb
    }
}

impl<U: Unsigned> DummyVolume<U> {
    fn new(position: TUVec3<U>, size: U) -> Self {
        DummyVolume {
            aabb: Aabb::new_unchecked(position, size),
        }
    }
}
