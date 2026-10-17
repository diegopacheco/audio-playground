use crate::error::AppError;
use crate::pattern::{Pattern, validate_name};
use std::collections::BTreeMap;
use std::sync::RwLock;

pub struct PatternStore {
    patterns: RwLock<BTreeMap<String, Pattern>>,
}

fn builtins() -> BTreeMap<String, Pattern> {
    let mut m = BTreeMap::new();
    m.insert(
        "four-on-the-floor".to_string(),
        Pattern::from_strings(&[
            ("kick", "x...x...x...x..."),
            ("snare", "................"),
            ("hihat", "..x...x...x...x."),
            ("clap", "....x.......x..."),
        ]),
    );
    m.insert(
        "boom-bap".to_string(),
        Pattern::from_strings(&[
            ("kick", "x.........x....."),
            ("snare", "....x.......x..."),
            ("hihat", "x.x.x.x.x.x.x.x."),
            ("clap", "................"),
        ]),
    );
    m.insert(
        "breakbeat".to_string(),
        Pattern::from_strings(&[
            ("kick", "x.x.......x.x..."),
            ("snare", "....x..x.x..x..x"),
            ("hihat", "xxxxxxxxxxxxxxxx"),
            ("clap", "................"),
        ]),
    );
    m
}

impl PatternStore {
    pub fn with_builtins() -> Self {
        Self {
            patterns: RwLock::new(builtins()),
        }
    }

    pub fn names(&self) -> Result<Vec<String>, AppError> {
        let guard = self
            .patterns
            .read()
            .map_err(|e| AppError::Io(e.to_string()))?;
        Ok(guard.keys().cloned().collect())
    }

    pub fn get(&self, name: &str) -> Result<Pattern, AppError> {
        let guard = self
            .patterns
            .read()
            .map_err(|e| AppError::Io(e.to_string()))?;
        guard
            .get(name)
            .cloned()
            .ok_or_else(|| AppError::NotFound(name.to_string()))
    }

    pub fn put(&self, name: &str, pattern: Pattern) -> Result<(), AppError> {
        validate_name(name)?;
        pattern.validate()?;
        let mut guard = self
            .patterns
            .write()
            .map_err(|e| AppError::Io(e.to_string()))?;
        guard.insert(name.to_string(), pattern);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_must_be_valid_so_the_ui_can_load_them() {
        let store = PatternStore::with_builtins();
        let names = store.names().unwrap();
        assert!(names.len() >= 3);
        for n in names {
            assert_eq!(store.get(&n).unwrap().validate(), Ok(()));
        }
    }

    #[test]
    fn saved_pattern_must_be_returned_unchanged() {
        let store = PatternStore::with_builtins();
        let p = Pattern::from_strings(&[("kick", "x.x.x.x.x.x.x.x.")]);
        store.put("mine", p.clone()).unwrap();
        assert_eq!(store.get("mine").unwrap(), p);
        assert!(store.names().unwrap().contains(&"mine".to_string()));
    }

    #[test]
    fn invalid_pattern_must_not_be_stored() {
        let store = PatternStore::with_builtins();
        let bad = Pattern::from_strings(&[("kick", "x.x")]);
        assert!(store.put("bad", bad).is_err());
        assert_eq!(store.get("bad"), Err(AppError::NotFound("bad".into())));
    }
}
