use serde::{Serialize, Deserialize};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::os::raw::c_char;
use std::ffi::CString;

pub trait Component: 'static + serde::Serialize + for<'de> serde::Deserialize<'de> {}


#[cxx::bridge]
mod ffi {
    // C++と共有するデータ構造
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Vec3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Transform {
        pub position: Vec3,
        pub rotation: Vec3, // 一旦オイラー角で
        pub scale: Vec3,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Velocity {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    // A simple command for the renderer. For now, it only supports drawing a triangle.
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct DrawTriangleCommand {
        pub transform: Transform,
    }

    // Rust側の`World`オブジェクトへのOpaqueなハンドル
    extern "Rust" {
        type World;
    }
}

use std::any::{Any};
impl Component for ffi::Transform {}
impl Component for ffi::Velocity {}

// Rust内でのみ使用するデータ構造
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity(u64);

// The actual storage for components of a given type. Stored as a Box<dyn Any>
// so we can have a collection of these of different underlying types.
type ComponentVec = Box<dyn Any + 'static>;

// An Archetype is a collection of entities that all have the same set of component types.
pub struct Archetype {
    // The set of component types that define this archetype.
    types: HashSet<TypeId>,
    // A map from a component's TypeId to its storage. The storage is a Box<dyn Any>
    // that holds a Vec<T> for the component type T.
    storage: HashMap<TypeId, ComponentVec>,
    entity_count: usize,
}

impl Archetype {
    fn new(types: HashSet<TypeId>) -> Self {
        Self {
            types,
            storage: HashMap::new(),
            entity_count: 0,
        }
    }
}

// C++に公開するWorld構造体
#[derive(Serialize, Deserialize)]
pub struct World {
    entities: HashMap<Entity, (usize, usize)>, // Map Entity -> (archetype_idx, entity_idx_in_archetype)
    archetypes: Vec<Archetype>,
    next_entity: u64,
    #[serde(skip)]
    render_commands: Vec<ffi::DrawTriangleCommand>,
}


// --- ComponentBundle Trait ---
// This allows us to pass different combinations of components to the spawn function.
pub trait ComponentBundle {
    fn get_type_ids() -> HashSet<TypeId> where Self: Sized;
    fn push_to_storage(self, archetype: &mut Archetype);
}

impl<T: Component> ComponentBundle for (T,) {
    fn get_type_ids() -> HashSet<TypeId> {
        let mut types = HashSet::new();
        types.insert(TypeId::of::<T>());
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec = archetype.storage.get_mut(&TypeId::of::<T>()).unwrap().downcast_mut::<Vec<T>>().unwrap();
        vec.push(self.0);
    }
}

impl<T: Component, U: Component> ComponentBundle for (T, U) {
    fn get_type_ids() -> HashSet<TypeId> {
        let mut types = HashSet::new();
        types.insert(TypeId::of::<T>());
        types.insert(TypeId::of::<U>());
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        // Add first component
        let vec_t = archetype.storage.get_mut(&TypeId::of::<T>()).unwrap().downcast_mut::<Vec<T>>().unwrap();
        vec_t.push(self.0);

        // Add second component
        let vec_u = archetype.storage.get_mut(&TypeId::of::<U>()).unwrap().downcast_mut::<Vec<U>>().unwrap();
        vec_u.push(self.1);
    }
}


impl World {
    pub fn new() -> Self {
        World {
            entities: HashMap::new(),
            archetypes: Vec::new(),
            next_entity: 0,
            render_commands: Vec::new(),
        }
    }
    
    fn get_or_create_archetype(&mut self, types: HashSet<TypeId>) -> usize {
        if let Some(idx) = self.archetypes.iter().position(|arch| arch.types == types) {
            return idx;
        }

        // Create a new archetype
        let mut archetype = Archetype::new(types.clone());
        // Initialize the Vec<T> for each component type.
        if types.contains(&TypeId::of::<ffi::Transform>()) {
            archetype.storage.insert(TypeId::of::<ffi::Transform>(), Box::new(Vec::<ffi::Transform>::new()));
        }
        if types.contains(&TypeId::of::<ffi::Velocity>()) {
            archetype.storage.insert(TypeId::of::<ffi::Velocity>(), Box::new(Vec::<ffi::Velocity>::new()));
        }

        self.archetypes.push(archetype);
        self.archetypes.len() - 1
    }

    pub fn spawn<B: ComponentBundle>(&mut self, bundle: B) -> Entity {
        let types = B::get_type_ids();
        let archetype_idx = self.get_or_create_archetype(types);
        let archetype = &mut self.archetypes[archetype_idx];

        // Add components to storage using the trait
        bundle.push_to_storage(archetype);

        let entity_idx_in_archetype = archetype.entity_count;
        archetype.entity_count += 1;

        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        
        self.entities.insert(entity, (archetype_idx, entity_idx_in_archetype));

        entity
    }

    pub fn build_render_commands(&mut self) {
        self.render_commands.clear();
        for archetype in &self.archetypes {
            if let Some(storage) = archetype.storage.get(&TypeId::of::<ffi::Transform>()) {
                if let Some(transforms) = storage.downcast_ref::<Vec<ffi::Transform>>() {
                    for transform in transforms {
                        self.render_commands.push(ffi::DrawTriangleCommand { transform: *transform });
                    }
                }
            }
        }
    }
    
    pub fn run_logic(&mut self) {
        let dt = 0.016; // 60FPS相当の固定ステップ

        for archetype in &mut self.archetypes {
            let has_transform = archetype.types.contains(&TypeId::of::<ffi::Transform>());
            let has_velocity = archetype.types.contains(&TypeId::of::<ffi::Velocity>());

            if has_transform && has_velocity {
                let mut trans_storage = archetype.storage.remove(&TypeId::of::<ffi::Transform>()).unwrap();
                let transforms = trans_storage.downcast_mut::<Vec<ffi::Transform>>().unwrap();

                let vel_storage = archetype.storage.get(&TypeId::of::<ffi::Velocity>()).unwrap();
                let velocities = vel_storage.downcast_ref::<Vec<ffi::Velocity>>().unwrap();

                for (transform, velocity) in transforms.iter_mut().zip(velocities.iter()) {
                    transform.position.x += velocity.x * dt;
                    transform.position.y += velocity.y * dt;
                    transform.position.z += velocity.z * dt;
                }
                
                archetype.storage.insert(TypeId::of::<ffi::Transform>(), trans_storage);
            }
        }
    }
}

// C++側から呼び出される、Worldを生成する関数
#[no_mangle]
pub extern "C" fn create_world() -> *mut World {
    let mut world = World::new();

    // 初期オブジェクトをワールドに追加
    world.spawn((
        ffi::Transform {
            position: ffi::Vec3 { x: -0.5, y: 0.0, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        },
        ffi::Velocity { x: 0.1, y: 0.0, z: 0.0 },
    ));
    world.spawn((
        ffi::Transform {
            position: ffi::Vec3 { x: 0.5, y: 0.0, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        },
        ffi::Velocity { x: -0.1, y: 0.0, z: 0.0 },
    ));

    Box::into_raw(Box::new(world))
}

#[no_mangle]
pub extern "C" fn destroy_world(world: *mut World) {
    if !world.is_null() {
        unsafe { drop(Box::from_raw(world)); };
    }
}

#[no_mangle]
pub extern "C" fn run_logic(world: *mut World) {
    let world = unsafe { &mut *world };
    world.run_logic();
}

#[repr(C)]
pub struct RenderCommands {
    pub commands: *const ffi::DrawTriangleCommand,
    pub count: usize,
}

#[no_mangle]
pub extern "C" fn build_render_commands(world: *mut World) -> RenderCommands {
    let world = unsafe { &mut *world };
    world.build_render_commands();
    RenderCommands {
        commands: world.render_commands.as_ptr(),
        count: world.render_commands.len(),
    }
}

#[no_mangle]
pub extern "C" fn serialize_world(world: *const World) -> *const c_char {
    let world = unsafe { &*world };
    let serialized = serde_json::to_string(world).unwrap();
    CString::new(serialized).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn deserialize_world(json: *const c_char) -> *mut World {
    let c_str = unsafe { std::ffi::CStr::from_ptr(json) };
    let json_str = c_str.to_str().unwrap();
    let mut world: World = serde_json::from_str(json_str).unwrap();
    // Re-initialize non-serialized fields
    world.render_commands = Vec::new();
    for archetype in &mut world.archetypes {
        archetype.storage = HashMap::new();
        if archetype.types.contains(&TypeId::of::<ffi::Transform>()) {
            archetype.storage.insert(TypeId::of::<ffi::Transform>(), Box::new(Vec::<ffi::Transform>::new()));
        }
        if archetype.types.contains(&TypeId::of::<ffi::Velocity>()) {
            archetype.storage.insert(TypeId::of::<ffi::Velocity>(), Box::new(Vec::<ffi::Velocity>::new()));
        }
    }
    Box::into_raw(Box::new(world))
}

#[no_mangle]
pub extern "C" fn free_serialized_string(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}

