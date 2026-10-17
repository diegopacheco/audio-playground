use std::sync::Mutex;

use crate::error::AppError;
use crate::preset::{Preset, built_in_presets};

pub struct PresetStore {
    presets: Mutex<Vec<Preset>>,
}

impl PresetStore {
    pub fn with_built_ins() -> Self {
        Self {
            presets: Mutex::new(built_in_presets()),
        }
    }

    pub fn list(&self) -> Result<Vec<Preset>, AppError> {
        let presets = self
            .presets
            .lock()
            .map_err(|_| AppError::StoreUnavailable)?;
        Ok(presets.clone())
    }

    pub fn save(&self, preset: Preset) -> Result<Preset, AppError> {
        preset.validate()?;
        let preset = Preset {
            name: preset.name.trim().to_string(),
            ..preset
        };
        let mut presets = self
            .presets
            .lock()
            .map_err(|_| AppError::StoreUnavailable)?;
        match presets.iter_mut().find(|p| p.name == preset.name) {
            Some(existing) => *existing = preset.clone(),
            None => presets.push(preset.clone()),
        }
        Ok(preset)
    }

    pub fn delete(&self, name: &str) -> Result<(), AppError> {
        let mut presets = self
            .presets
            .lock()
            .map_err(|_| AppError::StoreUnavailable)?;
        let before = presets.len();
        presets.retain(|p| p.name != name);
        if presets.len() == before {
            Err(AppError::NotFound(name.to_string()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_len(store: &PresetStore) -> usize {
        store.list().map(|l| l.len()).unwrap_or(0)
    }

    #[test]
    fn saving_same_name_replaces_instead_of_duplicating() {
        let store = PresetStore::with_built_ins();
        let before = store_len(&store);
        let mut p = built_in_presets().remove(0);
        p.gain = 0.1;
        assert!(store.save(p).is_ok());
        assert_eq!(store_len(&store), before);
        let saved = store.list().unwrap_or_default();
        assert!(saved.iter().any(|x| x.gain == 0.1));
    }

    #[test]
    fn invalid_preset_is_never_stored() {
        let store = PresetStore::with_built_ins();
        let before = store_len(&store);
        let mut p = built_in_presets().remove(0);
        p.name = "Broken".to_string();
        p.sustain = 2.0;
        assert!(store.save(p).is_err());
        assert_eq!(store_len(&store), before);
    }

    #[test]
    fn deleting_unknown_preset_reports_not_found() {
        let store = PresetStore::with_built_ins();
        assert!(matches!(store.delete("Nope"), Err(AppError::NotFound(_))));
        assert!(store.delete("Soft Sine").is_ok());
    }
}
