use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::any::{Any};

pub trait Component: 'static + serde::Serialize + for<'de> serde::Deserialize<'de> {
    const COMPONENT_TYPE: ComponentType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Transform,
    Velocity,
    Material,
    Player, // NOTE: This remains for now to avoid breaking get_or_create_archetype
}

#[cxx::bridge]
pub mod ffi {
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
        pub rotation: Vec3,
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

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct RenderableObject {
        pub mesh_id: u32,
        pub material_id: u32,
        pub texture_id: u32,
        pub transform: Transform,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum AssetCommandType {
        LoadTexture,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct AssetCommand {
        pub request_id: u32,
        pub type_: AssetCommandType,
        pub path: String,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Vec2 {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Vec4 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct TextCommand {
        pub text: String,
        pub position: Vec2,
        pub font_size: f32,
        pub color: Vec4,
    }
}

pub struct World;

#[derive(Serialize, Deserialize, Default)]
pub struct AssetServer {
    #[serde(skip)]
    pub pending_requests: HashMap<u32, String>,
    pub texture_handle_map: HashMap<String, u32>,
    pub next_request_id: u32,
    pub next_texture_handle: u32,
}

impl AssetServer {
    pub fn new() -> Self {
        Self {
            pending_requests: HashMap::new(),
            texture_handle_map: HashMap::new(),
            next_request_id: 1,
            next_texture_handle: 1,
        }
    }

    pub fn load_texture(&mut self, path: &str) -> u32 {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Material {
    pub texture_handle: u32,
}

impl Component for ffi::Transform {
    const COMPONENT_TYPE: ComponentType = ComponentType::Transform;
}
impl Component for ffi::Velocity {
    const COMPONENT_TYPE: ComponentType = ComponentType::Velocity;
}
impl Component for Material {
    const COMPONENT_TYPE: ComponentType = ComponentType::Material;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity(pub u64);

type ComponentVec = Box<dyn Any + 'static>;

#[derive(Serialize, Deserialize)]
pub struct Archetype {
    pub types: HashSet<ComponentType>,
    #[serde(skip)]
    pub storage: HashMap<ComponentType, ComponentVec>,
    pub entity_count: usize,
}

impl Archetype {
    pub fn new(types: HashSet<ComponentType>) -> Self {
        Self {
            types,
            storage: HashMap::new(),
            entity_count: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct InternalWorld {
    pub entities: HashMap<Entity, (usize, usize)>,
    pub archetypes: Vec<Archetype>,
    pub next_entity: u64,
    
    #[serde(skip)]
    pub asset_server: AssetServer,
    #[serde(skip)]
    pub texture_map: HashMap<u32, u32>,
    #[serde(skip)]
    pub input_state: ffi::InputState,

    #[serde(skip)]
    pub renderables: Vec<ffi::RenderableObject>,
    #[serde(skip)]
    pub asset_commands: Vec<ffi::AssetCommand>,
    #[serde(skip)]
    pub text_commands: Vec<ffi::TextCommand>,
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
            text_commands: Vec::new(),
        }
    }

    pub fn get_or_create_archetype(&mut self, types: HashSet<ComponentType>) -> usize {
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
        // This part now depends on the game crate defining its components
        if types.contains(&ComponentType::Player) {
            // How to handle this? For now, the game crate will have to handle creation of its own component vectors.
            // This is a deeper architectural issue to solve later.
            // For this refactor, we assume the game crate might add its own logic here,
            // or we might need a component registration system.
            // As a temporary measure, we'll let the game crate handle it.
        }
        self.archetypes.push(archetype);
        self.archetypes.len() - 1
    }

    pub fn spawn<B: ComponentBundle>(&mut self, bundle: B) -> Entity {
        let types = B::get_component_types();
        let archetype_idx = self.get_or_create_archetype(types.clone());

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
                    let texture_id = self.texture_map.get(&material.texture_handle).cloned().unwrap_or(0);
                    self.renderables.push(ffi::RenderableObject {
                        transform: *transform,
                        mesh_id: 1,
                        material_id: 1,
                        texture_id,
                    });
                }
            }
        }
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

    pub fn process_asset_server(&mut self) {
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

