use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use us_hr_core::{DeviceSettings, SettingsError};

const PRESET_SCHEMA_VERSION: u32 = 1;
const MAX_PRESET_NAME_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Preset {
    pub(crate) name: String,
    pub(crate) settings: DeviceSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PresetCollection {
    version: u32,
    pub(crate) presets: Vec<Preset>,
}

impl Default for PresetCollection {
    fn default() -> Self {
        Self {
            version: PRESET_SCHEMA_VERSION,
            presets: Vec::new(),
        }
    }
}

impl PresetCollection {
    pub(crate) fn upsert(
        &mut self,
        name: &str,
        settings: DeviceSettings,
    ) -> Result<usize, PresetError> {
        let name = validated_name(name)?;
        settings.validate()?;
        if let Some((index, preset)) = self
            .presets
            .iter_mut()
            .enumerate()
            .find(|(_, preset)| preset.name.eq_ignore_ascii_case(name))
        {
            name.clone_into(&mut preset.name);
            preset.settings = settings;
            return Ok(index);
        }
        self.presets.push(Preset {
            name: name.to_owned(),
            settings,
        });
        self.presets
            .sort_by_cached_key(|preset| preset.name.to_lowercase());
        self.presets
            .iter()
            .position(|preset| preset.name == name)
            .ok_or(PresetError::SavedPresetMissing)
    }

    pub(crate) fn remove(&mut self, index: usize) -> Result<Preset, PresetError> {
        if index >= self.presets.len() {
            return Err(PresetError::InvalidIndex(index));
        }
        Ok(self.presets.remove(index))
    }

    fn validate(&self) -> Result<(), PresetError> {
        for (index, preset) in self.presets.iter().enumerate() {
            let normalized_name = validated_name(&preset.name)?;
            if normalized_name != preset.name {
                return Err(PresetError::NonCanonicalName(preset.name.clone()));
            }
            if self.presets[..index]
                .iter()
                .any(|previous| previous.name.eq_ignore_ascii_case(&preset.name))
            {
                return Err(PresetError::DuplicateName(preset.name.clone()));
            }
            preset.settings.validate()?;
        }
        Ok(())
    }
}

pub(crate) struct PresetStore {
    path: PathBuf,
}

impl PresetStore {
    pub(crate) fn for_current_user() -> Result<Self, PresetError> {
        Ok(Self {
            path: config_directory()
                .ok_or(PresetError::ConfigDirectoryUnavailable)?
                .join("us-hr-custom-control")
                .join("presets.json"),
        })
    }

    #[cfg(test)]
    pub(crate) const fn at_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub(crate) fn load(&self) -> Result<PresetCollection, PresetError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PresetCollection::default());
            }
            Err(error) => return Err(error.into()),
        };
        let collection: PresetCollection = serde_json::from_slice(&bytes)?;
        if collection.version != PRESET_SCHEMA_VERSION {
            return Err(PresetError::UnsupportedVersion(collection.version));
        }
        collection.validate()?;
        Ok(collection)
    }

    pub(crate) fn save(&self, collection: &PresetCollection) -> Result<(), PresetError> {
        collection.validate()?;
        let parent = self.path.parent().ok_or(PresetError::InvalidStorePath)?;
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(temporary.as_file_mut(), collection)?;
        temporary.as_file_mut().write_all(b"\n")?;
        temporary.as_file_mut().sync_all()?;
        temporary.persist(&self.path).map_err(|error| error.error)?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn config_directory() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn config_directory() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn config_directory() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config"))
        })
}

fn validated_name(name: &str) -> Result<&str, PresetError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(PresetError::EmptyName);
    }
    if name.chars().count() > MAX_PRESET_NAME_LEN {
        return Err(PresetError::NameTooLong);
    }
    if name.chars().any(char::is_control) {
        return Err(PresetError::InvalidNameCharacter);
    }
    Ok(name)
}

#[derive(Debug, Error)]
pub(crate) enum PresetError {
    #[error("the operating-system configuration directory is unavailable")]
    ConfigDirectoryUnavailable,
    #[error("preset storage path has no parent directory")]
    InvalidStorePath,
    #[error("preset name cannot be empty")]
    EmptyName,
    #[error("preset name cannot exceed {MAX_PRESET_NAME_LEN} characters")]
    NameTooLong,
    #[error("preset name cannot contain control characters")]
    InvalidNameCharacter,
    #[error("stored preset name is not normalized: {0:?}")]
    NonCanonicalName(String),
    #[error("duplicate stored preset name: {0:?}")]
    DuplicateName(String),
    #[error("preset index {0} does not exist")]
    InvalidIndex(usize),
    #[error("saved preset could not be found after sorting")]
    SavedPresetMissing,
    #[error("preset file schema version {0} is unsupported")]
    UnsupportedVersion(u32),
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_loads_and_updates_presets() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = PresetStore::at_path(directory.path().join("presets.json"));
        let mut collection = PresetCollection::default();
        let initial = DeviceSettings::default();
        let index = collection.upsert("Streaming", initial)?;
        assert_eq!(index, 0);
        store.save(&collection)?;
        assert_eq!(store.load()?, collection);

        let mut updated = initial;
        updated.loopback_enabled = true;
        let index = collection.upsert("streaming", updated)?;
        assert_eq!(index, 0);
        assert_eq!(collection.presets.len(), 1);
        assert_eq!(collection.presets[0].settings, updated);
        Ok(())
    }

    #[test]
    fn validates_names_and_indices() {
        let mut collection = PresetCollection::default();
        assert!(collection.upsert("   ", DeviceSettings::default()).is_err());
        assert!(
            collection
                .upsert("line\nbreak", DeviceSettings::default())
                .is_err()
        );
        assert!(
            collection
                .upsert(
                    &"x".repeat(MAX_PRESET_NAME_LEN + 1),
                    DeviceSettings::default()
                )
                .is_err()
        );
        assert!(collection.remove(0).is_err());
    }

    #[test]
    fn missing_file_loads_as_empty_collection() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let store = PresetStore::at_path(directory.path().join("missing-presets.json"));
        assert_eq!(store.load()?.presets, []);
        Ok(())
    }

    #[test]
    fn rejects_unsupported_or_malformed_preset_files() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("presets.json");
        let store = PresetStore::at_path(path.clone());

        fs::write(&path, br#"{"version":2,"presets":[]}"#)?;
        assert!(matches!(
            store.load(),
            Err(PresetError::UnsupportedVersion(2))
        ));

        fs::write(&path, b"not json")?;
        assert!(matches!(store.load(), Err(PresetError::Json(_))));
        Ok(())
    }

    #[test]
    fn rejects_unsafe_persisted_presets_before_use() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("presets.json");
        let store = PresetStore::at_path(path.clone());
        let invalid_settings = DeviceSettings {
            monitor_balance: 128,
            ..DeviceSettings::default()
        };
        let collection = PresetCollection {
            version: PRESET_SCHEMA_VERSION,
            presets: vec![Preset {
                name: "Unsafe".to_owned(),
                settings: invalid_settings,
            }],
        };
        fs::write(&path, serde_json::to_vec(&collection)?)?;

        assert!(matches!(
            store.load(),
            Err(PresetError::Settings(
                SettingsError::MonitorBalanceOutOfRange(128)
            ))
        ));
        assert!(store.save(&collection).is_err());
        assert!(
            collection
                .clone()
                .upsert("Also unsafe", invalid_settings)
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn rejects_duplicate_and_noncanonical_stored_names() {
        let settings = DeviceSettings::default();
        let duplicate = PresetCollection {
            version: PRESET_SCHEMA_VERSION,
            presets: vec![
                Preset {
                    name: "Studio".to_owned(),
                    settings,
                },
                Preset {
                    name: "studio".to_owned(),
                    settings,
                },
            ],
        };
        let noncanonical = PresetCollection {
            version: PRESET_SCHEMA_VERSION,
            presets: vec![Preset {
                name: " Studio ".to_owned(),
                settings,
            }],
        };

        assert!(matches!(
            duplicate.validate(),
            Err(PresetError::DuplicateName(_))
        ));
        assert!(matches!(
            noncanonical.validate(),
            Err(PresetError::NonCanonicalName(_))
        ));
    }
}
