use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelData {
    pub player_spawn: SpawnPoint,
    pub obstacle_spawns: Vec<SpawnPoint>,
}

pub fn load_level_from_path(path: &Path) -> Result<LevelData, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str::<LevelData>(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{load_level_from_path, LevelData, SpawnPoint};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_level_path(test_name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("miyabi-level-{test_name}-{nanos}.json"))
    }

    #[test]
    fn load_level_round_trip() {
        let path = temp_level_path("round-trip");
        let expected = LevelData {
            player_spawn: SpawnPoint { x: 120.0, y: 80.0 },
            obstacle_spawns: vec![SpawnPoint { x: 400.0, y: 660.0 }],
        };
        fs::write(
            &path,
            serde_json::to_string(&expected).expect("level data must serialize"),
        )
        .expect("temp level file must be writable");

        let actual = load_level_from_path(&path).expect("level data must load");
        assert_eq!(actual, expected);
    }

    #[test]
    fn load_level_reports_parse_failure() {
        let path = temp_level_path("parse-error");
        fs::write(&path, "{ not-json }").expect("temp level file must be writable");

        let err = load_level_from_path(&path).expect_err("invalid json must fail");
        assert!(err.contains("failed to parse"));
    }
}
