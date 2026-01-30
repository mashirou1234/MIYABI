pub trait Component: 'static {}

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

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Velocity {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    // Rust側の`World`オブジェクトへのOpaqueなハンドル
    extern "Rust" {
        type World;

        // Worldのメソッド
        fn get_transforms(&self) -> &[Transform];
        fn run_logic(&mut self);

        // Worldを生成してC++に所有権を渡す
        fn create_world() -> Box<World>;
    }
}

impl Component for ffi::Transform {}
impl Component for ffi::Velocity {}

// Rust内でのみ使用するデータ構造
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u64);

// C++に公開するWorld構造体
pub struct World {
    entities: Vec<Entity>,
    transforms: Vec<ffi::Transform>,
    velocities: Vec<ffi::Velocity>,
    next_entity: u64,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            transforms: Vec::new(),
            velocities: Vec::new(),
            next_entity: 0,
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }

    pub fn spawn(&mut self, transform: ffi::Transform, velocity: ffi::Velocity) -> Entity {
        let entity = self.create_entity();
        self.transforms.push(transform);
        self.velocities.push(velocity);
        entity
    }

    // C++に公開されるメソッド
    pub fn get_transforms(&self) -> &[ffi::Transform] {
        &self.transforms
    }

    pub fn run_logic(&mut self) {
        let dt = 0.016; // 60FPS相当の固定ステップ
        for i in 0..self.transforms.len() {
            self.transforms[i].position.x += self.velocities[i].x * dt;
            self.transforms[i].position.y += self.velocities[i].y * dt;
            self.transforms[i].position.z += self.velocities[i].z * dt;
        }
    }
}

// C++側から呼び出される、Worldを生成する関数
fn create_world() -> Box<World> {
    let mut world = World::new();

    // 初期オブジェクトをワールドに追加
    world.spawn(
        ffi::Transform {
            position: ffi::Vec3 { x: -0.5, y: 0.0, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        },
        ffi::Velocity { x: 0.1, y: 0.0, z: 0.0 },
    );
    world.spawn(
        ffi::Transform {
            position: ffi::Vec3 { x: 0.5, y: 0.0, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        },
        ffi::Velocity { x: -0.1, y: 0.0, z: 0.0 },
    );

    Box::new(world)
}

