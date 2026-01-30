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

    // Rust側の`Scene`オブジェクトへのOpaqueなハンドル
    extern "Rust" {
        type Scene;

        // Sceneのメソッド
        fn get_transforms(&self) -> &[Transform];

        // Sceneを生成してC++に所有権を渡す
        fn create_scene() -> Box<Scene>;

        // 毎フレームのロジック更新（将来用）
        fn run_logic();
    }
}

// Rust内でのみ使用するデータ構造
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u64);

// C++に公開するScene構造体
pub struct Scene {
    entities: Vec<Entity>,
    transforms: Vec<ffi::Transform>,
    next_entity: u64,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            entities: Vec::new(),
            transforms: Vec::new(),
            next_entity: 0,
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        self.entities.push(entity);
        entity
    }

    pub fn add_transform(&mut self, _entity: Entity, transform: ffi::Transform) {
        self.transforms.push(transform);
    }

    // C++に公開されるメソッド
    pub fn get_transforms(&self) -> &[ffi::Transform] {
        &self.transforms
    }
}

// C++側から呼び出される、Sceneを生成する関数
fn create_scene() -> Box<Scene> {
    let mut scene = Scene::new();

    // 初期オブジェクトをシーンに追加
    let entity1 = scene.create_entity();
    scene.add_transform(
        entity1,
        ffi::Transform {
            position: ffi::Vec3 { x: -0.5, y: 0.0, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        },
    );
    let entity2 = scene.create_entity();
    scene.add_transform(
        entity2,
        ffi::Transform {
            position: ffi::Vec3 { x: 0.5, y: 0.0, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        },
    );

    Box::new(scene)
}

fn run_logic() {
    // この関数はSceneオブジェクトを引数に取るように変更する必要があるだろう
    // 例: fn run_logic(scene: &mut Scene)
}