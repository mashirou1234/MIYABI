use miyabi_logic::{ffi, update_input_state, AssetServer, Game, GameState, InternalWorld, SaveData};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::ptr;

fn make_test_game() -> Game {
    Game {
        world: InternalWorld::new(),
        current_state: GameState::Title,
        asset_server: AssetServer::new(),
        texture_map: HashMap::new(),
        input_state: ffi::InputState::default(),
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
        total_play_count: 0,
        save_data: SaveData::default(),
        player_texture_handle: 0,
        obstacle_texture_handle: 0,
        obstacle_spawn_accumulator_sec: 0.0,
        esc_was_pressed: false,
        u_was_pressed: false,
        asset_integrity_tick: 0,
        reported_missing_texture_handles: HashSet::new(),
        reported_unresolved_texture_handles: HashSet::new(),
        reported_registry_inconsistency: false,
        save_file_path: PathBuf::from("save/save_data.json"),
    }
}

#[test]
fn update_input_state_ignores_null_input_pointer() {
    let mut game = make_test_game();
    let before = game.input_state;

    update_input_state(&mut game as *mut Game, ptr::null());

    assert_eq!(
        game.input_state, before,
        "null input must not mutate input_state"
    );
}

#[test]
fn update_input_state_accepts_empty_frame() {
    let mut game = make_test_game();
    let empty_input = ffi::InputState::default();

    update_input_state(&mut game as *mut Game, &empty_input as *const ffi::InputState);

    assert_eq!(
        game.input_state, empty_input,
        "empty frame input must be stored as-is"
    );
}
