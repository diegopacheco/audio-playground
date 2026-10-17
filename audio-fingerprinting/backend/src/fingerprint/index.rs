use crate::fingerprint::hasher::Fingerprint;
use std::collections::HashMap;

pub trait Index: Send + Sync {
    fn insert(&mut self, track: u32, prints: &[Fingerprint]);
    fn lookup(&self, hash: u32) -> &[(u32, u32)];
    fn len(&self) -> usize;
}

#[derive(Default)]
pub struct MemoryIndex {
    map: HashMap<u32, Vec<(u32, u32)>>,
    entries: usize,
}

impl Index for MemoryIndex {
    fn insert(&mut self, track: u32, prints: &[Fingerprint]) {
        for p in prints {
            self.map.entry(p.hash).or_default().push((track, p.t));
        }
        self.entries += prints.len();
    }

    fn lookup(&self, hash: u32) -> &[(u32, u32)] {
        self.map.get(&hash).map(|v| v.as_slice()).unwrap_or(&[])
    }

    fn len(&self) -> usize {
        self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_every_track_and_time_sharing_a_hash() {
        let mut index = MemoryIndex::default();
        index.insert(0, &[Fingerprint { hash: 7, t: 3 }]);
        index.insert(
            1,
            &[Fingerprint { hash: 7, t: 9 }, Fingerprint { hash: 8, t: 1 }],
        );
        assert_eq!(index.lookup(7), &[(0, 3), (1, 9)]);
        assert!(index.lookup(99).is_empty());
        assert_eq!(index.len(), 3);
    }
}
