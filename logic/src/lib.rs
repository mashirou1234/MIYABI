use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::os::raw::c_char;
use std::ffi::CString;

pub trait Component: 'static + serde::Serialize + for<'de> serde::Deserialize<'de> {
    const COMPONENT_TYPE: ComponentType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Transform,
    Velocity,
}

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
        pub rotation: Vec3, // 一旦オイラー角で?
        pub scale: Vec3,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Velocity {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    // New renderable object struct as per DESIGN_Renderer.md
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct RenderableObject {
        pub mesh_id: u32,
        pub material_id: u32,
        pub transform: Transform,
    }
}

// Opaque pointer to the World. C++ should not know its layout.
pub struct World;

use std::any::{Any};
impl Component for ffi::Transform {
    const COMPONENT_TYPE: ComponentType = ComponentType::Transform;
}
impl Component for ffi::Velocity {
    const COMPONENT_TYPE: ComponentType = ComponentType::Velocity;
}

// Rust内でのみ使用するデータ構造
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity(u64);

type ComponentVec = Box<dyn Any + 'static>;

#[derive(Serialize, Deserialize)]
pub struct Archetype {
    types: HashSet<ComponentType>,
    #[serde(skip)]
    storage: HashMap<ComponentType, ComponentVec>,
    entity_count: usize,
}

impl Archetype {
    fn new(types: HashSet<ComponentType>) -> Self {
        Self {
            types,
            storage: HashMap::new(),
            entity_count: 0,
        }
    }
}

// Internal World struct. The `World` type alias is the opaque type.
#[derive(Serialize, Deserialize)]
struct InternalWorld {
    entities: HashMap<Entity, (usize, usize)>, // Map Entity -> (archetype_idx, entity_idx_in_archetype)
    archetypes: Vec<Archetype>,
    next_entity: u64,
    #[serde(skip)]
    renderables: Vec<ffi::RenderableObject>,
}

pub trait ComponentBundle {
    fn get_component_types() -> HashSet<ComponentType> where Self: Sized;
    fn push_to_storage(self, archetype: &mut Archetype);
}

impl<T: Component> ComponentBundle for (T,) {
    fn get_component_types() -> HashSet<ComponentType> {
        let mut types = HashSet::new();
        types.insert(T::COMPONENT_TYPE);
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec = archetype.storage.get_mut(&T::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<T>>().unwrap();
        vec.push(self.0);
    }
}

impl<T: Component, U: Component> ComponentBundle for (T, U) {
    fn get_component_types() -> HashSet<ComponentType> {
        let mut types = HashSet::new();
        types.insert(T::COMPONENT_TYPE);
        types.insert(U::COMPONENT_TYPE);
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec_t = archetype.storage.get_mut(&T::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<T>>().unwrap();
        vec_t.push(self.0);
        let vec_u = archetype.storage.get_mut(&U::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<U>>().unwrap();
        vec_u.push(self.1);
    }
}

impl InternalWorld {
    pub fn new() -> Self {
        InternalWorld {
            entities: HashMap::new(),
            archetypes: Vec::new(),
            next_entity: 0,
            renderables: Vec::new(),
        }
    }

    fn get_or_create_archetype(&mut self, types: HashSet<ComponentType>) -> usize {
        if let Some(idx) = self.archetypes.iter().position(|arch| arch.types == types) {
            return idx;
        }
        let mut archetype = Archetype::new(types.clone());
        if types.contains(&ComponentType::Transform) {
            archetype.storage.insert(ComponentType::Transform, Box::new(Vec::<ffi::Transform>::new()));
        }
        if types.contains(&ComponentType::Velocity) {
            archetype.storage.insert(ComponentType::Velocity, Box::new(Vec::<ffi::Velocity>::new()));
        }
        self.archetypes.push(archetype);
        self.archetypes.len() - 1
    }

    pub fn spawn<B: ComponentBundle>(&mut self, bundle: B) -> Entity {
        let types = B::get_component_types();
        let archetype_idx = self.get_or_create_archetype(types);
        let archetype = &mut self.archetypes[archetype_idx];
        bundle.push_to_storage(archetype);
        let entity_idx_in_archetype = archetype.entity_count;
        archetype.entity_count += 1;
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        self.entities.insert(entity, (archetype_idx, entity_idx_in_archetype));
        entity
    }

    pub fn build_renderables(&mut self) {
        self.renderables.clear();
        for archetype in &self.archetypes {
            if let Some(storage) = archetype.storage.get(&ComponentType::Transform) {
                if let Some(transforms) = storage.downcast_ref::<Vec<ffi::Transform>>() {
                    for transform in transforms {
                        self.renderables.push(ffi::RenderableObject {
                            transform: *transform,
                            mesh_id: 0,     // Hardcoded for now
                            material_id: 0, // Hardcoded for now
                        });
                    }
                }
            }
        }
    }

    pub fn run_logic_systems(&mut self) {
        let dt = 0.016; // 60FPS
        for archetype in &mut self.archetypes {
            let has_transform = archetype.types.contains(&ComponentType::Transform);
            let has_velocity = archetype.types.contains(&ComponentType::Velocity);
            if has_transform && has_velocity {
                let mut trans_storage = archetype.storage.remove(&ComponentType::Transform).unwrap();
                let transforms = trans_storage.downcast_mut::<Vec<ffi::Transform>>().unwrap();
                let vel_storage = archetype.storage.get(&ComponentType::Velocity).unwrap();
                let velocities = vel_storage.downcast_ref::<Vec<ffi::Velocity>>().unwrap();
                for (transform, velocity) in transforms.iter_mut().zip(velocities.iter()) {
                    transform.position.x += velocity.x * dt;
                    transform.position.y += velocity.y * dt;
                    transform.position.z += velocity.z * dt;
                }
                archetype.storage.insert(ComponentType::Transform, trans_storage);
            }
        }
    }
}

// --- FFI VTable Implementation ---

#[repr(C)]
pub struct RenderableObjectSlice {
    pub ptr: *const ffi::RenderableObject,
    pub len: usize,
}

#[repr(C)]
pub struct MiyabiVTable {
    pub create_world: extern "C" fn() -> *mut World,
    pub destroy_world: extern "C" fn(*mut World),
    pub serialize_world: extern "C" fn(*const World) -> *const c_char,
    pub deserialize_world: extern "C" fn(*const c_char) -> *mut World,
    pub free_serialized_string: extern "C" fn(*mut c_char),
    pub run_logic_systems: extern "C" fn(*mut World),
    pub get_renderables: extern "C" fn(*mut World) -> RenderableObjectSlice,
}

#[no_mangle]
pub extern "C" fn get_miyabi_vtable() -> MiyabiVTable {
    MiyabiVTable {
        create_world: rust_create_world,
        destroy_world: rust_destroy_world,
        serialize_world: rust_serialize_world,
        deserialize_world: rust_deserialize_world,
        free_serialized_string: rust_free_serialized_string,
        run_logic_systems: rust_run_logic_systems,
        get_renderables: rust_get_renderables,
    }
}

// --- VTable Function Implementations ---

#[no_mangle]
extern "C" fn rust_create_world() -> *mut World {
    let mut world = InternalWorld::new();
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
    Box::into_raw(Box::new(world)) as *mut World
}

#[no_mangle]
extern "C" fn rust_destroy_world(world: *mut World) {
    if !world.is_null() {
        unsafe { drop(Box::from_raw(world as *mut InternalWorld)); };
    }
}

#[no_mangle]
extern "C" fn rust_run_logic_systems(world: *mut World) {
    let world = unsafe { &mut *(world as *mut InternalWorld) };
    world.run_logic_systems();
}

#[no_mangle]
extern "C" fn rust_get_renderables(world: *mut World) -> RenderableObjectSlice {
    let world = unsafe { &mut *(world as *mut InternalWorld) };
    world.build_renderables();
    RenderableObjectSlice {
        ptr: world.renderables.as_ptr(),
        len: world.renderables.len(),
    }
}

#[no_mangle]
extern "C" fn rust_serialize_world(world: *const World) -> *const c_char {
    let world = unsafe { &*(world as *const InternalWorld) };
    let serialized = serde_json::to_string(world).unwrap();
    CString::new(serialized).unwrap().into_raw()
}

#[no_mangle]
extern "C" fn rust_deserialize_world(json: *const c_char) -> *mut World {
    let c_str = unsafe { std::ffi::CStr::from_ptr(json) };
    let json_str = c_str.to_str().unwrap();
    let mut world: InternalWorld = serde_json::from_str(json_str).unwrap();
    world.renderables = Vec::new();
    for archetype in &mut world.archetypes {
        archetype.storage = HashMap::new();
        if archetype.types.contains(&ComponentType::Transform) {
            archetype.storage.insert(ComponentType::Transform, Box::new(Vec::<ffi::Transform>::new()));
        }
        if archetype.types.contains(&ComponentType::Velocity) {
            archetype.storage.insert(ComponentType::Velocity, Box::new(Vec::<ffi::Velocity>::new()));
        }
    }
    Box::into_raw(Box::new(world)) as *mut World
}

#[no_mangle]
extern "C" fn rust_free_serialized_string(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            drop(CString::from_raw(s));
        }
    };
}

