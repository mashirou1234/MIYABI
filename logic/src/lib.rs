mod paths;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use crate::ui::Button;

pub mod ui;

// Forward-declare the C++ types.
#[repr(C)]
pub struct RenderableObjectSlice {
    ptr: *const ffi::RenderableObject,
    len: usize,
}

#[repr(C)]
pub struct AssetCommandSlice {
    ptr: *const ffi::AssetCommand,
    len: usize,
}

#[repr(C)]
pub struct TextCommandSlice {
    ptr: *const ffi::TextCommand,
    len: usize,
}

#[repr(C)]
pub struct MiyabiVTable {
    create_game: extern "C" fn() -> *mut Game,
    destroy_game: extern "C" fn(*mut Game),
    serialize_game: extern "C" fn(*const Game) -> *mut c_char,
    deserialize_game: extern "C" fn(*const c_char) -> *mut Game,
    free_serialized_string: extern "C" fn(*mut c_char),
    update_game: extern "C" fn(*mut Game) -> GameState,
    get_renderables: extern "C" fn(*mut Game) -> RenderableObjectSlice,
    get_asset_commands: extern "C" fn(*mut Game) -> AssetCommandSlice,
    clear_asset_commands: extern "C" fn(*mut Game),
    notify_asset_loaded: extern "C" fn(*mut Game, u32, u32),
    update_input_state: extern "C" fn(*mut Game, *const ffi::InputState),
    get_asset_command_path_cstring: extern "C" fn(*const ffi::AssetCommand) -> *mut c_char,
    get_text_commands: extern "C" fn(*mut Game) -> TextCommandSlice,
    get_text_command_text_cstring: extern "C" fn(*const ffi::TextCommand) -> *mut c_char,
    free_cstring: extern "C" fn(*mut c_char),
}

#[no_mangle]
pub extern "C" fn get_miyabi_vtable() -> MiyabiVTable {
    MiyabiVTable {
        create_game,
        destroy_game,
        serialize_game,
        deserialize_game,
        free_serialized_string,
        update_game,
        get_renderables,
        get_asset_commands,
        clear_asset_commands,
        notify_asset_loaded,
        update_input_state,
        get_asset_command_path_cstring,
        get_text_commands,
        get_text_command_text_cstring,
        free_cstring,
    }
}

pub trait Component: 'static + serde::Serialize + for<'de> serde::Deserialize<'de> {
    const COMPONENT_TYPE: ComponentType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Transform,
    Velocity,
    Material,
    Player,
    Button,
    Physics,
    Sprite,
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
        pub s_key: bool,
        pub p_key: bool,
        pub mouse_pos: Vec2,
        pub mouse_clicked: bool,
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

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct CollisionEvent {
        pub bodyA: u64,
        pub bodyB: u64,
    }

    unsafe extern "C++" {
        include!("miyabi/bridge.h");

        // Audio
        fn play_sound(path: &str);

        // Physics
        fn create_dynamic_box_body(x: f32, y: f32, width: f32, height: f32) -> u64;
        fn create_static_box_body(x: f32, y: f32, width: f32, height: f32) -> u64;
        fn get_body_position(id: u64) -> Vec2;
        fn get_collision_events() -> &'static [CollisionEvent];

        #[cfg(feature = "performance_test")]
        fn get_performance_test_sprite_count() -> u32;
    }

    extern "Rust" {
        #[cfg(feature = "performance_test")]
        fn get_sprite_count() -> u32;
    }
}

#[cfg(feature = "performance_test")]
fn get_sprite_count() -> u32 {
    unsafe {
        ffi::get_performance_test_sprite_count()
    }
}


// Main game state
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
    MainMenu,
    InGame,
    SpriteStressTest,
    PhysicsStressTest,
}

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
pub struct Player;
impl Component for Player {
    const COMPONENT_TYPE: ComponentType = ComponentType::Player;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sprite;
impl Component for Sprite {
    const COMPONENT_TYPE: ComponentType = ComponentType::Sprite;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicsBody {
    pub id: u64,
}
impl Component for PhysicsBody {
    const COMPONENT_TYPE: ComponentType = ComponentType::Physics;
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
}

impl InternalWorld {
    pub fn new() -> Self {
        InternalWorld {
            entities: HashMap::new(),
            archetypes: Vec::new(),
            next_entity: 0,
        }
    }

    pub fn get_or_create_archetype(&mut self, types: HashSet<ComponentType>) -> usize {
        if let Some(idx) = self.archetypes.iter().position(|arch| arch.types == types) {
            return idx;
        }
        let mut archetype = Archetype::new(types.clone());
        if types.contains(&ComponentType::Transform) {
            archetype.storage.insert(
                ComponentType::Transform,
                Box::new(Vec::<ffi::Transform>::new()),
            );
        }
        if types.contains(&ComponentType::Velocity) {
            archetype.storage.insert(
                ComponentType::Velocity,
                Box::new(Vec::<ffi::Velocity>::new()),
            );
        }
        if types.contains(&ComponentType::Material) {
            archetype
                .storage
                .insert(ComponentType::Material, Box::new(Vec::<Material>::new()));
        }
        if types.contains(&ComponentType::Player) {
            archetype
                .storage
                .insert(ComponentType::Player, Box::new(Vec::<Player>::new()));
        }
        if types.contains(&ComponentType::Button) {
            archetype
                .storage
                .insert(ComponentType::Button, Box::new(Vec::<Button>::new()));
        }
        if types.contains(&ComponentType::Physics) {
            archetype
                .storage
                .insert(ComponentType::Physics, Box::new(Vec::<PhysicsBody>::new()));
        }
        if types.contains(&ComponentType::Sprite) {
            archetype
                .storage
                .insert(ComponentType::Sprite, Box::new(Vec::<Sprite>::new()));
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
        self.entities
            .insert(entity, (archetype_idx, entity_idx_in_archetype));
        entity
    }

    pub fn clear_entities_of_component(&mut self, component_type: ComponentType) {
        // This is a simplified and potentially slow implementation.
        // A more robust ECS would have faster ways to do this.
        let mut entities_to_remove = Vec::new();

        // Find all entities that have the component
        for (entity, (archetype_idx, _)) in &self.entities {
            if self.archetypes[*archetype_idx]
                .types
                .contains(&component_type)
            {
                entities_to_remove.push(*entity);
            }
        }

        for entity in entities_to_remove {
            // This is a placeholder for a proper entity removal implementation.
            // For now, we are just removing it from the map, but not cleaning up
            // the component data in the archetype, which will lead to memory leaks.
            // This needs to be addressed in a future refactoring of the ECS.
            self.entities.remove(&entity);
        }

        // Also clear the archetypes that are now empty
        for archetype in self.archetypes.iter_mut() {
            if archetype.types.contains(&component_type) {
                archetype.entity_count = 0;
                for storage in archetype.storage.values_mut() {
                    // This is a dynamic way of clearing a vector of any type.
                    // It's a bit of a hack, but it works for now.
                    // A proper implementation would have a trait with a clear method.
                    if let Some(vec) = storage.downcast_mut::<Vec<ffi::Transform>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<ffi::Velocity>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Material>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Player>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Button>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<PhysicsBody>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Sprite>>() {
                        vec.clear();
                    }
                }
            }
        }
    }
}

pub trait ComponentBundle {
    fn get_component_types() -> HashSet<ComponentType>
    where
        Self: Sized;
    fn push_to_storage(self, archetype: &mut Archetype);
}

impl<T: Component> ComponentBundle for (T,) {
    fn get_component_types() -> HashSet<ComponentType> {
        let mut types = HashSet::new();
        types.insert(T::COMPONENT_TYPE);
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec = archetype
            .storage
            .get_mut(&T::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .unwrap();
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
        let vec_t = archetype
            .storage
            .get_mut(&T::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .unwrap();
        vec_t.push(self.0);
        let vec_u = archetype
            .storage
            .get_mut(&U::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<U>>()
            .unwrap();
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
        let vec_t = archetype
            .storage
            .get_mut(&T::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .unwrap();
        vec_t.push(self.0);
        let vec_u = archetype
            .storage
            .get_mut(&U::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<U>>()
            .unwrap();
        vec_u.push(self.1);
        let vec_v = archetype
            .storage
            .get_mut(&V::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<V>>()
            .unwrap();
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
        let vec_t = archetype
            .storage
            .get_mut(&T::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .unwrap();
        vec_t.push(self.0);
        let vec_u = archetype
            .storage
            .get_mut(&U::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<U>>()
            .unwrap();
        vec_u.push(self.1);
        let vec_v = archetype
            .storage
            .get_mut(&V::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<V>>()
            .unwrap();
        vec_v.push(self.2);
        let vec_w = archetype
            .storage
            .get_mut(&W::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<W>>()
            .unwrap();
        vec_w.push(self.3);
    }
}

impl<T: Component, U: Component, V: Component, W: Component, X: Component> ComponentBundle
    for (T, U, V, W, X)
{
    fn get_component_types() -> HashSet<ComponentType> {
        let mut types = HashSet::new();
        types.insert(T::COMPONENT_TYPE);
        types.insert(U::COMPONENT_TYPE);
        types.insert(V::COMPONENT_TYPE);
        types.insert(W::COMPONENT_TYPE);
        types.insert(X::COMPONENT_TYPE);
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec_t = archetype
            .storage
            .get_mut(&T::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .unwrap();
        vec_t.push(self.0);
        let vec_u = archetype
            .storage
            .get_mut(&U::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<U>>()
            .unwrap();
        vec_u.push(self.1);
        let vec_v = archetype
            .storage
            .get_mut(&V::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<V>>()
            .unwrap();
        vec_v.push(self.2);
        let vec_w = archetype
            .storage
            .get_mut(&W::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<W>>()
            .unwrap();
        vec_w.push(self.3);
        let vec_x = archetype
            .storage
            .get_mut(&X::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<X>>()
            .unwrap();
        vec_x.push(self.4);
    }
}

impl<T: Component, U: Component, V: Component, W: Component, X: Component, Y: Component>
    ComponentBundle for (T, U, V, W, X, Y)
{
    fn get_component_types() -> HashSet<ComponentType> {
        let mut types = HashSet::new();
        types.insert(T::COMPONENT_TYPE);
        types.insert(U::COMPONENT_TYPE);
        types.insert(V::COMPONENT_TYPE);
        types.insert(W::COMPONENT_TYPE);
        types.insert(X::COMPONENT_TYPE);
        types.insert(Y::COMPONENT_TYPE);
        types
    }

    fn push_to_storage(self, archetype: &mut Archetype) {
        let vec_t = archetype
            .storage
            .get_mut(&T::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<T>>()
            .unwrap();
        vec_t.push(self.0);
        let vec_u = archetype
            .storage
            .get_mut(&U::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<U>>()
            .unwrap();
        vec_u.push(self.1);
        let vec_v = archetype
            .storage
            .get_mut(&V::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<V>>()
            .unwrap();
        vec_v.push(self.2);
        let vec_w = archetype
            .storage
            .get_mut(&W::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<W>>()
            .unwrap();
        vec_w.push(self.3);
        let vec_x = archetype
            .storage
            .get_mut(&X::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<X>>()
            .unwrap();
        vec_x.push(self.4);
        let vec_y = archetype
            .storage
            .get_mut(&Y::COMPONENT_TYPE)
            .unwrap()
            .downcast_mut::<Vec<Y>>()
            .unwrap();
        vec_y.push(self.5);
    }
}

// The main game object
#[derive(Serialize, Deserialize)]
pub struct Game {
    pub world: InternalWorld,
    pub current_state: GameState,

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
    #[serde(skip)]
    pub collision_events: Vec<ffi::CollisionEvent>,
}

// Temporary alias for backward compatibility with sample_game
pub type World = Game;

// use sample_game::{setup_game_world, update_game_logic};

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            world: InternalWorld::new(),
            current_state: GameState::MainMenu,
            asset_server: AssetServer::new(),
            texture_map: HashMap::new(),
            input_state: ffi::InputState {
                up: false,
                down: false,
                left: false,
                right: false,
                s_key: false,
                mouse_pos: ffi::Vec2 { x: 0.0, y: 0.0 },
                mouse_clicked: false,
            },
            renderables: Vec::new(),
            asset_commands: Vec::new(),
            text_commands: Vec::new(),
            collision_events: Vec::new(),
        };
        // Setup the initial state
        game.setup_main_menu();
        game
    }

    pub fn update(&mut self) {
        match self.current_state {
            GameState::MainMenu => self.update_main_menu(),
            GameState::InGame => self.update_in_game(),
            GameState::SpriteStressTest => self.update_sprite_stress_test(),
            GameState::PhysicsStressTest => self.update_physics_stress_test(),
        }
    }

    fn update_main_menu(&mut self) {
        self.text_commands.clear();
        self.renderables.clear();

        if self.input_state.s_key {
            self.current_state = GameState::SpriteStressTest;
            self.setup_sprite_stress_test();
            return;
        }

        if self.input_state.p_key {
            self.current_state = GameState::PhysicsStressTest;
            self.setup_physics_stress_test();
            return;
        }

        // The UI system now handles drawing and interactions for buttons.
        ui::ui_system(self);
    }

    fn setup_sprite_stress_test(&mut self) {
        self.world.clear_entities_of_component(ComponentType::Button);
        self.world.clear_entities_of_component(ComponentType::Physics);

        let mut rng = rand::thread_rng();
        let player_texture = self.asset_server.load_texture("assets/player.png");

        #[cfg(feature = "performance_test")]
        let sprite_count = get_sprite_count();
        #[cfg(not(feature = "performance_test"))]
        let sprite_count = 10000;

        for _ in 0..sprite_count {
            self.world.spawn((
                ffi::Transform {
                    position: ffi::Vec3 {
                        x: rng.gen_range(0.0..800.0),
                        y: rng.gen_range(0.0..600.0),
                        z: 0.0,
                    },
                    rotation: ffi::Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: ffi::Vec3 {
                        x: 10.0,
                        y: 10.0,
                        z: 1.0,
                    },
                },
                Material {
                    texture_handle: player_texture,
                },
                Sprite,
            ));
        }
    }

    fn update_sprite_stress_test(&mut self) {
        self.text_commands.clear();
        self.process_asset_server();
        self.build_renderables();
    }

    fn setup_physics_stress_test(&mut self) {
        self.world.clear_entities_of_component(ComponentType::Button);
        self.world.clear_entities_of_component(ComponentType::Sprite);

        const PPM: f32 = 50.0; // Pixels Per Meter
        const SCREEN_WIDTH: f32 = 800.0;
        const SCREEN_HEIGHT: f32 = 600.0;
        const WALL_THICKNESS: f32 = 10.0;

        let ground_texture = self.asset_server.load_texture("assets/test.png");
        let box_texture = self.asset_server.load_texture("assets/player.png");

        // Create container walls
        let walls = [
            // Bottom
            (SCREEN_WIDTH / 2.0, WALL_THICKNESS / 2.0, SCREEN_WIDTH, WALL_THICKNESS),
            // Top
            (SCREEN_WIDTH / 2.0, SCREEN_HEIGHT - WALL_THICKNESS / 2.0, SCREEN_WIDTH, WALL_THICKNESS),
            // Left
            (WALL_THICKNESS / 2.0, SCREEN_HEIGHT / 2.0, WALL_THICKNESS, SCREEN_HEIGHT),
            // Right
            (SCREEN_WIDTH - WALL_THICKNESS / 2.0, SCREEN_HEIGHT / 2.0, WALL_THICKNESS, SCREEN_HEIGHT),
        ];

        for (x, y, w, h) in walls {
            let body_id = ffi::create_static_box_body(x / PPM, y / PPM, w / PPM, h / PPM);
            self.world.spawn((
                ffi::Transform {
                    position: ffi::Vec3 { x, y, z: 0.0 },
                    rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                    scale: ffi::Vec3 { x: w, y: h, z: 1.0 },
                },
                PhysicsBody { id: body_id },
                Material { texture_handle: ground_texture },
            ));
        }

        // Create dynamic falling boxes
        let mut rng = rand::thread_rng();
        let box_size = 10.0;
        for _ in 0..500 {
            let x = rng.gen_range((WALL_THICKNESS + box_size)..(SCREEN_WIDTH - WALL_THICKNESS - box_size));
            let y = rng.gen_range((WALL_THICKNESS + box_size)..(SCREEN_HEIGHT - WALL_THICKNESS - box_size));

            let body_id = ffi::create_dynamic_box_body(x / PPM, y / PPM, box_size / PPM, box_size / PPM);
            self.world.spawn((
                ffi::Transform {
                    position: ffi::Vec3 { x, y, z: 0.0 },
                    rotation: ffi::Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                    scale: ffi::Vec3 { x: box_size, y: box_size, z: 1.0 },
                },
                PhysicsBody { id: body_id },
                Material { texture_handle: box_texture },
            ));
        }
    }

    fn update_physics_stress_test(&mut self) {
        self.text_commands.clear();
        self.poll_physics_events();
        self.sync_physics_to_render();
        self.process_asset_server();
        self.build_renderables();
    }

    fn setup_in_game(&mut self) {
        const PPM: f32 = 50.0; // Pixels Per Meter

        // Create a static ground body
        let ground_width = 800.0;
        let ground_height = 50.0;
        let ground_x = 400.0;
        let ground_y = 25.0;
        let ground_body_id = ffi::create_static_box_body(
            ground_x / PPM,
            ground_y / PPM,
            ground_width / PPM,
            ground_height / PPM,
        );

        self.world.spawn((
            ffi::Transform {
                position: ffi::Vec3 {
                    x: ground_x,
                    y: ground_y,
                    z: 0.0,
                },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: ground_width,
                    y: ground_height,
                    z: 1.0,
                },
            },
            PhysicsBody {
                id: ground_body_id,
            },
            Material {
                texture_handle: self.asset_server.load_texture("assets/test.png"),
            },
        ));

        // Create a dynamic falling box
        let box_width = 50.0;
        let box_height = 50.0;
        let box_x = 400.0;
        let box_y = 500.0;
        let box_body_id = ffi::create_dynamic_box_body(
            box_x / PPM,
            box_y / PPM,
            box_width / PPM,
            box_height / PPM,
        );

        self.world.spawn((
            ffi::Transform {
                position: ffi::Vec3 {
                    x: box_x,
                    y: box_y,
                    z: 0.0,
                },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: box_width,
                    y: box_height,
                    z: 1.0,
                },
            },
            PhysicsBody { id: box_body_id },
            Material {
                texture_handle: self.asset_server.load_texture("assets/player.png"),
            },
        ));
    }

    fn run_input_system(&mut self) {
        // For now, we disable player input to observe physics
    }

    fn poll_physics_events(&mut self) {
        self.collision_events.clear();
        let events = ffi::get_collision_events();
        self.collision_events.extend_from_slice(events);
    }

    fn update_in_game(&mut self) {
        self.text_commands.clear();
        self.run_input_system();
        self.poll_physics_events();
        self.sync_physics_to_render(); // New physics system
                                      // self.run_movement_system(); // Old movement system is not needed for now
        self.process_asset_server();
        self.build_renderables();
    }

    fn sync_physics_to_render(&mut self) {
        const PPM: f32 = 50.0; // Pixels Per Meter

        for archetype in &mut self.world.archetypes {
            if archetype.types.contains(&ComponentType::Physics)
                && archetype.types.contains(&ComponentType::Transform)
            {
                let mut transform_storage =
                    archetype.storage.remove(&ComponentType::Transform).unwrap();
                let physics_storage = archetype.storage.get(&ComponentType::Physics).unwrap();

                let transforms = transform_storage
                    .downcast_mut::<Vec<ffi::Transform>>()
                    .unwrap();
                let physics_bodies = physics_storage.downcast_ref::<Vec<PhysicsBody>>().unwrap();

                for i in 0..archetype.entity_count {
                    let body_id = physics_bodies[i].id;
                    let new_pos_meters = ffi::get_body_position(body_id);

                    transforms[i].position.x = new_pos_meters.x * PPM;
                    transforms[i].position.y = new_pos_meters.y * PPM;
                }

                archetype
                    .storage
                    .insert(ComponentType::Transform, transform_storage);
            }
        }
    }

    // --- Old systems to be removed or refactored ---

    fn setup_main_menu(&mut self) {
        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 400.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Start Game".to_string(),
            action: ui::ButtonAction::StartGame,
        },));
    }

    pub fn build_renderables(&mut self) {
        self.renderables.clear();
        for archetype in &self.world.archetypes {
            let has_transform = archetype.types.contains(&ComponentType::Transform);
            let has_material = archetype.types.contains(&ComponentType::Material);

            if has_transform && has_material {
                let transforms = archetype
                    .storage
                    .get(&ComponentType::Transform)
                    .unwrap()
                    .downcast_ref::<Vec<ffi::Transform>>()
                    .unwrap();
                let materials = archetype
                    .storage
                    .get(&ComponentType::Material)
                    .unwrap()
                    .downcast_ref::<Vec<Material>>()
                    .unwrap();

                for (transform, material) in transforms.iter().zip(materials.iter()) {
                    let texture_id = self
                        .texture_map
                        .get(&material.texture_handle)
                        .cloned()
                        .unwrap_or(0);
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

// --- VTable Functions ---

#[no_mangle]
pub extern "C" fn create_game() -> *mut Game {
    Box::into_raw(Box::new(Game::new()))
}

#[no_mangle]
pub extern "C" fn destroy_game(game: *mut Game) {
    if !game.is_null() {
        unsafe {
            Box::from_raw(game);
        }
    }
}

#[no_mangle]
pub extern "C" fn serialize_game(game: *const Game) -> *mut c_char {
    if game.is_null() {
        return ptr::null_mut();
    }
    let game = unsafe { &*game };
    let serialized = serde_json::to_string(game).unwrap();
    CString::new(serialized).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn deserialize_game(json: *const c_char) -> *mut Game {
    if json.is_null() {
        return ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(json) };
    let r_str = c_str.to_str().unwrap();
    let mut game: Game = serde_json::from_str(r_str).unwrap();
    // Re-initialize non-serializable fields
    game.asset_server = AssetServer::new();
    // ... etc. for other non-serde fields
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn free_serialized_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            CString::from_raw(s);
        }
    }
}

#[no_mangle]
pub extern "C" fn update_game(game: *mut Game) -> GameState {
    if game.is_null() {
        return GameState::MainMenu;
    }
    let game = unsafe { &mut *game };
    game.update();
    game.current_state
}

#[no_mangle]
pub extern "C" fn get_renderables(game: *mut Game) -> RenderableObjectSlice {
    if game.is_null() {
        return RenderableObjectSlice {
            ptr: ptr::null(),
            len: 0,
        };
    }
    let game = unsafe { &*game };
    RenderableObjectSlice {
        ptr: game.renderables.as_ptr(),
        len: game.renderables.len(),
    }
}

#[no_mangle]
pub extern "C" fn get_asset_commands(game: *mut Game) -> AssetCommandSlice {
    if game.is_null() {
        return AssetCommandSlice {
            ptr: ptr::null(),
            len: 0,
        };
    }
    let game = unsafe { &*game };
    AssetCommandSlice {
        ptr: game.asset_commands.as_ptr(),
        len: game.asset_commands.len(),
    }
}

#[no_mangle]
pub extern "C" fn clear_asset_commands(game: *mut Game) {
    if game.is_null() {
        return;
    }
    let game = unsafe { &mut *game };
    game.asset_commands.clear();
}

#[no_mangle]
pub extern "C" fn notify_asset_loaded(game: *mut Game, request_id: u32, asset_id: u32) {
    if game.is_null() {
        return;
    }
    let game = unsafe { &mut *game };
    if let Some(path) = game.asset_server.pending_requests.remove(&request_id) {
        if let Some(handle) = game.asset_server.texture_handle_map.get_mut(&path) {
            game.texture_map.insert(*handle, asset_id);
        }
    }
}

#[no_mangle]
pub extern "C" fn update_input_state(game: *mut Game, input: *const ffi::InputState) {
    if game.is_null() || input.is_null() {
        return;
    }
    let game = unsafe { &mut *game };
    let input = unsafe { &*input };
    game.input_state = *input;
}

#[no_mangle]
pub extern "C" fn get_asset_command_path_cstring(command: *const ffi::AssetCommand) -> *mut c_char {
    if command.is_null() {
        return ptr::null_mut();
    }
    let command = unsafe { &*command };
    CString::new(command.path.as_str()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn get_text_commands(game: *mut Game) -> TextCommandSlice {
    if game.is_null() {
        return TextCommandSlice {
            ptr: ptr::null(),
            len: 0,
        };
    }
    let game = unsafe { &*game };
    TextCommandSlice {
        ptr: game.text_commands.as_ptr(),
        len: game.text_commands.len(),
    }
}

#[no_mangle]
pub extern "C" fn get_text_command_text_cstring(command: *const ffi::TextCommand) -> *mut c_char {
    if command.is_null() {
        return ptr::null_mut();
    }
    let command = unsafe { &*command };
    CString::new(command.text.as_str()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn free_cstring(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            CString::from_raw(s);
        }
    }
}