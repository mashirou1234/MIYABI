pub mod camera;
pub mod level;
pub mod perf;
pub mod save;
use crate::ui::Button;
use rand::Rng;
use sample_game_runtime::{
    SampleGameButtonAction, SampleGameEffect, SampleGameEvent, SampleGameLoop, SampleGameRunMode,
    SampleGameState,
};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::ptr;
#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

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
    abi_version: u32,
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

const MIYABI_ABI_VERSION: u32 = (1u32 << 16) | (0u32 << 8);

#[no_mangle]
pub extern "C" fn get_miyabi_vtable() -> MiyabiVTable {
    MiyabiVTable {
        abi_version: MIYABI_ABI_VERSION,
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
    RenderMesh,
    Player,
    Obstacle,
    Button,
    Physics,
    Sprite,
    SpriteAnimation,
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
        pub w_key: bool,
        pub a_key: bool,
        pub esc_key: bool,
        pub s_key: bool,
        pub p_key: bool,
        pub u_key: bool,
        pub d_key: bool,
        pub mouse_pos: Vec2,
        pub mouse_clicked: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct RenderableObject {
        pub mesh_id: u32,
        pub material_id: u32,
        pub texture_id: u32,
        pub is_3d: bool,
        pub transform: Transform,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum AssetCommandType {
        LoadTexture,
        ReloadTexture,
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
        fn play_bgm(path: &str, looped: bool);
        fn stop_bgm();
        fn set_runtime_audio_settings(master_volume: f32, bgm_volume: f32, se_volume: f32);
        fn request_fullscreen(enabled: bool);
        fn request_window_close();
        fn consume_pending_window_close_request() -> bool;

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
    ffi::get_performance_test_sprite_count()
}

#[cfg(test)]
mod runtime_bridge {
    use super::{ffi, AtomicBool, Ordering};

    static PENDING_WINDOW_CLOSE: AtomicBool = AtomicBool::new(false);

    pub fn play_sound(_path: &str) {}

    pub fn play_bgm(_path: &str, _looped: bool) {}

    pub fn stop_bgm() {}

    pub fn set_runtime_audio_settings(_master_volume: f32, _bgm_volume: f32, _se_volume: f32) {}

    pub fn request_fullscreen(_enabled: bool) {}

    pub fn request_window_close() {
        PENDING_WINDOW_CLOSE.store(true, Ordering::Release);
    }

    pub fn consume_pending_window_close_request() -> bool {
        PENDING_WINDOW_CLOSE.swap(false, Ordering::AcqRel)
    }

    pub fn create_dynamic_box_body(_x: f32, _y: f32, _width: f32, _height: f32) -> u64 {
        0
    }

    pub fn create_static_box_body(_x: f32, _y: f32, _width: f32, _height: f32) -> u64 {
        0
    }

    pub fn get_body_position(_id: u64) -> ffi::Vec2 {
        ffi::Vec2::default()
    }

    pub fn get_collision_events() -> &'static [ffi::CollisionEvent] {
        &[]
    }
}

#[cfg(not(test))]
mod runtime_bridge {
    use super::ffi;

    pub fn play_sound(path: &str) {
        ffi::play_sound(path);
    }

    pub fn play_bgm(path: &str, looped: bool) {
        ffi::play_bgm(path, looped);
    }

    pub fn stop_bgm() {
        ffi::stop_bgm();
    }

    pub fn set_runtime_audio_settings(master_volume: f32, bgm_volume: f32, se_volume: f32) {
        ffi::set_runtime_audio_settings(master_volume, bgm_volume, se_volume);
    }

    pub fn request_fullscreen(enabled: bool) {
        ffi::request_fullscreen(enabled);
    }

    pub fn request_window_close() {
        ffi::request_window_close();
    }

    pub fn consume_pending_window_close_request() -> bool {
        ffi::consume_pending_window_close_request()
    }

    pub fn create_dynamic_box_body(x: f32, y: f32, width: f32, height: f32) -> u64 {
        ffi::create_dynamic_box_body(x, y, width, height)
    }

    pub fn create_static_box_body(x: f32, y: f32, width: f32, height: f32) -> u64 {
        ffi::create_static_box_body(x, y, width, height)
    }

    pub fn get_body_position(id: u64) -> ffi::Vec2 {
        ffi::get_body_position(id)
    }

    pub fn get_collision_events() -> &'static [ffi::CollisionEvent] {
        ffi::get_collision_events()
    }
}

// Main game state
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
    Title,
    InGame,
    Pause,
    Result,
    SpriteStressTest,
    PhysicsStressTest,
    UIStressTest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveProgress {
    pub best_score: u32,
    pub best_survival_sec: u32,
    pub total_play_count: u32,
    pub total_clear_count: u32,
}

impl Default for SaveProgress {
    fn default() -> Self {
        Self {
            best_score: 0,
            best_survival_sec: 0,
            total_play_count: 0,
            total_clear_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MovementPreset {
    ArrowKeys,
    Wasd,
}

impl Default for MovementPreset {
    fn default() -> Self {
        Self::ArrowKeys
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveSettings {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub se_volume: f32,
    pub fullscreen: bool,
    pub movement_preset: MovementPreset,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            bgm_volume: 0.8,
            se_volume: 0.8,
            fullscreen: false,
            movement_preset: MovementPreset::ArrowKeys,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveData {
    pub progress: SaveProgress,
    pub settings: SaveSettings,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            progress: SaveProgress::default(),
            settings: SaveSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunMode {
    BoxSurvival2d,
    Arena3d,
}

impl Default for RunMode {
    fn default() -> Self {
        Self::BoxSurvival2d
    }
}

impl SaveSettings {
    fn sanitized(mut self) -> Self {
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.bgm_volume = self.bgm_volume.clamp(0.0, 1.0);
        self.se_volume = self.se_volume.clamp(0.0, 1.0);
        self
    }
}

impl SaveData {
    fn sanitized(mut self) -> Self {
        self.settings = self.settings.sanitized();
        self
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct AssetServer {
    #[serde(skip)]
    pub pending_requests: HashMap<u32, PendingAssetRequest>,
    pub texture_handle_map: HashMap<String, u32>,
    pub texture_path_map: HashMap<u32, String>,
    pub texture_asset_id_map: HashMap<u32, u64>,
    pub asset_id_path_map: HashMap<u64, String>,
    pub next_request_id: u32,
    pub next_texture_handle: u32,
    pub next_asset_id: u64,
}

#[derive(Debug, Clone)]
pub struct PendingAssetRequest {
    pub path: String,
    pub command_type: ffi::AssetCommandType,
    pub dispatched: bool,
}

impl AssetServer {
    pub fn new() -> Self {
        Self {
            pending_requests: HashMap::new(),
            texture_handle_map: HashMap::new(),
            texture_path_map: HashMap::new(),
            texture_asset_id_map: HashMap::new(),
            asset_id_path_map: HashMap::new(),
            next_request_id: 1,
            next_texture_handle: 1,
            next_asset_id: 1,
        }
    }

    pub fn load_texture(&mut self, path: &str) -> u32 {
        if let Some(handle) = self.texture_handle_map.get(path) {
            return *handle;
        }

        let handle = self.next_texture_handle;
        self.next_texture_handle += 1;
        let asset_id = self.next_asset_id;
        self.next_asset_id += 1;

        self.texture_handle_map.insert(path.to_string(), handle);
        self.texture_path_map.insert(handle, path.to_string());
        self.texture_asset_id_map.insert(handle, asset_id);
        self.asset_id_path_map.insert(asset_id, path.to_string());
        self.enqueue_request(path, ffi::AssetCommandType::LoadTexture);

        handle
    }

    pub fn reimport_texture(&mut self, path: &str) -> bool {
        if !self.texture_handle_map.contains_key(path) {
            return false;
        }
        if self.has_pending_request(path) {
            return false;
        }

        self.enqueue_request(path, ffi::AssetCommandType::ReloadTexture);
        true
    }

    pub fn reimport_all_textures(&mut self) -> usize {
        let paths: Vec<String> = self.texture_handle_map.keys().cloned().collect();
        let mut queued_count = 0usize;
        for path in paths {
            if self.reimport_texture(&path) {
                queued_count += 1;
            }
        }
        queued_count
    }

    fn enqueue_request(&mut self, path: &str, command_type: ffi::AssetCommandType) {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.pending_requests.insert(
            request_id,
            PendingAssetRequest {
                path: path.to_string(),
                command_type,
                dispatched: false,
            },
        );
    }

    fn has_pending_request(&self, path: &str) -> bool {
        self.pending_requests
            .values()
            .any(|request| request.path == path)
    }

    pub fn path_for_texture_handle(&self, texture_handle: u32) -> Option<&str> {
        self.texture_path_map
            .get(&texture_handle)
            .map(|path| path.as_str())
    }

    pub fn asset_id_for_texture_handle(&self, texture_handle: u32) -> Option<u64> {
        self.texture_asset_id_map.get(&texture_handle).copied()
    }

    pub fn has_pending_request_for_texture_handle(&self, texture_handle: u32) -> bool {
        let Some(path) = self.path_for_texture_handle(texture_handle) else {
            return false;
        };
        self.has_pending_request(path)
    }

    pub fn is_registry_consistent(&self) -> bool {
        for (path, handle) in &self.texture_handle_map {
            if self.texture_path_map.get(handle) != Some(path) {
                return false;
            }
            let Some(asset_id) = self.texture_asset_id_map.get(handle) else {
                return false;
            };
            if self.asset_id_path_map.get(asset_id) != Some(path) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Material {
    pub texture_handle: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct RenderMesh {
    mesh_id: u32,
    material_id: u32,
    is_3d: bool,
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
impl Component for RenderMesh {
    const COMPONENT_TYPE: ComponentType = ComponentType::RenderMesh;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimation {
    pub frame_handles: Vec<u32>,
    pub current_frame: usize,
    pub elapsed_sec: f32,
    pub frame_duration_sec: f32,
    pub idle_speed_scale: f32,
    pub active_speed_scale: f32,
}

impl Component for SpriteAnimation {
    const COMPONENT_TYPE: ComponentType = ComponentType::SpriteAnimation;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Obstacle;
impl Component for Obstacle {
    const COMPONENT_TYPE: ComponentType = ComponentType::Obstacle;
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
        if types.contains(&ComponentType::RenderMesh) {
            archetype
                .storage
                .insert(ComponentType::RenderMesh, Box::new(Vec::<RenderMesh>::new()));
        }
        if types.contains(&ComponentType::Player) {
            archetype
                .storage
                .insert(ComponentType::Player, Box::new(Vec::<Player>::new()));
        }
        if types.contains(&ComponentType::Obstacle) {
            archetype
                .storage
                .insert(ComponentType::Obstacle, Box::new(Vec::<Obstacle>::new()));
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
        if types.contains(&ComponentType::SpriteAnimation) {
            archetype.storage.insert(
                ComponentType::SpriteAnimation,
                Box::new(Vec::<SpriteAnimation>::new()),
            );
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
                    } else if let Some(vec) = storage.downcast_mut::<Vec<RenderMesh>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Player>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Obstacle>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Button>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<PhysicsBody>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<Sprite>>() {
                        vec.clear();
                    } else if let Some(vec) = storage.downcast_mut::<Vec<SpriteAnimation>>() {
                        vec.clear();
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRegistrationMeta {
    pub stage: String,
    pub source: String,
}

#[derive(Debug, Default, Clone)]
pub struct SystemRegistry {
    systems: HashMap<String, SystemRegistrationMeta>,
}

impl SystemRegistry {
    pub fn register(&mut self, name: &str, stage: &str, source: &str) -> Result<(), String> {
        if let Some(existing) = self.systems.get(name) {
            return Err(Self::duplicate_registration_log(
                name,
                stage,
                source,
                existing.stage.as_str(),
                existing.source.as_str(),
            ));
        }

        self.systems.insert(
            name.to_string(),
            SystemRegistrationMeta {
                stage: stage.to_string(),
                source: source.to_string(),
            },
        );
        Ok(())
    }

    pub fn duplicate_registration_log(
        name: &str,
        new_stage: &str,
        new_source: &str,
        existing_stage: &str,
        existing_source: &str,
    ) -> String {
        format!(
            "[ecs][system_registry][duplicate] system={name} new_stage={new_stage} new_source={new_source} existing_stage={existing_stage} existing_source={existing_source}"
        )
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
    pub run_mode: RunMode,
    #[serde(skip)]
    pub sample_loop: SampleGameLoop,

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

    pub hp: i32,
    pub survival_time_sec: f32,
    pub avoid_count: u32,
    pub score: u32,
    pub difficulty_level: u32,
    pub result_is_clear: bool,
    pub total_play_count: u32,
    pub save_data: SaveData,

    #[serde(skip)]
    pub player_texture_handle: u32,
    #[serde(skip)]
    pub obstacle_texture_handle: u32,
    #[serde(skip)]
    pub obstacle_spawn_accumulator_sec: f32,
    #[serde(skip)]
    pub esc_was_pressed: bool,
    #[serde(skip)]
    pub p_was_pressed: bool,
    #[serde(skip)]
    pub u_was_pressed: bool,
    #[serde(skip)]
    pub asset_integrity_tick: u32,
    #[serde(skip)]
    pub reported_missing_texture_handles: HashSet<u32>,
    #[serde(skip)]
    pub reported_unresolved_texture_handles: HashSet<u32>,
    #[serde(skip)]
    pub reported_registry_inconsistency: bool,
    #[serde(skip)]
    pub save_file_path: PathBuf,
}

// Temporary alias for backward compatibility with sample_game
pub type World = Game;

// use sample_game::{setup_game_world, update_game_logic};

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const FIXED_DT_SEC: f32 = 1.0 / 60.0;
const PLAYER_SIZE: f32 = 32.0;
const PLAYER_SPEED: f32 = 260.0;
const OBSTACLE_SIZE: f32 = 28.0;
const BASE_OBSTACLE_SPEED: f32 = 120.0;
const MAX_OBSTACLES: usize = 80;
const BASE_SPAWN_INTERVAL_SEC: f32 = 1.2;
const MIN_SPAWN_INTERVAL_SEC: f32 = 0.25;
const SAVE_FILE_REL_PATH: &str = "save/save_data.json";
const LEVEL_FILE_REL_PATH: &str = "assets/levels/stage1.json";
pub(crate) const SETTINGS_STEP: f32 = 0.1;
const BGM_TRACK_PATH: &str = "assets/test_sound.wav";
const ASSET_INTEGRITY_CHECK_INTERVAL_FRAMES: u32 = 30;
const MESH_ID_QUAD_2D: u32 = 1;
const MESH_ID_ARENA_CUBE_3D: u32 = 100;
const MATERIAL_ID_TEXTURED: u32 = 1;
const MATERIAL_ID_LIT_TEXTURED_3D: u32 = 2;
const ARENA_HALF_EXTENT: f32 = 110.0;
const ARENA_WALL_THICKNESS: f32 = 8.0;
const ARENA_WALL_HEIGHT: f32 = 24.0;
const ARENA_FLOOR_Y: f32 = -18.0;
const ARENA_PLAYER_SIZE: f32 = 18.0;
const ARENA_PLAYER_MOVE_SPEED: f32 = 120.0;
const ARENA_CLEAR_TIME_SEC: f32 = 180.0;
const ARENA_OBSTACLE_SIZE: f32 = 18.0;
const ARENA_OBSTACLE_RESPAWN_Y: f32 = 42.0;
const ARENA_OBSTACLE_FALL_SPEED: f32 = 72.0;
const ARENA_OBSTACLE_LANE_X: f32 = 0.0;
const ARENA_OBSTACLE_LANE_Z: f32 = 48.0;

fn resolve_runtime_path(relative_path: &str) -> PathBuf {
    let direct = PathBuf::from(relative_path);
    if direct.exists() {
        return direct;
    }

    let repo_relative = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relative_path);
    if repo_relative.exists() {
        return repo_relative;
    }

    direct
}

fn sample_state_for_game_state(game_state: GameState) -> SampleGameState {
    match game_state {
        GameState::Title => SampleGameState::Title,
        GameState::InGame => SampleGameState::InGame,
        GameState::Pause => SampleGameState::Pause,
        GameState::Result => SampleGameState::Result,
        GameState::SpriteStressTest | GameState::PhysicsStressTest | GameState::UIStressTest => {
            SampleGameState::Title
        }
    }
}

fn sample_run_mode_for_run_mode(run_mode: RunMode) -> SampleGameRunMode {
    match run_mode {
        RunMode::BoxSurvival2d => SampleGameRunMode::BoxSurvival2d,
        RunMode::Arena3d => SampleGameRunMode::Arena3d,
    }
}

impl Game {
    pub fn new() -> Self {
        let save_file_path = PathBuf::from(SAVE_FILE_REL_PATH);
        let save_data = Self::load_save_data(&save_file_path);
        let mut game = Game {
            world: InternalWorld::new(),
            current_state: GameState::Title,
            run_mode: RunMode::BoxSurvival2d,
            sample_loop: SampleGameLoop::new(),
            asset_server: AssetServer::new(),
            texture_map: HashMap::new(),
            input_state: ffi::InputState {
                up: false,
                down: false,
                left: false,
                right: false,
                w_key: false,
                a_key: false,
                esc_key: false,
                s_key: false,
                p_key: false,
                u_key: false,
                d_key: false,
                mouse_pos: ffi::Vec2 { x: 0.0, y: 0.0 },
                mouse_clicked: false,
            },
            renderables: Vec::new(),
            asset_commands: Vec::new(),
            text_commands: Vec::new(),
            collision_events: Vec::new(),
            hp: 3,
            survival_time_sec: 0.0,
            avoid_count: 0,
            score: 0,
            difficulty_level: 1,
            result_is_clear: false,
            total_play_count: save_data.progress.total_play_count,
            save_data,
            player_texture_handle: 0,
            obstacle_texture_handle: 0,
            obstacle_spawn_accumulator_sec: 0.0,
            esc_was_pressed: false,
            p_was_pressed: false,
            u_was_pressed: false,
            asset_integrity_tick: 0,
            reported_missing_texture_handles: HashSet::new(),
            reported_unresolved_texture_handles: HashSet::new(),
            reported_registry_inconsistency: false,
            save_file_path,
        };
        // Setup the initial state
        game.setup_title_screen();
        game.apply_runtime_audio_settings();
        game.apply_runtime_fullscreen_setting();
        game
    }

    fn load_save_data(path: &Path) -> SaveData {
        match save::load_or_default::<SaveData>(path) {
            Ok(save::LoadState::Loaded(data)) => data.sanitized(),
            Ok(save::LoadState::Defaulted { data, backup_path }) => {
                if let Some(path) = backup_path {
                    eprintln!(
                        "[save] Corrupt save detected. Moved backup to {}",
                        path.display()
                    );
                }
                data.sanitized()
            }
            Err(save::SaveError::VersionMismatch { found, expected }) => {
                eprintln!(
                    "[save] Version mismatch (found={found}, expected={expected}). Starting with defaults."
                );
                SaveData::default()
            }
            Err(err) => {
                eprintln!(
                    "[save] Failed to load save from {}: {err}. Starting with defaults.",
                    path.display()
                );
                SaveData::default()
            }
        }
    }

    fn persist_save_data(&self, reason: &str) {
        if let Err(err) = save::save_to_path(&self.save_file_path, &self.save_data) {
            eprintln!(
                "[save] Failed to save data ({reason}) to {}: {err}",
                self.save_file_path.display()
            );
        }
    }

    fn apply_runtime_audio_settings(&self) {
        runtime_bridge::set_runtime_audio_settings(
            self.save_data.settings.master_volume,
            self.save_data.settings.bgm_volume,
            self.save_data.settings.se_volume,
        );
    }

    fn apply_runtime_fullscreen_setting(&self) {
        runtime_bridge::request_fullscreen(self.save_data.settings.fullscreen);
    }

    fn apply_runtime_bgm_for_state(&self) {
        match self.current_state {
            GameState::Title | GameState::InGame | GameState::Pause | GameState::Result => {
                runtime_bridge::play_bgm(BGM_TRACK_PATH, true);
            }
            _ => runtime_bridge::stop_bgm(),
        }
    }

    fn handle_reimport_shortcut(&mut self) {
        let reimport_just_pressed = self.input_state.u_key && !self.u_was_pressed;
        self.u_was_pressed = self.input_state.u_key;
        if !reimport_just_pressed {
            return;
        }

        let queued = self.asset_server.reimport_all_textures();
        if queued > 0 {
            eprintln!("[asset] queued texture reimport count={queued}");
        }
    }

    fn handle_movement_preset_shortcut(&mut self) {
        let toggle_just_pressed = self.input_state.p_key && !self.p_was_pressed;
        self.p_was_pressed = self.input_state.p_key;
        if !toggle_just_pressed {
            return;
        }

        self.save_data.settings.movement_preset = match self.save_data.settings.movement_preset {
            MovementPreset::ArrowKeys => MovementPreset::Wasd,
            MovementPreset::Wasd => MovementPreset::ArrowKeys,
        };
        self.persist_save_data("settings_changed");
    }

    fn collect_referenced_texture_handles(&self) -> HashSet<u32> {
        let mut handles = HashSet::new();
        for archetype in &self.world.archetypes {
            if !archetype.types.contains(&ComponentType::Material) {
                continue;
            }

            let Some(material_storage) = archetype.storage.get(&ComponentType::Material) else {
                continue;
            };
            let Some(materials) = material_storage.downcast_ref::<Vec<Material>>() else {
                continue;
            };

            for material in materials {
                handles.insert(material.texture_handle);
            }
        }
        handles
    }

    fn asset_integrity_registry_inconsistency_log(tick: u32) -> String {
        format!("[asset] integrity: asset_integrity_tick={tick} registry inconsistency detected")
    }

    fn asset_integrity_missing_registry_log(tick: u32, handle: u32) -> String {
        format!(
            "[asset] integrity: asset_integrity_tick={tick} missing registry for texture_handle={handle}"
        )
    }

    fn asset_integrity_unresolved_reference_log(
        tick: u32,
        handle: u32,
        asset_id: u64,
        path: &str,
        queued: bool,
    ) -> String {
        format!(
            "[asset] integrity: asset_integrity_tick={tick} unresolved reference handle={handle} asset_id={asset_id} path={path} queued_reimport={queued}"
        )
    }

    fn run_asset_integrity_check(&mut self) {
        self.asset_integrity_tick = self.asset_integrity_tick.wrapping_add(1);
        if self.asset_integrity_tick % ASSET_INTEGRITY_CHECK_INTERVAL_FRAMES != 0 {
            return;
        }

        let referenced_handles = self.collect_referenced_texture_handles();
        self.reported_missing_texture_handles
            .retain(|handle| referenced_handles.contains(handle));
        self.reported_unresolved_texture_handles
            .retain(|handle| referenced_handles.contains(handle));

        let registry_consistent = self.asset_server.is_registry_consistent();
        if !registry_consistent && !self.reported_registry_inconsistency {
            eprintln!(
                "{}",
                Self::asset_integrity_registry_inconsistency_log(self.asset_integrity_tick)
            );
            self.reported_registry_inconsistency = true;
        } else if registry_consistent && self.reported_registry_inconsistency {
            self.reported_registry_inconsistency = false;
        }

        for handle in referenced_handles {
            let Some(path) = self
                .asset_server
                .path_for_texture_handle(handle)
                .map(str::to_string)
            else {
                if self.reported_missing_texture_handles.insert(handle) {
                    eprintln!(
                        "{}",
                        Self::asset_integrity_missing_registry_log(
                            self.asset_integrity_tick,
                            handle
                        )
                    );
                }
                continue;
            };

            let asset_id = self
                .asset_server
                .asset_id_for_texture_handle(handle)
                .unwrap_or(0);
            let has_loaded_texture = self.texture_map.contains_key(&handle);
            let has_pending_request = self
                .asset_server
                .has_pending_request_for_texture_handle(handle);

            if !has_loaded_texture && !has_pending_request {
                let queued = self.asset_server.reimport_texture(&path);
                if self.reported_unresolved_texture_handles.insert(handle) || queued {
                    eprintln!(
                        "{}",
                        Self::asset_integrity_unresolved_reference_log(
                            self.asset_integrity_tick,
                            handle,
                            asset_id,
                            &path,
                            queued,
                        )
                    );
                }
            } else if has_loaded_texture {
                self.reported_unresolved_texture_handles.remove(&handle);
            }
        }
    }

    pub(crate) fn adjust_master_volume(&mut self, delta: f32) {
        let current = self.save_data.settings.master_volume;
        let next = (current + delta).clamp(0.0, 1.0);
        if (next - current).abs() > f32::EPSILON {
            self.save_data.settings.master_volume = next;
            self.apply_runtime_audio_settings();
            self.persist_save_data("settings_changed");
        }
    }

    pub(crate) fn adjust_bgm_volume(&mut self, delta: f32) {
        let current = self.save_data.settings.bgm_volume;
        let next = (current + delta).clamp(0.0, 1.0);
        if (next - current).abs() > f32::EPSILON {
            self.save_data.settings.bgm_volume = next;
            self.apply_runtime_audio_settings();
            self.persist_save_data("settings_changed");
        }
    }

    pub(crate) fn adjust_se_volume(&mut self, delta: f32) {
        let current = self.save_data.settings.se_volume;
        let next = (current + delta).clamp(0.0, 1.0);
        if (next - current).abs() > f32::EPSILON {
            self.save_data.settings.se_volume = next;
            self.apply_runtime_audio_settings();
            self.persist_save_data("settings_changed");
        }
    }

    pub(crate) fn toggle_fullscreen_setting(&mut self) {
        self.save_data.settings.fullscreen = !self.save_data.settings.fullscreen;
        self.apply_runtime_fullscreen_setting();
        self.persist_save_data("settings_changed");
    }

    fn spawn_settings_buttons(&mut self, first_row_y: f32) {
        let row_step = 56.0;
        let minus_x = 240.0;
        let plus_x = 510.0;
        let volume_button_w = 50.0;
        let button_h = 40.0;

        let rows = [
            (
                SampleGameButtonAction::MasterVolumeDown.action_id(),
                SampleGameButtonAction::MasterVolumeUp.action_id(),
                first_row_y,
            ),
            (
                SampleGameButtonAction::BgmVolumeDown.action_id(),
                SampleGameButtonAction::BgmVolumeUp.action_id(),
                first_row_y - row_step,
            ),
            (
                SampleGameButtonAction::SeVolumeDown.action_id(),
                SampleGameButtonAction::SeVolumeUp.action_id(),
                first_row_y - row_step * 2.0,
            ),
        ];

        for (down_action, up_action, y) in rows {
            self.world.spawn((Button {
                rect: ui::Rect {
                    x: minus_x,
                    y,
                    width: volume_button_w,
                    height: button_h,
                },
                text: "-".to_string(),
                action_id: down_action.to_string(),
            },));
            self.world.spawn((Button {
                rect: ui::Rect {
                    x: plus_x,
                    y,
                    width: volume_button_w,
                    height: button_h,
                },
                text: "+".to_string(),
                action_id: up_action.to_string(),
            },));
        }

        self.world.spawn((Button {
            rect: ui::Rect {
                x: 250.0,
                y: first_row_y - row_step * 3.0,
                width: 300.0,
                height: button_h,
            },
            text: "Toggle Fullscreen".to_string(),
            action_id: SampleGameButtonAction::ToggleFullscreen.action_id().to_string(),
        },));
    }

    fn push_settings_text(&mut self, first_row_y: f32) {
        let row_step = 56.0;
        let master_pct = (self.save_data.settings.master_volume * 100.0).round() as u32;
        let bgm_pct = (self.save_data.settings.bgm_volume * 100.0).round() as u32;
        let se_pct = (self.save_data.settings.se_volume * 100.0).round() as u32;
        let fullscreen = if self.save_data.settings.fullscreen {
            "ON"
        } else {
            "OFF"
        };
        let movement_preset = match self.save_data.settings.movement_preset {
            MovementPreset::ArrowKeys => "ArrowKeys",
            MovementPreset::Wasd => "WASD",
        };
        let movement_text_y = (first_row_y - row_step * 4.0 + 12.0).max(20.0);

        self.text_commands.push(ffi::TextCommand {
            text: format!("Master Volume: {master_pct}%"),
            position: ffi::Vec2 {
                x: 305.0,
                y: first_row_y + 12.0,
            },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.9,
                y: 0.9,
                z: 0.9,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!("BGM Volume: {bgm_pct}%"),
            position: ffi::Vec2 {
                x: 305.0,
                y: first_row_y - row_step + 12.0,
            },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.9,
                y: 0.9,
                z: 0.9,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!("SE Volume: {se_pct}%"),
            position: ffi::Vec2 {
                x: 305.0,
                y: first_row_y - row_step * 2.0 + 12.0,
            },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.9,
                y: 0.9,
                z: 0.9,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!("Fullscreen: {fullscreen}"),
            position: ffi::Vec2 {
                x: 315.0,
                y: first_row_y - row_step * 3.0 + 12.0,
            },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.8,
                y: 0.95,
                z: 0.8,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!("Move Preset: {movement_preset} (P to toggle)"),
            position: ffi::Vec2 {
                x: 220.0,
                y: movement_text_y,
            },
            font_size: 18.0,
            color: ffi::Vec4 {
                x: 0.95,
                y: 0.9,
                z: 0.75,
                w: 1.0,
            },
        });
    }

    fn apply_result_to_progress_and_persist(&mut self) {
        self.save_data.progress.best_score = self.save_data.progress.best_score.max(self.score);

        let survival_sec = self.survival_time_sec.floor() as u32;
        self.save_data.progress.best_survival_sec =
            self.save_data.progress.best_survival_sec.max(survival_sec);

        if self.result_is_clear {
            self.save_data.progress.total_clear_count =
                self.save_data.progress.total_clear_count.saturating_add(1);
        }

        self.persist_save_data("result_transition");
    }

    pub fn update(&mut self) {
        self.handle_reimport_shortcut();
        self.handle_movement_preset_shortcut();
        match self.current_state {
            GameState::Title => self.update_main_menu(),
            GameState::InGame => self.update_in_game(),
            GameState::Pause => self.update_pause(),
            GameState::Result => self.update_result(),
            GameState::SpriteStressTest => self.update_sprite_stress_test(),
            GameState::PhysicsStressTest => self.update_physics_stress_test(),
            GameState::UIStressTest => self.update_ui_stress_test(),
        }
        self.run_asset_integrity_check();
    }

    fn dispatch_sample_action_id(&mut self, action_id: &str) {
        let Some(action) = SampleGameButtonAction::from_action_id(action_id) else {
            eprintln!("[ui] Unknown sample action id: {action_id}");
            return;
        };
        self.dispatch_sample_event(SampleGameEvent::ButtonAction(action));
    }

    fn dispatch_sample_event(&mut self, event: SampleGameEvent) {
        let effects = self.sample_loop.dispatch(event);
        for effect in effects {
            self.apply_sample_effect(effect);
        }
    }

    fn apply_sample_effect(&mut self, effect: SampleGameEffect) {
        match effect {
            SampleGameEffect::PlayClickSound => {
                runtime_bridge::play_sound("assets/test_sound.wav");
            }
            SampleGameEffect::StartNewRun => {
                self.start_new_run();
            }
            SampleGameEffect::StartNew3dRun => {
                self.start_new_3d_run();
            }
            SampleGameEffect::ResumeRun => {
                self.clear_menu_buttons();
                self.current_state = GameState::InGame;
            }
            SampleGameEffect::SetupPauseMenu => {
                self.current_state = GameState::Pause;
                self.setup_pause_menu();
            }
            SampleGameEffect::SetupResultMenu => {
                self.current_state = GameState::Result;
                self.setup_result_menu();
            }
            SampleGameEffect::SetupTitleScreen => {
                self.setup_title_screen();
            }
            SampleGameEffect::AdjustMasterVolume(delta) => {
                self.adjust_master_volume(delta);
            }
            SampleGameEffect::AdjustBgmVolume(delta) => {
                self.adjust_bgm_volume(delta);
            }
            SampleGameEffect::AdjustSeVolume(delta) => {
                self.adjust_se_volume(delta);
            }
            SampleGameEffect::ToggleFullscreen => {
                self.toggle_fullscreen_setting();
            }
            SampleGameEffect::RequestWindowClose => {
                runtime_bridge::request_window_close();
            }
        }
    }

    fn update_main_menu(&mut self) {
        self.text_commands.clear();
        self.renderables.clear();
        self.text_commands.push(ffi::TextCommand {
            text: "MIYABI Box Survival".to_string(),
            position: ffi::Vec2 { x: 255.0, y: 520.0 },
            font_size: 36.0,
            color: ffi::Vec4 {
                x: 0.95,
                y: 0.95,
                z: 0.95,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: "Arrow Keys / WASD: Move / ESC: Pause".to_string(),
            position: ffi::Vec2 { x: 190.0, y: 480.0 },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.8,
                y: 0.8,
                z: 0.8,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: "U: Reimport Textures".to_string(),
            position: ffi::Vec2 { x: 290.0, y: 450.0 },
            font_size: 18.0,
            color: ffi::Vec4 {
                x: 0.7,
                y: 0.95,
                z: 0.95,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: "Settings (auto-saved)".to_string(),
            position: ffi::Vec2 { x: 285.0, y: 290.0 },
            font_size: 22.0,
            color: ffi::Vec4 {
                x: 0.8,
                y: 0.9,
                z: 1.0,
                w: 1.0,
            },
        });
        self.push_settings_text(230.0);

        if let Some(action_id) = ui::ui_system(self) {
            self.dispatch_sample_action_id(&action_id);
        }
    }

    pub(crate) fn clear_menu_buttons(&mut self) {
        self.world
            .clear_entities_of_component(ComponentType::Button);
    }

    fn clear_runtime_world(&mut self) {
        for component_type in [
            ComponentType::Transform,
            ComponentType::Velocity,
            ComponentType::Material,
            ComponentType::RenderMesh,
            ComponentType::Player,
            ComponentType::Obstacle,
            ComponentType::Button,
            ComponentType::Physics,
            ComponentType::Sprite,
            ComponentType::SpriteAnimation,
        ] {
            self.world.clear_entities_of_component(component_type);
        }
        self.renderables.clear();
        self.text_commands.clear();
        self.asset_commands.clear();
    }

    pub(crate) fn setup_title_screen(&mut self) {
        self.clear_runtime_world();
        self.sample_loop = SampleGameLoop::from_state_and_mode(
            SampleGameState::Title,
            SampleGameRunMode::BoxSurvival2d,
        );
        self.current_state = GameState::Title;
        self.esc_was_pressed = false;
        self.apply_runtime_bgm_for_state();

        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 450.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Start Game".to_string(),
            action_id: SampleGameButtonAction::StartGame.action_id().to_string(),
        },));
        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 380.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Start 3D Arena".to_string(),
            action_id: SampleGameButtonAction::Start3dArena.action_id().to_string(),
        },));
        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 310.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Exit".to_string(),
            action_id: SampleGameButtonAction::ExitGame.action_id().to_string(),
        },));
        self.spawn_settings_buttons(230.0);
    }

    pub(crate) fn start_new_run(&mut self) {
        self.clear_runtime_world();
        self.sample_loop = SampleGameLoop::from_state_and_mode(
            SampleGameState::InGame,
            SampleGameRunMode::BoxSurvival2d,
        );
        self.current_state = GameState::InGame;
        self.run_mode = RunMode::BoxSurvival2d;
        self.esc_was_pressed = false;
        self.apply_runtime_bgm_for_state();
        self.hp = 3;
        self.survival_time_sec = 0.0;
        self.avoid_count = 0;
        self.score = 0;
        self.difficulty_level = 1;
        self.result_is_clear = false;
        self.obstacle_spawn_accumulator_sec = 0.0;
        self.save_data.progress.total_play_count =
            self.save_data.progress.total_play_count.saturating_add(1);
        self.total_play_count = self.save_data.progress.total_play_count;

        self.player_texture_handle = self.asset_server.load_texture("assets/player.png");
        self.obstacle_texture_handle = self.asset_server.load_texture("assets/test.png");
        let level_path = resolve_runtime_path(LEVEL_FILE_REL_PATH);
        let level_data = match level::load_level_from_path(&level_path) {
            Ok(level) => Some(level),
            Err(err) => {
                eprintln!(
                    "[level] Failed to load {}: {err}. Falling back to procedural obstacle layout.",
                    level_path.display()
                );
                None
            }
        };
        let player_spawn = level_data
            .as_ref()
            .map(|level| level.player_spawn)
            .unwrap_or(level::SpawnPoint {
                x: SCREEN_WIDTH * 0.5,
                y: 80.0,
            });

        self.world.spawn((
            ffi::Transform {
                position: ffi::Vec3 {
                    x: player_spawn.x,
                    y: player_spawn.y,
                    z: 0.0,
                },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: PLAYER_SIZE,
                    y: PLAYER_SIZE,
                    z: 1.0,
                },
            },
            ffi::Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Material {
                texture_handle: self.player_texture_handle,
            },
            Sprite,
            SpriteAnimation {
                frame_handles: vec![self.player_texture_handle, self.obstacle_texture_handle],
                current_frame: 0,
                elapsed_sec: 0.0,
                frame_duration_sec: 0.18,
                idle_speed_scale: 0.35,
                active_speed_scale: 1.0,
            },
            Player,
        ));

        if let Some(level_data) = level_data {
            for spawn in level_data.obstacle_spawns {
                self.spawn_obstacle_at(spawn.x, spawn.y);
            }
        } else {
            for _ in 0..8 {
                self.spawn_obstacle();
            }
        }

        runtime_bridge::play_sound("assets/test_sound.wav");
    }

    fn start_new_3d_run(&mut self) {
        self.clear_runtime_world();
        self.sample_loop = SampleGameLoop::from_state_and_mode(
            SampleGameState::InGame,
            SampleGameRunMode::Arena3d,
        );
        self.current_state = GameState::InGame;
        self.run_mode = RunMode::Arena3d;
        self.esc_was_pressed = false;
        self.apply_runtime_bgm_for_state();
        self.hp = 3;
        self.survival_time_sec = 0.0;
        self.avoid_count = 0;
        self.score = 0;
        self.difficulty_level = 1;
        self.result_is_clear = false;
        self.obstacle_spawn_accumulator_sec = 0.0;
        self.save_data.progress.total_play_count =
            self.save_data.progress.total_play_count.saturating_add(1);
        self.total_play_count = self.save_data.progress.total_play_count;

        self.player_texture_handle = self.asset_server.load_texture("assets/player.png");
        self.obstacle_texture_handle = self.asset_server.load_texture("assets/test.png");

        self.spawn_3d_arena_box(
            0.0,
            ARENA_FLOOR_Y,
            0.0,
            ARENA_HALF_EXTENT * 2.0,
            6.0,
            ARENA_HALF_EXTENT * 2.0,
            self.obstacle_texture_handle,
        );
        self.spawn_3d_arena_box(
            -ARENA_HALF_EXTENT,
            0.0,
            0.0,
            ARENA_WALL_THICKNESS,
            ARENA_WALL_HEIGHT,
            ARENA_HALF_EXTENT * 2.0,
            self.obstacle_texture_handle,
        );
        self.spawn_3d_arena_box(
            ARENA_HALF_EXTENT,
            0.0,
            0.0,
            ARENA_WALL_THICKNESS,
            ARENA_WALL_HEIGHT,
            ARENA_HALF_EXTENT * 2.0,
            self.obstacle_texture_handle,
        );
        self.spawn_3d_arena_box(
            0.0,
            0.0,
            -ARENA_HALF_EXTENT,
            ARENA_HALF_EXTENT * 2.0,
            ARENA_WALL_HEIGHT,
            ARENA_WALL_THICKNESS,
            self.obstacle_texture_handle,
        );
        self.spawn_3d_arena_box(
            0.0,
            0.0,
            ARENA_HALF_EXTENT,
            ARENA_HALF_EXTENT * 2.0,
            ARENA_WALL_HEIGHT,
            ARENA_WALL_THICKNESS,
            self.obstacle_texture_handle,
        );

        self.world.spawn((
            ffi::Transform {
                position: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 48.0,
                },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: ARENA_PLAYER_SIZE,
                    y: ARENA_PLAYER_SIZE,
                    z: ARENA_PLAYER_SIZE,
                },
            },
            ffi::Velocity {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Material {
                texture_handle: self.player_texture_handle,
            },
            RenderMesh {
                mesh_id: MESH_ID_ARENA_CUBE_3D,
                material_id: MATERIAL_ID_LIT_TEXTURED_3D,
                is_3d: true,
            },
            Player,
        ));

        self.spawn_3d_obstacle_at(
            ARENA_OBSTACLE_LANE_X,
            ARENA_OBSTACLE_RESPAWN_Y,
            ARENA_OBSTACLE_LANE_Z,
        );

        runtime_bridge::play_sound("assets/test_sound.wav");
    }

    fn spawn_3d_arena_box(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        scale_x: f32,
        scale_y: f32,
        scale_z: f32,
        texture_handle: u32,
    ) {
        self.world.spawn((
            ffi::Transform {
                position: ffi::Vec3 { x, y, z },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: scale_x,
                    y: scale_y,
                    z: scale_z,
                },
            },
            Material { texture_handle },
            RenderMesh {
                mesh_id: MESH_ID_ARENA_CUBE_3D,
                material_id: MATERIAL_ID_LIT_TEXTURED_3D,
                is_3d: true,
            },
        ));
    }

    fn spawn_3d_obstacle_at(&mut self, x: f32, y: f32, z: f32) {
        self.world.spawn((
            ffi::Transform {
                position: ffi::Vec3 { x, y, z },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: ARENA_OBSTACLE_SIZE,
                    y: ARENA_OBSTACLE_SIZE,
                    z: ARENA_OBSTACLE_SIZE,
                },
            },
            ffi::Velocity {
                x: 0.0,
                y: -ARENA_OBSTACLE_FALL_SPEED,
                z: 0.0,
            },
            Material {
                texture_handle: self.obstacle_texture_handle,
            },
            RenderMesh {
                mesh_id: MESH_ID_ARENA_CUBE_3D,
                material_id: MATERIAL_ID_LIT_TEXTURED_3D,
                is_3d: true,
            },
            Obstacle,
        ));
    }

    fn setup_pause_menu(&mut self) {
        self.clear_menu_buttons();
        self.sample_loop = SampleGameLoop::from_state_and_mode(
            SampleGameState::Pause,
            sample_run_mode_for_run_mode(self.run_mode),
        );
        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 340.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Resume".to_string(),
            action_id: SampleGameButtonAction::ResumeGame.action_id().to_string(),
        },));
        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 270.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Back To Title".to_string(),
            action_id: SampleGameButtonAction::BackToTitle.action_id().to_string(),
        },));
        self.spawn_settings_buttons(170.0);
    }

    fn setup_result_menu(&mut self) {
        self.clear_menu_buttons();
        self.sample_loop = SampleGameLoop::from_state_and_mode(
            SampleGameState::Result,
            sample_run_mode_for_run_mode(self.run_mode),
        );
        self.apply_runtime_bgm_for_state();
        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 250.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Retry".to_string(),
            action_id: SampleGameButtonAction::RetryGame.action_id().to_string(),
        },));
        self.world.spawn((Button {
            rect: ui::Rect {
                x: 300.0,
                y: 180.0,
                width: 200.0,
                height: 50.0,
            },
            text: "Back To Title".to_string(),
            action_id: SampleGameButtonAction::BackToTitle.action_id().to_string(),
        },));
    }

    fn spawn_obstacle(&mut self) {
        if self.obstacle_texture_handle == 0 {
            self.obstacle_texture_handle = self.asset_server.load_texture("assets/test.png");
        }

        let mut rng = rand::thread_rng();
        self.spawn_obstacle_at(
            rng.gen_range(20.0..(SCREEN_WIDTH - 20.0)),
            SCREEN_HEIGHT + rng.gen_range(20.0..120.0),
        );
    }

    fn spawn_obstacle_at(&mut self, x: f32, y: f32) {
        if self.obstacle_texture_handle == 0 {
            self.obstacle_texture_handle = self.asset_server.load_texture("assets/test.png");
        }

        self.world.spawn((
            ffi::Transform {
                position: ffi::Vec3 { x, y, z: 0.0 },
                rotation: ffi::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                scale: ffi::Vec3 {
                    x: OBSTACLE_SIZE,
                    y: OBSTACLE_SIZE,
                    z: 1.0,
                },
            },
            ffi::Velocity {
                x: 0.0,
                y: -BASE_OBSTACLE_SPEED,
                z: 0.0,
            },
            Material {
                texture_handle: self.obstacle_texture_handle,
            },
            Sprite,
            Obstacle,
        ));
    }

    fn count_obstacles(&self) -> usize {
        self.world
            .archetypes
            .iter()
            .filter(|arch| arch.types.contains(&ComponentType::Obstacle))
            .map(|arch| arch.entity_count)
            .sum()
    }

    fn current_spawn_interval_sec(&self) -> f32 {
        let level_reduction = (self.difficulty_level.saturating_sub(1) as f32) * 0.08;
        (BASE_SPAWN_INTERVAL_SEC - level_reduction).max(MIN_SPAWN_INTERVAL_SEC)
    }

    fn update_player_and_get_bounds(&mut self) -> Option<(f32, f32, f32, f32)> {
        let mut player_bounds = None;
        let (move_up, move_down, move_left, move_right) =
            match self.save_data.settings.movement_preset {
                MovementPreset::ArrowKeys => (
                    self.input_state.up,
                    self.input_state.down,
                    self.input_state.left,
                    self.input_state.right,
                ),
                MovementPreset::Wasd => (
                    self.input_state.w_key,
                    self.input_state.s_key,
                    self.input_state.a_key,
                    self.input_state.d_key,
                ),
            };

        for archetype in &mut self.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Player)
                && archetype.types.contains(&ComponentType::Transform)
                && archetype.types.contains(&ComponentType::Velocity))
            {
                continue;
            }

            let mut transform_storage =
                archetype.storage.remove(&ComponentType::Transform).unwrap();
            let mut velocity_storage = archetype.storage.remove(&ComponentType::Velocity).unwrap();
            let transforms = transform_storage
                .downcast_mut::<Vec<ffi::Transform>>()
                .unwrap();
            let velocities = velocity_storage
                .downcast_mut::<Vec<ffi::Velocity>>()
                .unwrap();

            for i in 0..archetype.entity_count {
                let mut move_x: f32 = 0.0;
                let mut move_y: f32 = 0.0;
                if move_left {
                    move_x -= 1.0;
                }
                if move_right {
                    move_x += 1.0;
                }
                if move_up {
                    move_y += 1.0;
                }
                if move_down {
                    move_y -= 1.0;
                }

                let length = (move_x * move_x + move_y * move_y).sqrt();
                if length > 0.0 {
                    move_x /= length;
                    move_y /= length;
                }

                velocities[i].x = move_x * PLAYER_SPEED;
                velocities[i].y = move_y * PLAYER_SPEED;
                transforms[i].position.x += velocities[i].x * FIXED_DT_SEC;
                transforms[i].position.y += velocities[i].y * FIXED_DT_SEC;

                let half_w = transforms[i].scale.x * 0.5;
                let half_h = transforms[i].scale.y * 0.5;
                transforms[i].position.x = transforms[i]
                    .position
                    .x
                    .clamp(half_w, SCREEN_WIDTH - half_w);
                transforms[i].position.y = transforms[i]
                    .position
                    .y
                    .clamp(half_h, SCREEN_HEIGHT - half_h);

                player_bounds = Some((
                    transforms[i].position.x - half_w,
                    transforms[i].position.y - half_h,
                    transforms[i].position.x + half_w,
                    transforms[i].position.y + half_h,
                ));
            }

            archetype
                .storage
                .insert(ComponentType::Transform, transform_storage);
            archetype
                .storage
                .insert(ComponentType::Velocity, velocity_storage);
        }

        player_bounds
    }

    fn update_player_3d(&mut self) {
        let (move_up, move_down, move_left, move_right) =
            match self.save_data.settings.movement_preset {
                MovementPreset::ArrowKeys => (
                    self.input_state.up,
                    self.input_state.down,
                    self.input_state.left,
                    self.input_state.right,
                ),
                MovementPreset::Wasd => (
                    self.input_state.w_key,
                    self.input_state.s_key,
                    self.input_state.a_key,
                    self.input_state.d_key,
                ),
            };

        for archetype in &mut self.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Player)
                && archetype.types.contains(&ComponentType::Transform)
                && archetype.types.contains(&ComponentType::Velocity)
                && archetype.types.contains(&ComponentType::RenderMesh))
            {
                continue;
            }

            let render_meshes = archetype
                .storage
                .get(&ComponentType::RenderMesh)
                .and_then(|storage| storage.downcast_ref::<Vec<RenderMesh>>())
                .expect("render mesh storage must be Vec<RenderMesh>");
            let is_3d_player = render_meshes.first().map(|mesh| mesh.is_3d).unwrap_or(false);
            if !is_3d_player {
                continue;
            }

            let mut transform_storage =
                archetype.storage.remove(&ComponentType::Transform).unwrap();
            let mut velocity_storage = archetype.storage.remove(&ComponentType::Velocity).unwrap();
            let transforms = transform_storage
                .downcast_mut::<Vec<ffi::Transform>>()
                .unwrap();
            let velocities = velocity_storage
                .downcast_mut::<Vec<ffi::Velocity>>()
                .unwrap();

            for i in 0..archetype.entity_count {
                let mut move_x: f32 = 0.0;
                let mut move_z: f32 = 0.0;
                if move_left {
                    move_x -= 1.0;
                }
                if move_right {
                    move_x += 1.0;
                }
                if move_up {
                    move_z -= 1.0;
                }
                if move_down {
                    move_z += 1.0;
                }

                let length = (move_x * move_x + move_z * move_z).sqrt();
                if length > 0.0 {
                    move_x /= length;
                    move_z /= length;
                }

                velocities[i].x = move_x * ARENA_PLAYER_MOVE_SPEED;
                velocities[i].y = 0.0;
                velocities[i].z = move_z * ARENA_PLAYER_MOVE_SPEED;
                transforms[i].position.x += velocities[i].x * FIXED_DT_SEC;
                transforms[i].position.z += velocities[i].z * FIXED_DT_SEC;
                transforms[i].position.x = transforms[i]
                    .position
                    .x
                    .clamp(-ARENA_HALF_EXTENT + 14.0, ARENA_HALF_EXTENT - 14.0);
                transforms[i].position.z = transforms[i]
                    .position
                    .z
                    .clamp(-ARENA_HALF_EXTENT + 14.0, ARENA_HALF_EXTENT - 14.0);
            }

            archetype
                .storage
                .insert(ComponentType::Transform, transform_storage);
            archetype
                .storage
                .insert(ComponentType::Velocity, velocity_storage);
        }
    }

    fn player_bounds_3d(&self) -> Option<(ffi::Vec3, ffi::Vec3)> {
        for archetype in &self.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Player)
                && archetype.types.contains(&ComponentType::Transform)
                && archetype.types.contains(&ComponentType::RenderMesh))
            {
                continue;
            }

            let render_meshes = archetype
                .storage
                .get(&ComponentType::RenderMesh)
                .and_then(|storage| storage.downcast_ref::<Vec<RenderMesh>>())
                .expect("render mesh storage must be Vec<RenderMesh>");
            let is_3d_player = render_meshes.first().map(|mesh| mesh.is_3d).unwrap_or(false);
            if !is_3d_player {
                continue;
            }

            let transforms = archetype
                .storage
                .get(&ComponentType::Transform)
                .and_then(|storage| storage.downcast_ref::<Vec<ffi::Transform>>())
                .expect("transform storage must be Vec<Transform>");
            let Some(transform) = transforms.first() else {
                continue;
            };

            let half = ffi::Vec3 {
                x: transform.scale.x * 0.5,
                y: transform.scale.y * 0.5,
                z: transform.scale.z * 0.5,
            };
            return Some((
                ffi::Vec3 {
                    x: transform.position.x - half.x,
                    y: transform.position.y - half.y,
                    z: transform.position.z - half.z,
                },
                ffi::Vec3 {
                    x: transform.position.x + half.x,
                    y: transform.position.y + half.y,
                    z: transform.position.z + half.z,
                },
            ));
        }

        None
    }

    fn update_3d_obstacles_and_collisions(&mut self, player_bounds: Option<(ffi::Vec3, ffi::Vec3)>) {
        let obstacle_speed =
            ARENA_OBSTACLE_FALL_SPEED + (self.difficulty_level.saturating_sub(1) as f32) * 12.0;
        let mut hit_detected = false;

        for archetype in &mut self.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Obstacle)
                && archetype.types.contains(&ComponentType::Transform)
                && archetype.types.contains(&ComponentType::Velocity)
                && archetype.types.contains(&ComponentType::RenderMesh))
            {
                continue;
            }

            let render_meshes = archetype
                .storage
                .get(&ComponentType::RenderMesh)
                .and_then(|storage| storage.downcast_ref::<Vec<RenderMesh>>())
                .expect("render mesh storage must be Vec<RenderMesh>");
            let has_3d_obstacle = render_meshes.first().map(|mesh| mesh.is_3d).unwrap_or(false);
            if !has_3d_obstacle {
                continue;
            }

            let mut transform_storage =
                archetype.storage.remove(&ComponentType::Transform).unwrap();
            let mut velocity_storage = archetype.storage.remove(&ComponentType::Velocity).unwrap();
            let transforms = transform_storage
                .downcast_mut::<Vec<ffi::Transform>>()
                .unwrap();
            let velocities = velocity_storage
                .downcast_mut::<Vec<ffi::Velocity>>()
                .unwrap();

            for i in 0..archetype.entity_count {
                velocities[i].x = 0.0;
                velocities[i].y = -obstacle_speed;
                velocities[i].z = 0.0;
                transforms[i].position.y += velocities[i].y * FIXED_DT_SEC;

                if let Some((player_min, player_max)) = player_bounds {
                    let half = ffi::Vec3 {
                        x: transforms[i].scale.x * 0.5,
                        y: transforms[i].scale.y * 0.5,
                        z: transforms[i].scale.z * 0.5,
                    };
                    let obstacle_min = ffi::Vec3 {
                        x: transforms[i].position.x - half.x,
                        y: transforms[i].position.y - half.y,
                        z: transforms[i].position.z - half.z,
                    };
                    let obstacle_max = ffi::Vec3 {
                        x: transforms[i].position.x + half.x,
                        y: transforms[i].position.y + half.y,
                        z: transforms[i].position.z + half.z,
                    };
                    let overlaps = player_min.x < obstacle_max.x
                        && player_max.x > obstacle_min.x
                        && player_min.y < obstacle_max.y
                        && player_max.y > obstacle_min.y
                        && player_min.z < obstacle_max.z
                        && player_max.z > obstacle_min.z;
                    if overlaps {
                        if self.hp > 0 {
                            self.hp -= 1;
                        }
                        transforms[i].position.y = ARENA_OBSTACLE_RESPAWN_Y;
                        hit_detected = true;
                        continue;
                    }
                }

                if transforms[i].position.y < ARENA_FLOOR_Y - transforms[i].scale.y {
                    transforms[i].position.y = ARENA_OBSTACLE_RESPAWN_Y;
                    self.avoid_count = self.avoid_count.saturating_add(1);
                }
            }

            archetype
                .storage
                .insert(ComponentType::Transform, transform_storage);
            archetype
                .storage
                .insert(ComponentType::Velocity, velocity_storage);
        }

        if hit_detected {
            runtime_bridge::play_sound("assets/test_sound.wav");
        }
    }

    fn update_obstacles_and_collisions(&mut self, player_bounds: Option<(f32, f32, f32, f32)>) {
        let obstacle_speed =
            BASE_OBSTACLE_SPEED + (self.difficulty_level.saturating_sub(1) as f32) * 30.0;
        let mut rng = rand::thread_rng();
        let mut hit_detected = false;

        for archetype in &mut self.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Obstacle)
                && archetype.types.contains(&ComponentType::Transform)
                && archetype.types.contains(&ComponentType::Velocity))
            {
                continue;
            }

            let mut transform_storage =
                archetype.storage.remove(&ComponentType::Transform).unwrap();
            let mut velocity_storage = archetype.storage.remove(&ComponentType::Velocity).unwrap();
            let transforms = transform_storage
                .downcast_mut::<Vec<ffi::Transform>>()
                .unwrap();
            let velocities = velocity_storage
                .downcast_mut::<Vec<ffi::Velocity>>()
                .unwrap();

            for i in 0..archetype.entity_count {
                velocities[i].y = -obstacle_speed;
                transforms[i].position.y += velocities[i].y * FIXED_DT_SEC;

                if transforms[i].position.y < -OBSTACLE_SIZE {
                    transforms[i].position.y = SCREEN_HEIGHT + rng.gen_range(20.0..120.0);
                    transforms[i].position.x = rng.gen_range(20.0..(SCREEN_WIDTH - 20.0));
                    self.avoid_count = self.avoid_count.saturating_add(1);
                }

                if let Some((pl, pb, pr, pt)) = player_bounds {
                    let half_w = transforms[i].scale.x * 0.5;
                    let half_h = transforms[i].scale.y * 0.5;
                    let ol = transforms[i].position.x - half_w;
                    let ob = transforms[i].position.y - half_h;
                    let oright = transforms[i].position.x + half_w;
                    let ot = transforms[i].position.y + half_h;
                    let overlaps = pl < oright && pr > ol && pb < ot && pt > ob;
                    if overlaps {
                        self.hp -= 1;
                        transforms[i].position.y = SCREEN_HEIGHT + rng.gen_range(20.0..120.0);
                        transforms[i].position.x = rng.gen_range(20.0..(SCREEN_WIDTH - 20.0));
                        hit_detected = true;
                    }
                }
            }

            archetype
                .storage
                .insert(ComponentType::Transform, transform_storage);
            archetype
                .storage
                .insert(ComponentType::Velocity, velocity_storage);
        }

        if hit_detected {
            runtime_bridge::play_sound("assets/test_sound.wav");
        }
    }

    fn update_sprite_animations(&mut self) {
        for archetype in &mut self.world.archetypes {
            if !(archetype.types.contains(&ComponentType::SpriteAnimation)
                && archetype.types.contains(&ComponentType::Material))
            {
                continue;
            }

            let mut animation_storage = archetype
                .storage
                .remove(&ComponentType::SpriteAnimation)
                .expect("sprite animation storage must exist");
            let mut material_storage = archetype
                .storage
                .remove(&ComponentType::Material)
                .expect("material storage must exist");

            let animations = animation_storage
                .downcast_mut::<Vec<SpriteAnimation>>()
                .expect("sprite animation storage must be Vec<SpriteAnimation>");
            let materials = material_storage
                .downcast_mut::<Vec<Material>>()
                .expect("material storage must be Vec<Material>");

            let velocities = archetype
                .storage
                .get(&ComponentType::Velocity)
                .and_then(|storage| storage.downcast_ref::<Vec<ffi::Velocity>>());

            for index in 0..archetype.entity_count {
                let animation = &mut animations[index];
                if animation.frame_handles.len() < 2 {
                    continue;
                }

                let speed_scale = velocities
                    .and_then(|velocities| velocities.get(index))
                    .map(|velocity| {
                        if velocity.x.abs() > f32::EPSILON || velocity.y.abs() > f32::EPSILON {
                            animation.active_speed_scale
                        } else {
                            animation.idle_speed_scale
                        }
                    })
                    .unwrap_or(animation.active_speed_scale);

                animation.elapsed_sec += FIXED_DT_SEC * speed_scale;
                while animation.elapsed_sec >= animation.frame_duration_sec {
                    animation.elapsed_sec -= animation.frame_duration_sec;
                    animation.current_frame =
                        (animation.current_frame + 1) % animation.frame_handles.len();
                    materials[index].texture_handle =
                        animation.frame_handles[animation.current_frame];
                }
            }

            archetype
                .storage
                .insert(ComponentType::SpriteAnimation, animation_storage);
            archetype
                .storage
                .insert(ComponentType::Material, material_storage);
        }
    }

    fn push_hud_text(&mut self) {
        self.text_commands.push(ffi::TextCommand {
            text: format!(
                "HP:{}  Time:{:.1}s  Score:{}  Lv:{}",
                self.hp, self.survival_time_sec, self.score, self.difficulty_level
            ),
            position: ffi::Vec2 { x: 16.0, y: 570.0 },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
                w: 1.0,
            },
        });
    }

    fn update_pause(&mut self) {
        self.text_commands.clear();
        self.text_commands.push(ffi::TextCommand {
            text: "PAUSED".to_string(),
            position: ffi::Vec2 { x: 340.0, y: 420.0 },
            font_size: 36.0,
            color: ffi::Vec4 {
                x: 1.0,
                y: 0.95,
                z: 0.2,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: "U: Reimport Textures".to_string(),
            position: ffi::Vec2 { x: 290.0, y: 390.0 },
            font_size: 18.0,
            color: ffi::Vec4 {
                x: 0.7,
                y: 0.95,
                z: 0.95,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: "Settings (auto-saved)".to_string(),
            position: ffi::Vec2 { x: 285.0, y: 230.0 },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.8,
                y: 0.9,
                z: 1.0,
                w: 1.0,
            },
        });
        self.push_settings_text(170.0);

        let esc_just_pressed = self.input_state.esc_key && !self.esc_was_pressed;
        self.esc_was_pressed = self.input_state.esc_key;
        if esc_just_pressed {
            self.dispatch_sample_event(SampleGameEvent::EscapePressed);
            return;
        }

        if let Some(action_id) = ui::ui_system(self) {
            self.dispatch_sample_action_id(&action_id);
        }
    }

    fn update_result(&mut self) {
        self.text_commands.clear();
        self.renderables.clear();

        let headline = if self.result_is_clear {
            "CLEAR"
        } else {
            "GAME OVER"
        };
        self.text_commands.push(ffi::TextCommand {
            text: headline.to_string(),
            position: ffi::Vec2 { x: 300.0, y: 440.0 },
            font_size: 42.0,
            color: ffi::Vec4 {
                x: 1.0,
                y: 0.9,
                z: 0.2,
                w: 1.0,
            },
        });
        if self.run_mode == RunMode::Arena3d {
            self.text_commands.push(ffi::TextCommand {
                text: "3D Arena Result".to_string(),
                position: ffi::Vec2 { x: 300.0, y: 410.0 },
                font_size: 18.0,
                color: ffi::Vec4 {
                    x: 0.75,
                    y: 0.9,
                    z: 1.0,
                    w: 1.0,
                },
            });
        }
        self.text_commands.push(ffi::TextCommand {
            text: format!("Score: {}", self.score),
            position: ffi::Vec2 { x: 300.0, y: 390.0 },
            font_size: 26.0,
            color: ffi::Vec4 {
                x: 0.95,
                y: 0.95,
                z: 0.95,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!("Survival: {:.1} sec", self.survival_time_sec),
            position: ffi::Vec2 { x: 300.0, y: 360.0 },
            font_size: 22.0,
            color: ffi::Vec4 {
                x: 0.85,
                y: 0.85,
                z: 0.85,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!("High Score: {}", self.save_data.progress.best_score),
            position: ffi::Vec2 { x: 300.0, y: 330.0 },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.8,
                y: 0.95,
                z: 0.8,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!(
                "Best Survival: {} sec",
                self.save_data.progress.best_survival_sec
            ),
            position: ffi::Vec2 { x: 300.0, y: 305.0 },
            font_size: 20.0,
            color: ffi::Vec4 {
                x: 0.8,
                y: 0.85,
                z: 0.95,
                w: 1.0,
            },
        });
        self.text_commands.push(ffi::TextCommand {
            text: format!(
                "Play:{}  Clear:{}",
                self.save_data.progress.total_play_count, self.save_data.progress.total_clear_count
            ),
            position: ffi::Vec2 { x: 300.0, y: 280.0 },
            font_size: 18.0,
            color: ffi::Vec4 {
                x: 0.75,
                y: 0.75,
                z: 0.75,
                w: 1.0,
            },
        });

        if let Some(action_id) = ui::ui_system(self) {
            self.dispatch_sample_action_id(&action_id);
        }
    }

    fn setup_sprite_stress_test(&mut self) {
        self.world
            .clear_entities_of_component(ComponentType::Button);
        self.world
            .clear_entities_of_component(ComponentType::Physics);

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

    fn poll_physics_events(&mut self) {
        self.collision_events.clear();
        let events = runtime_bridge::get_collision_events();
        self.collision_events.extend_from_slice(events);
    }

    fn setup_physics_stress_test(&mut self) {
        self.world
            .clear_entities_of_component(ComponentType::Button);
        self.world
            .clear_entities_of_component(ComponentType::Sprite);

        const PPM: f32 = 50.0; // Pixels Per Meter
        const SCREEN_WIDTH: f32 = 800.0;
        const SCREEN_HEIGHT: f32 = 600.0;
        const WALL_THICKNESS: f32 = 10.0;

        let ground_texture = self.asset_server.load_texture("assets/test.png");
        let box_texture = self.asset_server.load_texture("assets/player.png");

        // Create container walls
        let walls = [
            // Bottom
            (
                SCREEN_WIDTH / 2.0,
                WALL_THICKNESS / 2.0,
                SCREEN_WIDTH,
                WALL_THICKNESS,
            ),
            // Top
            (
                SCREEN_WIDTH / 2.0,
                SCREEN_HEIGHT - WALL_THICKNESS / 2.0,
                SCREEN_WIDTH,
                WALL_THICKNESS,
            ),
            // Left
            (
                WALL_THICKNESS / 2.0,
                SCREEN_HEIGHT / 2.0,
                WALL_THICKNESS,
                SCREEN_HEIGHT,
            ),
            // Right
            (
                SCREEN_WIDTH - WALL_THICKNESS / 2.0,
                SCREEN_HEIGHT / 2.0,
                WALL_THICKNESS,
                SCREEN_HEIGHT,
            ),
        ];

        for (x, y, w, h) in walls {
            let body_id =
                runtime_bridge::create_static_box_body(x / PPM, y / PPM, w / PPM, h / PPM);
            self.world.spawn((
                ffi::Transform {
                    position: ffi::Vec3 { x, y, z: 0.0 },
                    rotation: ffi::Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: ffi::Vec3 { x: w, y: h, z: 1.0 },
                },
                PhysicsBody { id: body_id },
                Material {
                    texture_handle: ground_texture,
                },
            ));
        }

        // Create dynamic falling boxes
        let mut rng = rand::thread_rng();
        let box_size = 10.0;
        for _ in 0..500 {
            let x = rng
                .gen_range((WALL_THICKNESS + box_size)..(SCREEN_WIDTH - WALL_THICKNESS - box_size));
            let y = rng.gen_range(
                (WALL_THICKNESS + box_size)..(SCREEN_HEIGHT - WALL_THICKNESS - box_size),
            );

            let body_id = runtime_bridge::create_dynamic_box_body(
                x / PPM,
                y / PPM,
                box_size / PPM,
                box_size / PPM,
            );
            self.world.spawn((
                ffi::Transform {
                    position: ffi::Vec3 { x, y, z: 0.0 },
                    rotation: ffi::Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: ffi::Vec3 {
                        x: box_size,
                        y: box_size,
                        z: 1.0,
                    },
                },
                PhysicsBody { id: body_id },
                Material {
                    texture_handle: box_texture,
                },
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

    fn setup_ui_stress_test(&mut self) {
        self.world
            .clear_entities_of_component(ComponentType::Button);
        self.world
            .clear_entities_of_component(ComponentType::Sprite);
        self.world
            .clear_entities_of_component(ComponentType::Physics);
    }

    fn update_ui_stress_test(&mut self) {
        self.text_commands.clear();

        let items_per_row = 30;
        let items_per_col = 40;
        let font_size = 12.0;
        let mut count = 0;

        for i in 0..items_per_col {
            for j in 0..items_per_row {
                count += 1;
                self.text_commands.push(ffi::TextCommand {
                    text: format!("T{}", count),
                    position: ffi::Vec2 {
                        x: 5.0 + (j as f32 * (800.0 / items_per_row as f32)),
                        y: 15.0 + (i as f32 * (600.0 / items_per_col as f32)),
                    },
                    font_size,
                    color: ffi::Vec4 {
                        x: 0.8,
                        y: 0.8,
                        z: 0.1,
                        w: 1.0,
                    },
                });
            }
        }
    }

    fn setup_in_game(&mut self) {
        self.start_new_run();
    }

    fn update_in_game_3d(&mut self) {
        self.update_player_3d();
        let player_bounds = self.player_bounds_3d();
        self.update_3d_obstacles_and_collisions(player_bounds);
        self.survival_time_sec += FIXED_DT_SEC;
        self.difficulty_level = (self.survival_time_sec / 60.0).floor() as u32 + 1;
        self.score = (self.survival_time_sec as u32)
            .saturating_mul(10)
            .saturating_add(self.avoid_count.saturating_mul(100));

        if self.hp <= 0 {
            self.result_is_clear = false;
            self.apply_result_to_progress_and_persist();
            self.dispatch_sample_event(SampleGameEvent::RunFailed);
            return;
        }
        if self.survival_time_sec >= ARENA_CLEAR_TIME_SEC {
            self.result_is_clear = true;
            self.apply_result_to_progress_and_persist();
            self.dispatch_sample_event(SampleGameEvent::RunCleared);
            return;
        }

        self.process_asset_server();
        self.build_renderables();
        self.push_hud_text();
        self.text_commands.push(ffi::TextCommand {
            text: "3D Arena Prototype".to_string(),
            position: ffi::Vec2 { x: 16.0, y: 540.0 },
            font_size: 18.0,
            color: ffi::Vec4 {
                x: 0.75,
                y: 0.9,
                z: 1.0,
                w: 1.0,
            },
        });
    }

    fn update_in_game(&mut self) {
        self.text_commands.clear();

        let esc_just_pressed = self.input_state.esc_key && !self.esc_was_pressed;
        self.esc_was_pressed = self.input_state.esc_key;
        if esc_just_pressed {
            self.dispatch_sample_event(SampleGameEvent::EscapePressed);
            return;
        }

        if self.run_mode == RunMode::Arena3d {
            self.update_in_game_3d();
            return;
        }

        let player_bounds = self.update_player_and_get_bounds();

        self.obstacle_spawn_accumulator_sec += FIXED_DT_SEC;
        let spawn_interval = self.current_spawn_interval_sec();
        while self.obstacle_spawn_accumulator_sec >= spawn_interval {
            if self.count_obstacles() < MAX_OBSTACLES {
                self.spawn_obstacle();
            }
            self.obstacle_spawn_accumulator_sec -= spawn_interval;
        }

        self.update_obstacles_and_collisions(player_bounds);
        self.update_sprite_animations();
        self.survival_time_sec += FIXED_DT_SEC;
        self.difficulty_level = (self.survival_time_sec / 60.0).floor() as u32 + 1;
        self.score = (self.survival_time_sec as u32)
            .saturating_mul(10)
            .saturating_add(self.avoid_count.saturating_mul(100));

        if self.hp <= 0 {
            self.result_is_clear = false;
            self.apply_result_to_progress_and_persist();
            self.dispatch_sample_event(SampleGameEvent::RunFailed);
            return;
        }
        if self.survival_time_sec >= 1800.0 {
            self.result_is_clear = true;
            self.apply_result_to_progress_and_persist();
            self.dispatch_sample_event(SampleGameEvent::RunCleared);
            return;
        }

        self.process_asset_server();
        self.build_renderables();
        self.push_hud_text();
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
                    let new_pos_meters = runtime_bridge::get_body_position(body_id);

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
        self.setup_title_screen();
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
                let render_meshes = archetype
                    .storage
                    .get(&ComponentType::RenderMesh)
                    .and_then(|storage| storage.downcast_ref::<Vec<RenderMesh>>());

                for (index, (transform, material)) in
                    transforms.iter().zip(materials.iter()).enumerate()
                {
                    let texture_id = self
                        .texture_map
                        .get(&material.texture_handle)
                        .cloned()
                        .unwrap_or(0);
                    let render_mesh = render_meshes.and_then(|render_meshes| render_meshes.get(index));
                    self.renderables.push(ffi::RenderableObject {
                        transform: *transform,
                        mesh_id: render_mesh.map(|mesh| mesh.mesh_id).unwrap_or(MESH_ID_QUAD_2D),
                        material_id: render_mesh
                            .map(|mesh| mesh.material_id)
                            .unwrap_or(MATERIAL_ID_TEXTURED),
                        texture_id,
                        is_3d: render_mesh.map(|mesh| mesh.is_3d).unwrap_or(false),
                    });
                }
            }
        }
    }

    pub fn process_asset_server(&mut self) {
        self.asset_commands.clear();
        for (request_id, request) in self.asset_server.pending_requests.iter_mut() {
            if request.dispatched {
                continue;
            }

            self.asset_commands.push(ffi::AssetCommand {
                request_id: *request_id,
                type_: request.command_type.clone(),
                path: request.path.clone(),
            });
            request.dispatched = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use sample_game_runtime::SampleGameButtonAction;

    use super::{
        ffi, runtime_bridge, save, ui, Archetype, ComponentBundle, ComponentType, Game, GameState,
        Material, MovementPreset, RenderMesh, RunMode, SystemRegistry, ARENA_CLEAR_TIME_SEC,
        FIXED_DT_SEC, MATERIAL_ID_LIT_TEXTURED_3D, MESH_ID_ARENA_CUBE_3D, MESH_ID_QUAD_2D,
    };
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_save_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("miyabi-{test_name}-{nanos}.json"))
    }

    fn new_game_for_test(test_name: &str) -> Game {
        let mut game = Game::new();
        game.save_file_path = temp_save_path(test_name);
        let _ = std::fs::remove_file(&game.save_file_path);
        game
    }

    fn find_button_rect(game: &Game, action_id: &str) -> ui::Rect {
        for archetype in &game.world.archetypes {
            if !archetype.types.contains(&ComponentType::Button) {
                continue;
            }

            let buttons = archetype
                .storage
                .get(&ComponentType::Button)
                .expect("button storage must exist")
                .downcast_ref::<Vec<ui::Button>>()
                .expect("button storage must be Vec<Button>");

            if let Some(button) = buttons
                .iter()
                .find(|button| button.action_id.as_str() == action_id)
            {
                return button.rect;
            }
        }

        panic!("button not found for action_id: {action_id}");
    }

    fn click_button(game: &mut Game, action_id: &str) {
        let rect = find_button_rect(game, action_id);
        game.input_state.mouse_pos = ffi::Vec2 {
            x: rect.x + rect.width * 0.5,
            y: rect.y + rect.height * 0.5,
        };
        game.input_state.mouse_clicked = true;
        game.update();
        game.input_state.mouse_clicked = false;
        game.update();
    }

    fn button_action_ids(game: &Game) -> Vec<String> {
        let mut action_ids = Vec::new();
        for archetype in &game.world.archetypes {
            if !archetype.types.contains(&ComponentType::Button) {
                continue;
            }

            let buttons = archetype
                .storage
                .get(&ComponentType::Button)
                .expect("button storage must exist")
                .downcast_ref::<Vec<ui::Button>>()
                .expect("button storage must be Vec<Button>");

            for button in buttons {
                action_ids.push(button.action_id.clone());
            }
        }
        action_ids.sort();
        action_ids
    }

    fn text_command_texts(game: &Game) -> Vec<String> {
        game.text_commands
            .iter()
            .map(|command| command.text.clone())
            .collect()
    }

    fn keep_safe_corridor(game: &mut Game) {
        const CORRIDOR_LEFT: f32 = 340.0;
        const CORRIDOR_RIGHT: f32 = 460.0;
        const DANGER_Y: f32 = 220.0;
        const SAFE_LEFT_X: f32 = 60.0;
        const SAFE_RIGHT_X: f32 = 740.0;

        for archetype in &mut game.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Obstacle)
                && archetype.types.contains(&ComponentType::Transform))
            {
                continue;
            }

            let mut transform_storage = archetype
                .storage
                .remove(&ComponentType::Transform)
                .expect("transform storage must exist");
            let transforms = transform_storage
                .downcast_mut::<Vec<ffi::Transform>>()
                .expect("transform storage must be Vec<Transform>");

            for transform in transforms.iter_mut() {
                let half_w = transform.scale.x * 0.5;
                let overlaps_corridor = transform.position.x + half_w > CORRIDOR_LEFT
                    && transform.position.x - half_w < CORRIDOR_RIGHT;
                if overlaps_corridor && transform.position.y < DANGER_Y {
                    transform.position.x = if transform.position.x < 400.0 {
                        SAFE_LEFT_X
                    } else {
                        SAFE_RIGHT_X
                    };
                }
            }

            archetype
                .storage
                .insert(ComponentType::Transform, transform_storage);
        }
    }

    fn player_position(game: &Game) -> ffi::Vec2 {
        for archetype in &game.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Player)
                && archetype.types.contains(&ComponentType::Transform))
            {
                continue;
            }

            let transforms = archetype
                .storage
                .get(&ComponentType::Transform)
                .expect("transform storage must exist")
                .downcast_ref::<Vec<ffi::Transform>>()
                .expect("transform storage must be Vec<Transform>");

            if let Some(transform) = transforms.first() {
                return ffi::Vec2 {
                    x: transform.position.x,
                    y: transform.position.y,
                };
            }
        }

        panic!("player transform not found");
    }

    fn player_texture_handle(game: &Game) -> u32 {
        for archetype in &game.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Player)
                && archetype.types.contains(&ComponentType::Material))
            {
                continue;
            }

            let materials = archetype
                .storage
                .get(&ComponentType::Material)
                .expect("material storage must exist")
                .downcast_ref::<Vec<Material>>()
                .expect("material storage must be Vec<Material>");

            if let Some(material) = materials.first() {
                return material.texture_handle;
            }
        }

        panic!("player material not found");
    }

    fn obstacle_positions(game: &Game) -> Vec<(u32, u32)> {
        let mut positions = Vec::new();

        for archetype in &game.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Obstacle)
                && archetype.types.contains(&ComponentType::Transform))
            {
                continue;
            }

            let transforms = archetype
                .storage
                .get(&ComponentType::Transform)
                .expect("transform storage must exist")
                .downcast_ref::<Vec<ffi::Transform>>()
                .expect("transform storage must be Vec<Transform>");

            for transform in transforms {
                positions.push((
                    transform.position.x.round() as u32,
                    transform.position.y.round() as u32,
                ));
            }
        }

        positions.sort_unstable();
        positions
    }

    fn player_position_3d(game: &Game) -> ffi::Vec3 {
        for archetype in &game.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Player)
                && archetype.types.contains(&ComponentType::Transform))
            {
                continue;
            }

            let transforms = archetype
                .storage
                .get(&ComponentType::Transform)
                .expect("transform storage must exist")
                .downcast_ref::<Vec<ffi::Transform>>()
                .expect("transform storage must be Vec<Transform>");

            if let Some(transform) = transforms.first() {
                return transform.position;
            }
        }

        panic!("player transform not found");
    }

    fn obstacle_positions_3d(game: &Game) -> Vec<(i32, i32, i32)> {
        let mut positions = Vec::new();

        for archetype in &game.world.archetypes {
            if !(archetype.types.contains(&ComponentType::Obstacle)
                && archetype.types.contains(&ComponentType::Transform)
                && archetype.types.contains(&ComponentType::RenderMesh))
            {
                continue;
            }

            let render_meshes = archetype
                .storage
                .get(&ComponentType::RenderMesh)
                .and_then(|storage| storage.downcast_ref::<Vec<RenderMesh>>())
                .expect("render mesh storage must be Vec<RenderMesh>");
            if !render_meshes.first().map(|mesh| mesh.is_3d).unwrap_or(false) {
                continue;
            }

            let transforms = archetype
                .storage
                .get(&ComponentType::Transform)
                .expect("transform storage must exist")
                .downcast_ref::<Vec<ffi::Transform>>()
                .expect("transform storage must be Vec<Transform>");

            for transform in transforms {
                positions.push((
                    transform.position.x.round() as i32,
                    transform.position.y.round() as i32,
                    transform.position.z.round() as i32,
                ));
            }
        }

        positions.sort_unstable();
        positions
    }

    fn count_3d_obstacles(game: &Game) -> usize {
        obstacle_positions_3d(game).len()
    }

    fn count_3d_renderables(game: &Game) -> usize {
        game.renderables.iter().filter(|renderable| renderable.is_3d).count()
    }

    #[test]
    fn asset_integrity_registry_log_includes_tick() {
        let message = Game::asset_integrity_registry_inconsistency_log(30);
        assert_eq!(
            message,
            "[asset] integrity: asset_integrity_tick=30 registry inconsistency detected"
        );
    }

    #[test]
    fn asset_integrity_missing_registry_log_includes_tick_and_handle() {
        let message = Game::asset_integrity_missing_registry_log(60, 42);
        assert_eq!(
            message,
            "[asset] integrity: asset_integrity_tick=60 missing registry for texture_handle=42"
        );
    }

    #[test]
    fn asset_integrity_unresolved_reference_log_includes_tick_and_context() {
        let message =
            Game::asset_integrity_unresolved_reference_log(90, 42, 77, "assets/player.png", true);
        assert_eq!(
            message,
            "[asset] integrity: asset_integrity_tick=90 unresolved reference handle=42 asset_id=77 path=assets/player.png queued_reimport=true"
        );
    }

    #[test]
    fn ffi_null_pointer_error_is_standardized() {
        assert_eq!(
            crate::ffi_null_pointer_error("update_input_state", "input"),
            "[ffi][error] update_input_state: null pointer argument `input`"
        );
    }

    #[test]
    fn ffi_error_is_standardized() {
        assert_eq!(
            crate::ffi_error("deserialize_game", "JSON parse failed: invalid type"),
            "[ffi][error] deserialize_game: JSON parse failed: invalid type"
        );
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn component_pool_uninitialized_storage_reference_is_detected() {
        let mut types = HashSet::new();
        types.insert(ComponentType::Transform);
        let mut archetype = Archetype::new(types);

        // Component storage を初期化しないまま push すると未初期化参照になる。
        (ffi::Transform {
            position: ffi::Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: ffi::Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            scale: ffi::Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        },)
            .push_to_storage(&mut archetype);
    }

    #[test]
    fn system_registry_duplicate_registration_log_includes_cause() {
        let mut registry = SystemRegistry::default();
        registry
            .register("ui_system", "post_update", "logic/src/ui.rs")
            .expect("initial registration must succeed");

        let err = registry
            .register("ui_system", "update", "logic/src/lib.rs")
            .expect_err("duplicate registration must be rejected");

        assert_eq!(
            err,
            "[ecs][system_registry][duplicate] system=ui_system new_stage=update new_source=logic/src/lib.rs existing_stage=post_update existing_source=logic/src/ui.rs"
        );
    }

    #[test]
    fn title_pause_result_and_exit_flow_is_reachable() {
        let mut game = new_game_for_test("scene-flow");
        assert_eq!(game.current_state, GameState::Title);

        click_button(&mut game, SampleGameButtonAction::StartGame.action_id());
        assert_eq!(game.current_state, GameState::InGame);

        game.input_state.esc_key = true;
        game.update();
        assert_eq!(game.current_state, GameState::Pause);

        game.input_state.esc_key = false;
        game.update();
        click_button(&mut game, SampleGameButtonAction::ResumeGame.action_id());
        assert_eq!(game.current_state, GameState::InGame);

        game.hp = 0;
        game.update();
        assert_eq!(game.current_state, GameState::Result);
        assert!(!game.result_is_clear);

        click_button(&mut game, SampleGameButtonAction::RetryGame.action_id());
        assert_eq!(game.current_state, GameState::InGame);

        game.hp = 0;
        game.update();
        click_button(&mut game, SampleGameButtonAction::BackToTitle.action_id());
        assert_eq!(game.current_state, GameState::Title);

        assert!(!runtime_bridge::consume_pending_window_close_request());
        click_button(&mut game, SampleGameButtonAction::ExitGame.action_id());
        assert!(runtime_bridge::consume_pending_window_close_request());
        assert!(!runtime_bridge::consume_pending_window_close_request());
    }

    #[test]
    fn title_screen_exposes_2d_ui_and_text_commands() {
        let mut game = new_game_for_test("title-ui");
        game.update();

        let action_ids = button_action_ids(&game);
        let texts = text_command_texts(&game);

        assert!(texts.iter().any(|text| text == "MIYABI Box Survival"));
        assert!(texts.iter().any(|text| text.contains("Arrow Keys / WASD")));
        assert!(action_ids
            .iter()
            .any(|action_id| action_id == SampleGameButtonAction::StartGame.action_id()));
        assert!(action_ids
            .iter()
            .any(|action_id| action_id == SampleGameButtonAction::Start3dArena.action_id()));
        assert!(action_ids
            .iter()
            .any(|action_id| action_id == SampleGameButtonAction::ExitGame.action_id()));

        println!(
            "[c2-04][2d-title] buttons={} text_commands={} actions={:?}",
            action_ids.len(),
            texts.len(),
            action_ids
        );
    }

    #[test]
    fn headless_g2_stability_run_reaches_clear_with_safe_corridor() {
        let mut game = new_game_for_test("g2-stability");
        click_button(&mut game, SampleGameButtonAction::StartGame.action_id());

        for _ in 0..110_000 {
            if game.current_state != GameState::InGame {
                break;
            }

            keep_safe_corridor(&mut game);
            game.input_state = ffi::InputState::default();
            game.update();
        }

        assert_eq!(game.current_state, GameState::Result);
        assert!(game.result_is_clear);
        assert!(game.survival_time_sec >= 1800.0);
        assert!(game.hp > 0);
    }

    #[test]
    fn movement_preset_toggle_persists_and_changes_runtime_input() {
        let mut game = new_game_for_test("movement-preset");
        click_button(&mut game, SampleGameButtonAction::StartGame.action_id());

        let before_arrow = player_position(&game);
        game.input_state.w_key = true;
        game.update();
        let after_arrow = player_position(&game);
        assert_eq!(after_arrow.y, before_arrow.y);

        game.input_state = ffi::InputState::default();
        game.input_state.p_key = true;
        game.update();
        game.input_state.p_key = false;
        game.update();

        assert_eq!(
            game.save_data.settings.movement_preset,
            MovementPreset::Wasd
        );
        let persisted = save::load_or_default::<super::SaveData>(&game.save_file_path)
            .expect("movement preset toggle must persist settings");
        match persisted {
            save::LoadState::Loaded(data) => {
                assert_eq!(data.settings.movement_preset, MovementPreset::Wasd);
            }
            save::LoadState::Defaulted { .. } => {
                panic!("movement preset toggle unexpectedly defaulted");
            }
        }

        let before_wasd = player_position(&game);
        game.input_state.w_key = true;
        game.update();
        let after_wasd = player_position(&game);
        assert!(after_wasd.y > before_wasd.y);
    }

    #[test]
    fn sprite_animation_cycles_frames_and_speeds_up_when_player_moves() {
        let mut idle_game = new_game_for_test("sprite-animation-idle");
        click_button(&mut idle_game, SampleGameButtonAction::StartGame.action_id());
        let initial_idle = player_texture_handle(&idle_game);
        for _ in 0..20 {
            idle_game.input_state = ffi::InputState::default();
            idle_game.update();
        }
        assert_eq!(player_texture_handle(&idle_game), initial_idle);
        for _ in 0..15 {
            idle_game.input_state = ffi::InputState::default();
            idle_game.update();
        }
        assert_ne!(player_texture_handle(&idle_game), initial_idle);

        let mut moving_game = new_game_for_test("sprite-animation-active");
        click_button(&mut moving_game, SampleGameButtonAction::StartGame.action_id());
        let initial_active = player_texture_handle(&moving_game);
        for _ in 0..12 {
            moving_game.input_state = ffi::InputState::default();
            moving_game.input_state.right = true;
            moving_game.update();
        }
        assert_ne!(player_texture_handle(&moving_game), initial_active);
    }

    #[test]
    fn start_new_run_uses_external_level_layout() {
        let mut game = new_game_for_test("external-level");
        click_button(&mut game, SampleGameButtonAction::StartGame.action_id());

        assert_eq!(
            obstacle_positions(&game),
            vec![
                (120, 658),
                (180, 868),
                (220, 718),
                (320, 778),
                (420, 688),
                (520, 748),
                (620, 808),
                (680, 928),
            ]
        );
    }

    #[test]
    fn start_3d_arena_builds_3d_renderables() {
        let mut game = new_game_for_test("start-3d-arena");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());

        assert_eq!(game.current_state, GameState::InGame);
        assert_eq!(game.run_mode, RunMode::Arena3d);
        assert!(count_3d_renderables(&game) >= 5);
        assert!(
            game.renderables
                .iter()
                .all(|renderable| renderable.is_3d || renderable.mesh_id == MESH_ID_QUAD_2D)
        );
        assert!(
            game.renderables
                .iter()
                .filter(|renderable| renderable.is_3d)
                .all(|renderable| renderable.mesh_id == MESH_ID_ARENA_CUBE_3D)
        );
        assert!(
            game.renderables
                .iter()
                .filter(|renderable| renderable.is_3d)
                .all(|renderable| renderable.material_id == MATERIAL_ID_LIT_TEXTURED_3D)
        );
    }

    #[test]
    fn start_3d_arena_moves_player_on_xz_plane() {
        let mut game = new_game_for_test("move-3d-arena");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());

        let before = player_position_3d(&game);
        game.input_state = ffi::InputState::default();
        game.input_state.right = true;
        game.update();
        let after_x = player_position_3d(&game);
        assert!(after_x.x > before.x);
        assert_eq!(after_x.y, before.y);

        game.input_state = ffi::InputState::default();
        game.input_state.up = true;
        game.update();
        let after_z = player_position_3d(&game);
        assert!(after_z.z < after_x.z);
    }

    #[test]
    fn start_3d_arena_preserves_2d_text_overlay_and_3d_renderables() {
        let mut game = new_game_for_test("3d-overlay");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());
        game.input_state = ffi::InputState::default();
        game.update();

        let texts = text_command_texts(&game);
        let renderables_3d = count_3d_renderables(&game);
        let renderables_2d = game
            .renderables
            .iter()
            .filter(|renderable| !renderable.is_3d)
            .count();

        assert!(texts.iter().any(|text| text.starts_with("HP:")));
        assert!(texts.iter().any(|text| text == "3D Arena Prototype"));
        assert!(renderables_3d >= 5);

        println!(
            "[c2-04][3d-arena] renderables_3d={} renderables_2d={} text_commands={} hud={}",
            renderables_3d,
            renderables_2d,
            texts.len(),
            texts.iter().any(|text| text.starts_with("HP:"))
        );
    }

    #[test]
    fn start_3d_arena_spawns_falling_obstacle_renderables() {
        let mut game = new_game_for_test("3d-obstacle-spawn");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());
        game.input_state = ffi::InputState::default();
        game.update();

        let positions = obstacle_positions_3d(&game);
        assert_eq!(count_3d_obstacles(&game), 1);
        assert!(positions
            .iter()
            .any(|(x, y, z)| *x == 0 && *z == 48 && *y > 0));

        println!(
            "[g4-03][spawn] obstacles={} positions={:?}",
            count_3d_obstacles(&game),
            positions
        );
    }

    #[test]
    fn start_3d_arena_pause_and_back_to_title_flow_is_reachable() {
        let mut game = new_game_for_test("3d-pause-flow");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());
        assert_eq!(game.current_state, GameState::InGame);
        assert_eq!(game.run_mode, RunMode::Arena3d);

        game.input_state.esc_key = true;
        game.update();
        assert_eq!(game.current_state, GameState::Pause);

        game.input_state.esc_key = false;
        game.update();
        click_button(&mut game, SampleGameButtonAction::BackToTitle.action_id());
        assert_eq!(game.current_state, GameState::Title);

        println!("[c2-04][3d-flow] pause-and-back-to-title=pass");
        println!("[g4-02][pause-back] pause-and-back-to-title=pass");
    }

    #[test]
    fn start_3d_arena_game_over_reaches_result_screen() {
        let mut game = new_game_for_test("3d-game-over");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());

        game.hp = 0;
        game.input_state = ffi::InputState::default();
        game.update();

        assert_eq!(game.current_state, GameState::Result);
        assert_eq!(game.run_mode, RunMode::Arena3d);
        assert!(!game.result_is_clear);

        game.input_state = ffi::InputState::default();
        game.update();
        let texts = text_command_texts(&game);
        assert!(texts.iter().any(|text| text == "GAME OVER"));
        assert!(texts.iter().any(|text| text == "3D Arena Result"));
        assert!(texts.iter().any(|text| text.starts_with("Score:")));

        println!(
            "[g4-02][game-over] state={:?} run_mode={:?} texts={:?}",
            game.current_state, game.run_mode, texts
        );
    }

    #[test]
    fn start_3d_arena_clear_and_retry_stays_in_3d() {
        let mut game = new_game_for_test("3d-clear-retry");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());

        game.survival_time_sec = ARENA_CLEAR_TIME_SEC - FIXED_DT_SEC;
        game.input_state = ffi::InputState::default();
        game.update();

        assert_eq!(game.current_state, GameState::Result);
        assert_eq!(game.run_mode, RunMode::Arena3d);
        assert!(game.result_is_clear);

        game.input_state = ffi::InputState::default();
        game.update();
        let result_texts = text_command_texts(&game);
        assert!(result_texts.iter().any(|text| text == "CLEAR"));
        assert!(result_texts.iter().any(|text| text == "3D Arena Result"));
        assert!(result_texts
            .iter()
            .any(|text| text == &format!("Survival: {:.1} sec", ARENA_CLEAR_TIME_SEC)));

        click_button(&mut game, SampleGameButtonAction::RetryGame.action_id());
        assert_eq!(game.current_state, GameState::InGame);
        assert_eq!(game.run_mode, RunMode::Arena3d);
        assert!(game.survival_time_sec < 1.0);

        let retry_texts = text_command_texts(&game);
        assert!(retry_texts.iter().any(|text| text == "3D Arena Prototype"));
        assert!(retry_texts.iter().any(|text| text.starts_with("HP:")));

        println!(
            "[g4-02][clear-retry] result_texts={:?} retry_texts={:?}",
            result_texts, retry_texts
        );
    }

    #[test]
    fn start_3d_arena_obstacle_hits_can_reach_game_over() {
        let mut game = new_game_for_test("3d-obstacle-game-over");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());

        let mut frames = 0;
        while game.current_state == GameState::InGame && frames < 600 {
            game.input_state = ffi::InputState::default();
            game.update();
            frames += 1;
        }

        assert_eq!(game.current_state, GameState::Result);
        assert_eq!(game.run_mode, RunMode::Arena3d);
        assert!(!game.result_is_clear);
        assert!(game.hp <= 0);

        game.input_state = ffi::InputState::default();
        game.update();
        let texts = text_command_texts(&game);
        assert!(texts.iter().any(|text| text == "GAME OVER"));
        assert!(texts.iter().any(|text| text == "3D Arena Result"));

        println!(
            "[g4-03][fail] frames={} hp={} avoid_count={} texts={:?}",
            frames, game.hp, game.avoid_count, texts
        );
    }

    #[test]
    fn start_3d_arena_obstacle_avoidance_can_reach_clear() {
        let mut game = new_game_for_test("3d-obstacle-clear");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());

        let mut frames = 0;
        while game.current_state == GameState::InGame && frames < 11_000 {
            game.input_state = ffi::InputState::default();
            game.input_state.right = true;
            game.update();
            frames += 1;
        }

        assert_eq!(game.current_state, GameState::Result);
        assert_eq!(game.run_mode, RunMode::Arena3d);
        assert!(game.result_is_clear);
        assert!(game.hp > 0);
        assert!(game.survival_time_sec >= ARENA_CLEAR_TIME_SEC);
        assert!(game.avoid_count > 0);
        assert!(game.score > (ARENA_CLEAR_TIME_SEC as u32).saturating_mul(10));

        game.input_state = ffi::InputState::default();
        game.update();
        let texts = text_command_texts(&game);
        assert!(texts.iter().any(|text| text == "CLEAR"));
        assert!(texts.iter().any(|text| text == "3D Arena Result"));

        println!(
            "[g4-03][clear] frames={} hp={} avoid_count={} score={} texts={:?}",
            frames, game.hp, game.avoid_count, game.score, texts
        );
    }

    #[test]
    fn start_3d_arena_preserves_settings_across_result_and_retry() {
        let mut game = new_game_for_test("3d-settings");
        click_button(&mut game, SampleGameButtonAction::Start3dArena.action_id());

        game.input_state = ffi::InputState::default();
        game.input_state.p_key = true;
        game.update();
        game.input_state = ffi::InputState::default();
        game.update();

        assert_eq!(
            game.save_data.settings.movement_preset,
            MovementPreset::Wasd
        );

        game.hp = 0;
        game.input_state = ffi::InputState::default();
        game.update();
        assert_eq!(game.current_state, GameState::Result);

        click_button(&mut game, SampleGameButtonAction::RetryGame.action_id());
        assert_eq!(game.current_state, GameState::InGame);
        assert_eq!(
            game.save_data.settings.movement_preset,
            MovementPreset::Wasd
        );

        let before = player_position_3d(&game);
        game.input_state = ffi::InputState::default();
        game.input_state.w_key = true;
        game.update();
        let after = player_position_3d(&game);
        assert!(after.z < before.z);

        println!(
            "[g4][settings] preset={:?} before={:?} after={:?}",
            game.save_data.settings.movement_preset, before, after
        );
    }
}

// --- VTable Functions ---

fn ffi_null_pointer_error(function: &str, arg: &str) -> String {
    format!("[ffi][error] {function}: null pointer argument `{arg}`")
}

fn ffi_error(function: &str, detail: &str) -> String {
    format!("[ffi][error] {function}: {detail}")
}

#[no_mangle]
pub extern "C" fn create_game() -> *mut Game {
    Box::into_raw(Box::new(Game::new()))
}

#[no_mangle]
pub extern "C" fn destroy_game(game: *mut Game) {
    if !game.is_null() {
        unsafe {
            let boxed = Box::from_raw(game);
            boxed.persist_save_data("app_exit");
            drop(boxed);
        }
    }
}

#[no_mangle]
pub extern "C" fn serialize_game(game: *const Game) -> *mut c_char {
    if game.is_null() {
        eprintln!("{}", ffi_null_pointer_error("serialize_game", "game"));
        return ptr::null_mut();
    }
    let game = unsafe { &*game };
    let serialized = match serde_json::to_string(game) {
        Ok(serialized) => serialized,
        Err(err) => {
            eprintln!(
                "{}",
                ffi_error("serialize_game", &format!("serialization failed: {err}"))
            );
            return ptr::null_mut();
        }
    };
    match CString::new(serialized) {
        Ok(cstr) => cstr.into_raw(),
        Err(err) => {
            eprintln!(
                "{}",
                ffi_error(
                    "serialize_game",
                    &format!("serialization output contained NUL byte: {err}"),
                )
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn deserialize_game(json: *const c_char) -> *mut Game {
    if json.is_null() {
        eprintln!("{}", ffi_null_pointer_error("deserialize_game", "json"));
        return ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(json) };
    let r_str = match c_str.to_str() {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "{}",
                ffi_error("deserialize_game", &format!("invalid UTF-8 input: {err}"))
            );
            return ptr::null_mut();
        }
    };
    let mut game: Game = match serde_json::from_str(r_str) {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "{}",
                ffi_error("deserialize_game", &format!("JSON parse failed: {err}"))
            );
            return ptr::null_mut();
        }
    };
    game.save_data = game.save_data.sanitized();
    game.total_play_count = game.save_data.progress.total_play_count;
    game.save_file_path = PathBuf::from(SAVE_FILE_REL_PATH);
    game.sample_loop = SampleGameLoop::from_state_and_mode(
        sample_state_for_game_state(game.current_state),
        sample_run_mode_for_run_mode(game.run_mode),
    );
    game.apply_runtime_audio_settings();
    game.apply_runtime_fullscreen_setting();
    game.apply_runtime_bgm_for_state();
    // Re-initialize non-serializable fields
    game.asset_server = AssetServer::new();
    // ... etc. for other non-serde fields
    Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn free_serialized_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[no_mangle]
pub extern "C" fn update_game(game: *mut Game) -> GameState {
    if game.is_null() {
        eprintln!("{}", ffi_null_pointer_error("update_game", "game"));
        return GameState::Title;
    }
    let game = unsafe { &mut *game };
    game.update();
    game.current_state
}

#[no_mangle]
pub extern "C" fn get_renderables(game: *mut Game) -> RenderableObjectSlice {
    if game.is_null() {
        eprintln!("{}", ffi_null_pointer_error("get_renderables", "game"));
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
        eprintln!("{}", ffi_null_pointer_error("get_asset_commands", "game"));
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
        eprintln!("{}", ffi_null_pointer_error("clear_asset_commands", "game"));
        return;
    }
    let game = unsafe { &mut *game };
    game.asset_commands.clear();
}

#[no_mangle]
pub extern "C" fn notify_asset_loaded(game: *mut Game, request_id: u32, asset_id: u32) {
    if game.is_null() {
        eprintln!("{}", ffi_null_pointer_error("notify_asset_loaded", "game"));
        return;
    }
    let game = unsafe { &mut *game };
    if let Some(request) = game.asset_server.pending_requests.remove(&request_id) {
        if let Some(handle) = game.asset_server.texture_handle_map.get_mut(&request.path) {
            game.texture_map.insert(*handle, asset_id);
        }
    }
}

#[no_mangle]
pub extern "C" fn update_input_state(game: *mut Game, input: *const ffi::InputState) {
    if game.is_null() {
        eprintln!("{}", ffi_null_pointer_error("update_input_state", "game"));
        return;
    }
    if input.is_null() {
        eprintln!("{}", ffi_null_pointer_error("update_input_state", "input"));
        return;
    }
    let game = unsafe { &mut *game };
    let input = unsafe { &*input };
    game.input_state = *input;
}

#[no_mangle]
pub extern "C" fn get_asset_command_path_cstring(command: *const ffi::AssetCommand) -> *mut c_char {
    if command.is_null() {
        eprintln!(
            "{}",
            ffi_null_pointer_error("get_asset_command_path_cstring", "command")
        );
        return ptr::null_mut();
    }
    let command = unsafe { &*command };
    match CString::new(command.path.as_str()) {
        Ok(cstr) => cstr.into_raw(),
        Err(err) => {
            eprintln!(
                "{}",
                ffi_error(
                    "get_asset_command_path_cstring",
                    &format!("command.path contained NUL byte: {err}"),
                )
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn get_text_commands(game: *mut Game) -> TextCommandSlice {
    if game.is_null() {
        eprintln!("{}", ffi_null_pointer_error("get_text_commands", "game"));
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
        eprintln!(
            "{}",
            ffi_null_pointer_error("get_text_command_text_cstring", "command")
        );
        return ptr::null_mut();
    }
    let command = unsafe { &*command };
    match CString::new(command.text.as_str()) {
        Ok(cstr) => cstr.into_raw(),
        Err(err) => {
            eprintln!(
                "{}",
                ffi_error(
                    "get_text_command_text_cstring",
                    &format!("command.text contained NUL byte: {err}"),
                )
            );
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn free_cstring(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}
