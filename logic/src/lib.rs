use once_cell::sync::Lazy;
use std::sync::Mutex;

#[cxx::bridge]
mod ffi {
    // C++と共有するデータ構造
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Vec3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Transform {
        pub position: Vec3,
        pub rotation: Vec3, // 一旦オイラー角で
        pub scale: Vec3,
    }

    extern "Rust" {
        fn init_scene();
        fn run_logic();
        fn get_transforms() -> Vec<Transform>;
    }
}

// Rust内でのみ使用するデータ構造
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u64);

use std::collections::HashMap;

pub struct World {
    entities: Vec<Entity>,
    transforms: HashMap<Entity, ffi::Transform>,
    next_entity: u64,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            transforms: HashMap::new(),
            next_entity: 0,
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }

    pub fn add_transform(&mut self, entity: Entity, transform: ffi::Transform) {
        self.transforms.insert(entity, transform);
    }
}

static WORLD: Lazy<Mutex<World>> = Lazy::new(|| Mutex::new(World::new()));

fn init_scene() {
    let mut world = WORLD.lock().unwrap();
    if world.entities.is_empty() {
        let entity = world.create_entity();
        world.add_transform(
            entity,
            ffi::Transform {
                position: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                scale: ffi::Vec3 { x: 1.0, y: 1.0, z: 1.0 },
            },
        );
    }
}

fn run_logic() {
    // mainループから毎フレーム呼ばれる
    // 今後、オブジェクトを動かすなどのロジックをここに追加する
}

fn get_transforms() -> Vec<ffi::Transform> {
    let world = WORLD.lock().unwrap();
    world.transforms.values().cloned().collect()
}
