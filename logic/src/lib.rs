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
    Material,
    Player, // Player component
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

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
    pub struct InputState {
        pub up: bool,
        pub down: bool,
        pub left: bool,
        pub right: bool,
    }

    // New renderable object struct as per DESIGN_Renderer.md
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct RenderableObject {
        pub mesh_id: u32,
        pub material_id: u32,
        pub texture_id: u32, // texture_idを追加
        pub transform: Transform,
    }

    // Commands for asset loading
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum AssetCommandType {
        LoadTexture,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct AssetCommand {
        pub request_id: u32, // request_idを追加
        pub type_: AssetCommandType,
        pub path: String,
    }
}

// Opaque pointer to the World. C++ should not know its layout.
pub struct World;

// --- Asset Management ---
#[derive(Serialize, Deserialize, Default)]
struct AssetServer {
    #[serde(skip)]
    pending_requests: HashMap<u32, String>, // request_id -> path
    texture_handle_map: HashMap<String, u32>, // path -> handle
    next_request_id: u32,
    next_texture_handle: u32,
}

impl AssetServer {
    fn new() -> Self {
        Self {
            pending_requests: HashMap::new(),
            texture_handle_map: HashMap::new(),
            next_request_id: 1,
            next_texture_handle: 1,
        }
    }

    // Returns a handle, which might not be loaded yet.
    fn load_texture(&mut self, path: &str) -> u32 {
        if let Some(handle) = self.texture_handle_map.get(path) {
            return *handle;
        }

        let request_id = self.next_request_id;
        self.next_request_id += 1;
        
        let handle = self.next_texture_handle;
        self.next_texture_handle += 1;

        self.pending_requests.insert(request_id, path.to_string());
        self.texture_handle_map.insert(path.to_string(), handle);
        
        handle
    }
}


// --- Components & Entity ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Player;
impl Component for Player {
    const COMPONENT_TYPE: ComponentType = ComponentType::Player;
}

// The actual material data, for now just a texture handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Material {
    pub texture_handle: u32,
}

use std::any::{Any};
impl Component for ffi::Transform {
    const COMPONENT_TYPE: ComponentType = ComponentType::Transform;
}
impl Component for ffi::Velocity {
    const COMPONENT_TYPE: ComponentType = ComponentType::Velocity;
}
impl Component for Material {
    const COMPONENT_TYPE: ComponentType = ComponentType::Material;
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
    asset_server: AssetServer,
    #[serde(skip)]
    texture_map: HashMap<u32, u32>, // texture_handle -> texture_id (from C++)
    #[serde(skip)]
    input_state: ffi::InputState,

    #[serde(skip)]
    renderables: Vec<ffi::RenderableObject>,
    #[serde(skip)]
    asset_commands: Vec<ffi::AssetCommand>,
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

impl<T: Component, U: Component, V: Component> ComponentBundle for (T, U, V) {
    fn get_component_types() -> HashSet<ComponentType> {
        let mut types = HashSet::new();
        types.insert(T::COMPONENT_TYPE);
        types.insert(U::COMPONENT_TYPE);
        types.insert(V::COMPONENT_TYPE);
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec_t = archetype.storage.get_mut(&T::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<T>>().unwrap();
        vec_t.push(self.0);
        let vec_u = archetype.storage.get_mut(&U::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<U>>().unwrap();
        vec_u.push(self.1);
        let vec_v = archetype.storage.get_mut(&V::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<V>>().unwrap();
        vec_v.push(self.2);
    }
}

impl<T: Component, U: Component, V: Component, W: Component> ComponentBundle for (T, U, V, W) {
    fn get_component_types() -> HashSet<ComponentType> {
        let mut types = HashSet::new();
        types.insert(T::COMPONENT_TYPE);
        types.insert(U::COMPONENT_TYPE);
        types.insert(V::COMPONENT_TYPE);
        types.insert(W::COMPONENT_TYPE);
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec_t = archetype.storage.get_mut(&T::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<T>>().unwrap();
        vec_t.push(self.0);
        let vec_u = archetype.storage.get_mut(&U::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<U>>().unwrap();
        vec_u.push(self.1);
        let vec_v = archetype.storage.get_mut(&V::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<V>>().unwrap();
        vec_v.push(self.2);
        let vec_w = archetype.storage.get_mut(&W::COMPONENT_TYPE).unwrap().downcast_mut::<Vec<W>>().unwrap();
        vec_w.push(self.3);
    }
}


impl InternalWorld {
    pub fn new() -> Self {
        InternalWorld {
            entities: HashMap::new(),
            archetypes: Vec::new(),
            next_entity: 0,
            asset_server: AssetServer::new(),
            texture_map: HashMap::new(),
            input_state: ffi::InputState { up: false, down: false, left: false, right: false },
            renderables: Vec::new(),
            asset_commands: Vec::new(),
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
        if types.contains(&ComponentType::Material) {
            archetype.storage.insert(ComponentType::Material, Box::new(Vec::<Material>::new()));
        }
        if types.contains(&ComponentType::Player) {
            archetype.storage.insert(ComponentType::Player, Box::new(Vec::<Player>::new()));
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
            let has_transform = archetype.types.contains(&ComponentType::Transform);
            let has_material = archetype.types.contains(&ComponentType::Material);

            if has_transform && has_material {
                 let transforms = archetype.storage.get(&ComponentType::Transform).unwrap().downcast_ref::<Vec<ffi::Transform>>().unwrap();
                 let materials = archetype.storage.get(&ComponentType::Material).unwrap().downcast_ref::<Vec<Material>>().unwrap();
                
                for (transform, material) in transforms.iter().zip(materials.iter()) {
                    let texture_id = self.texture_map.get(&material.texture_handle).cloned().unwrap_or(0); // 0 is default/unloaded texture
                    self.renderables.push(ffi::RenderableObject {
                        transform: *transform,
                        mesh_id: 1,     // Hardcoded
                        material_id: 1, // Hardcoded
                        texture_id,
                    });
                }
            }
        }
    }

    pub fn run_input_system(&mut self) {
        for archetype in &mut self.archetypes {
            if archetype.types.contains(&ComponentType::Player) {
                if let Some(vel_storage_any) = archetype.storage.get_mut(&ComponentType::Velocity) {
                    if let Some(velocities) = vel_storage_any.downcast_mut::<Vec<ffi::Velocity>>() {
                        for velocity in velocities {
                            velocity.x = 0.0;
                            velocity.y = 0.0;
                            let speed = 5.0;
                            if self.input_state.up {
                                velocity.y = speed;
                            }
                            if self.input_state.down {
                                velocity.y = -speed;
                            }
                            if self.input_state.left {
                                velocity.x = -speed;
                            }
                            if self.input_state.right {
                                velocity.x = speed;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn run_logic_systems(&mut self) {
        self.run_input_system();
        self.run_movement_system();
        self.process_asset_server();
    }

    pub fn run_movement_system(&mut self) {
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

    fn process_asset_server(&mut self) {
        self.asset_commands.clear();
        for (request_id, path) in self.asset_server.pending_requests.drain() {
            self.asset_commands.push(ffi::AssetCommand {
                request_id,
                type_: ffi::AssetCommandType::LoadTexture,
                path,
            });
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
pub struct AssetCommandSlice {
    pub ptr: *const ffi::AssetCommand,
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
    pub get_asset_commands: extern "C" fn(*mut World) -> AssetCommandSlice,
    pub clear_asset_commands: extern "C" fn(*mut World),
    pub notify_asset_loaded: extern "C" fn(*mut World, u32, u32), // request_id, asset_id
    pub update_input_state: extern "C" fn(*mut World, &ffi::InputState),
    pub get_asset_command_path_cstring: extern "C" fn(&ffi::AssetCommand) -> *const c_char,
    pub free_cstring: extern "C" fn(*mut c_char),
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
        get_asset_commands: rust_get_asset_commands,
        clear_asset_commands: rust_clear_asset_commands,
        notify_asset_loaded: rust_notify_asset_loaded,
        update_input_state: rust_update_input_state,
        get_asset_command_path_cstring: rust_get_asset_command_path_cstring,
        free_cstring: rust_free_cstring,
    }
}

// --- VTable Function Implementations ---

#[no_mangle]
extern "C" fn rust_update_input_state(world: *mut World, input_state: &ffi::InputState) {
    let world = unsafe { &mut *(world as *mut InternalWorld) };
    world.input_state = *input_state;
}

#[no_mangle]
extern "C" fn rust_notify_asset_loaded(world: *mut World, request_id: u32, asset_id: u32) {
    let world = unsafe { &mut *(world as *mut InternalWorld) };
    if let Some(path) = world.asset_server.pending_requests.get(&request_id) {
         if let Some(handle) = world.asset_server.texture_handle_map.get(path) {
            world.texture_map.insert(*handle, asset_id);
            println!("Rust: Asset (req_id: {}) loaded, handle: {}, asset_id: {}", request_id, handle, asset_id);
         }
    }
}


#[no_mangle]
extern "C" fn rust_get_asset_command_path_cstring(command: &ffi::AssetCommand) -> *const c_char {
    CString::new(command.path.as_str()).unwrap().into_raw()
}

#[no_mangle]
extern "C" fn rust_free_cstring(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); };
    }
}

#[no_mangle]
extern "C" fn rust_create_world() -> *mut World {
    let mut world = InternalWorld::new();

    // Player
    let player_texture_handle = world.asset_server.load_texture("assets/player.png");
    world.spawn((
        ffi::Transform {
            position: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 0.2, y: 0.2, z: 0.2 },
        },
        ffi::Velocity { x: 0.0, y: 0.0, z: 0.0 },
        Material { texture_handle: player_texture_handle },
        Player,
    ));

    // Other entity
    let test_texture_handle = world.asset_server.load_texture("assets/test.png");
    world.spawn((
        ffi::Transform {
            position: ffi::Vec3 { x: 0.5, y: 0.5, z: 0.0 },
            rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: ffi::Vec3 { x: 0.2, y: 0.2, z: 0.2 },
        },
        ffi::Velocity { x: -0.1, y: -0.1, z: 0.0 },
        Material { texture_handle: test_texture_handle },
    ));

    // After spawning, process the asset server to generate commands
    world.process_asset_server();

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
    world.process_asset_server();
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
extern "C" fn rust_get_asset_commands(world: *mut World) -> AssetCommandSlice {
    let world = unsafe { &mut *(world as *mut InternalWorld) };
    AssetCommandSlice {
        ptr: world.asset_commands.as_ptr(),
        len: world.asset_commands.len(),
    }
}

#[no_mangle]
extern "C" fn rust_clear_asset_commands(world: *mut World) {
    let world = unsafe { &mut *(world as *mut InternalWorld) };
    world.asset_commands.clear();
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
    
    // Re-initialize non-serializable fields
    world.asset_server = AssetServer::new();
    world.texture_map = HashMap::new();
    world.renderables = Vec::new();
    world.asset_commands = Vec::new();
    
    for archetype in &mut world.archetypes {
        archetype.storage = HashMap::new();
        if archetype.types.contains(&ComponentType::Transform) {
            archetype.storage.insert(ComponentType::Transform, Box::new(Vec::<ffi::Transform>::new()));
        }
        if archetype.types.contains(&ComponentType::Velocity) {
            archetype.storage.insert(ComponentType::Velocity, Box::new(Vec::<ffi::Velocity>::new()));
        }
        if archetype.types.contains(&ComponentType::Material) {
            archetype.storage.insert(ComponentType::Material, Box::new(Vec::<Material>::new()));
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

