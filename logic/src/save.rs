use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SAVE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveEnvelope<T> {
    pub save_version: u32,
    pub payload: T,
}

impl<T> SaveEnvelope<T> {
    pub fn new(payload: T) -> Self {
        Self {
            save_version: SAVE_SCHEMA_VERSION,
            payload,
        }
    }
}

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    VersionMismatch { found: u32, expected: u32 },
}

impl Display for SaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "I/O error: {e}"),
            SaveError::Serde(e) => write!(f, "JSON error: {e}"),
            SaveError::VersionMismatch { found, expected } => {
                write!(
                    f,
                    "save schema version mismatch: found={found}, expected={expected}"
                )
            }
        }
    }
}

impl std::error::Error for SaveError {}

impl From<std::io::Error> for SaveError {
    fn from(value: std::io::Error) -> Self {
        SaveError::Io(value)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(value: serde_json::Error) -> Self {
        SaveError::Serde(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadState<T> {
    Loaded(T),
    Defaulted {
        data: T,
        backup_path: Option<PathBuf>,
    },
}

pub fn load_or_default<T>(path: &Path) -> Result<LoadState<T>, SaveError>
where
    T: DeserializeOwned + Default,
{
    if !path.exists() {
        return Ok(LoadState::Defaulted {
            data: T::default(),
            backup_path: None,
        });
    }

    let raw = fs::read(path)?;
    match serde_json::from_slice::<SaveEnvelope<T>>(&raw) {
        Ok(envelope) => {
            if envelope.save_version != SAVE_SCHEMA_VERSION {
                return Err(SaveError::VersionMismatch {
                    found: envelope.save_version,
                    expected: SAVE_SCHEMA_VERSION,
                });
            }
            Ok(LoadState::Loaded(envelope.payload))
        }
        Err(_) => {
            let backup_path = backup_corrupt_file(path)?;
            Ok(LoadState::Defaulted {
                data: T::default(),
                backup_path: Some(backup_path),
            })
        }
    }
}

pub fn save_to_path<T>(path: &Path, data: &T) -> Result<(), SaveError>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let envelope = SaveEnvelope::new(data);
    let serialized = serde_json::to_vec_pretty(&envelope)?;
    let tmp_path = temp_path_for(path);

    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(&serialized)?;
        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut tmp_path = path.to_path_buf();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!("{ext}.tmp"))
        .unwrap_or_else(|| "tmp".to_string());
    tmp_path.set_extension(extension);
    tmp_path
}

fn backup_corrupt_file(path: &Path) -> Result<PathBuf, SaveError> {
    let mut backup_path = path.to_path_buf();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!("{ext}.bak"))
        .unwrap_or_else(|| "bak".to_string());
    backup_path.set_extension(extension);
    fs::rename(path, &backup_path)?;
    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct TestData {
        value: u32,
    }

    fn temp_dir_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("miyabi-save-test-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = temp_dir_path();
        let path = dir.join("save_data.json");
        let data = TestData { value: 42 };

        save_to_path(&path, &data).unwrap();
        let loaded = load_or_default::<TestData>(&path).unwrap();

        match loaded {
            LoadState::Loaded(v) => assert_eq!(v, data),
            _ => panic!("expected loaded state"),
        }
    }

    #[test]
    fn load_missing_returns_default() {
        let dir = temp_dir_path();
        let path = dir.join("not_found.json");
        let loaded = load_or_default::<TestData>(&path).unwrap();

        match loaded {
            LoadState::Defaulted { data, backup_path } => {
                assert_eq!(data, TestData::default());
                assert!(backup_path.is_none());
            }
            _ => panic!("expected defaulted state"),
        }
    }

    #[test]
    fn load_corrupt_file_moves_backup() {
        let dir = temp_dir_path();
        let path = dir.join("save_data.json");
        fs::write(&path, b"this is not json").unwrap();

        let loaded = load_or_default::<TestData>(&path).unwrap();

        match loaded {
            LoadState::Defaulted { data, backup_path } => {
                assert_eq!(data, TestData::default());
                let backup = backup_path.expect("backup path should exist");
                assert!(backup.exists());
                assert!(!path.exists());
            }
            _ => panic!("expected defaulted state"),
        }
    }
}
