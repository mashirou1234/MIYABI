use miyabi_logic::*;
use serde::{Serialize, Deserialize};
use std::os::raw::c_char;
use std::ffi::CString;
use std::collections::HashSet;
use std::any::Any;

// --- Game-specific Components ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Player;
impl Component for Player {
    const COMPONENT_TYPE: ComponentType = ComponentType::Player;
}

// --- Game-specific Systems and Extensions for the World ---

pub trait GameWorldExt {
    fn spawn_with_game_components<B: ComponentBundle>(&mut self, bundle: B) -> Entity;
    fn run_input_system(&mut self);
    fn run_ui_system(&mut self);
    fn run_all_logic_systems(&mut self);
}

impl GameWorldExt for InternalWorld {
    fn spawn_with_game_components<B: ComponentBundle>(&mut self, bundle: B) -> Entity {
        let types = B::get_component_types();
        let archetype_idx = self.get_or_create_archetype(types.clone());

        // HACK: Manually ensure storage for game-specific components exists.
        // A proper engine would use a component registration system.
        let archetype = &mut self.archetypes[archetype_idx];
        if types.contains(&ComponentType::Player) && !archetype.storage.contains_key(&ComponentType::Player) {
            archetype.storage.insert(ComponentType::Player, Box::new(Vec::<Player>::new()));
        }

        // Now call the original `push_to_storage`
        bundle.push_to_storage(archetype);

        let entity_idx_in_archetype = archetype.entity_count;
        archetype.entity_count += 1;
        let entity = Entity(self.next_entity);
        self.next_entity += 1;
        self.entities.insert(entity, (archetype_idx, entity_idx_in_archetype));
        entity
    }

    fn run_input_system(&mut self) {
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

    fn run_ui_system(&mut self) {
        self.text_commands.clear();
        self.text_commands.push(ffi::TextCommand {
            text: "Hello from Sample Game!".to_string(),
            position: ffi::Vec2 { x: 100.0, y: 100.0 },
            font_size: 32.0,
            color: ffi::Vec4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 },
        });
    }

    fn run_all_logic_systems(&mut self) {
        self.run_input_system();
        self.run_movement_system();
        self.process_asset_server();
        self.run_ui_system();
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
pub struct TextCommandSlice {
    pub ptr: *const ffi::TextCommand,
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
    pub get_text_commands: extern "C" fn(*mut World) -> TextCommandSlice,
    pub get_text_command_text_cstring: extern "C" fn(&ffi::TextCommand) -> *const c_char,
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
        get_text_commands: rust_get_text_commands,
        get_text_command_text_cstring: rust_get_text_command_text_cstring,
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
extern "C" fn rust_get_text_commands(world: *mut World) -> TextCommandSlice {
    let world = unsafe { &mut *(world as *mut InternalWorld) };
    TextCommandSlice {
        ptr: world.text_commands.as_ptr(),
        len: world.text_commands.len(),
    }
}

#[no_mangle]
extern "C" fn rust_get_text_command_text_cstring(command: &ffi::TextCommand) -> *const c_char {
    CString::new(command.text.as_str()).unwrap().into_raw()
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
    world.spawn_with_game_components((
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
    world.run_all_logic_systems();
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
    world.texture_map = Default::default();
    world.renderables = Vec::new();
    world.asset_commands = Vec::new();
    world.text_commands = Vec::new();
    
    for archetype in &mut world.archetypes {
        archetype.storage = Default::default();
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