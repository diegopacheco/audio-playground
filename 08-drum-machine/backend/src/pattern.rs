use crate::error::AppError;
use crate::instruments;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STEPS: usize = 16;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub tracks: BTreeMap<String, Vec<bool>>,
}

impl Pattern {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.tracks.is_empty() {
            return Err(AppError::InvalidPattern("no tracks".into()));
        }
        let known = instruments::names();
        for (name, steps) in &self.tracks {
            if !known.contains(&name.as_str()) {
                return Err(AppError::InvalidPattern(format!(
                    "unknown instrument {name}"
                )));
            }
            if steps.len() != STEPS {
                return Err(AppError::InvalidPattern(format!(
                    "{name} has {} steps, expected {STEPS}",
                    steps.len()
                )));
            }
        }
        Ok(())
    }

    pub fn from_strings(rows: &[(&str, &str)]) -> Self {
        let tracks = rows
            .iter()
            .map(|(name, row)| (name.to_string(), row.chars().map(|c| c == 'x').collect()))
            .collect();
        Self { tracks }
    }
}

pub fn validate_name(name: &str) -> Result<(), AppError> {
    let ok = !name.is_empty()
        && name.len() <= 40
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        Err(AppError::InvalidPattern(format!("bad name {name}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixteen_steps_per_known_instrument_is_valid() {
        let p =
            Pattern::from_strings(&[("kick", "x...x...x...x..."), ("hihat", "x.x.x.x.x.x.x.x.")]);
        assert_eq!(p.validate(), Ok(()));
    }

    #[test]
    fn a_track_without_sixteen_steps_must_be_rejected_so_the_grid_never_misaligns() {
        let short = Pattern::from_strings(&[("kick", "x...x...x...x..")]);
        let long = Pattern::from_strings(&[("kick", "x...x...x...x....")]);
        assert!(matches!(short.validate(), Err(AppError::InvalidPattern(_))));
        assert!(matches!(long.validate(), Err(AppError::InvalidPattern(_))));
    }

    #[test]
    fn unknown_instrument_must_be_rejected_because_it_cannot_be_rendered() {
        let p = Pattern::from_strings(&[("cowbell", "x...............")]);
        assert!(p.validate().is_err());
    }

    #[test]
    fn empty_pattern_must_be_rejected() {
        let p = Pattern {
            tracks: BTreeMap::new(),
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn names_must_be_url_safe() {
        assert!(validate_name("boom-bap_2").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("../etc").is_err());
        assert!(validate_name("a b").is_err());
    }
}
