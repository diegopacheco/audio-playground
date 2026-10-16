use crate::fingerprint::hasher::Fingerprint;
use crate::fingerprint::index::Index;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub track: u32,
    pub score: u32,
    pub offset: i64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MatchResult {
    pub candidates: Vec<Candidate>,
    pub histogram: Vec<(i64, u32)>,
}

pub trait Matcher: Send + Sync {
    fn rank(&self, index: &dyn Index, query: &[Fingerprint]) -> MatchResult;
}

pub struct OffsetHistogramMatcher {
    pub max_candidates: usize,
}

impl Matcher for OffsetHistogramMatcher {
    fn rank(&self, index: &dyn Index, query: &[Fingerprint]) -> MatchResult {
        let mut counts: HashMap<(u32, i64), u32> = HashMap::new();
        for q in query {
            for &(track, t) in index.lookup(q.hash) {
                *counts.entry((track, t as i64 - q.t as i64)).or_default() += 1;
            }
        }
        let mut best: HashMap<u32, Candidate> = HashMap::new();
        for (&(track, offset), &score) in &counts {
            let entry = best.entry(track).or_insert(Candidate {
                track,
                score: 0,
                offset,
            });
            if score > entry.score || (score == entry.score && offset < entry.offset) {
                entry.score = score;
                entry.offset = offset;
            }
        }
        let mut candidates: Vec<Candidate> = best.into_values().collect();
        candidates.sort_by(|a, b| b.score.cmp(&a.score).then(a.track.cmp(&b.track)));
        candidates.truncate(self.max_candidates);
        let histogram = match candidates.first() {
            Some(winner) => {
                let mut bins: Vec<(i64, u32)> = counts
                    .iter()
                    .filter(|((track, _), _)| *track == winner.track)
                    .map(|((_, offset), count)| (*offset, *count))
                    .collect();
                bins.sort();
                bins
            }
            None => Vec::new(),
        };
        MatchResult {
            candidates,
            histogram,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::index::MemoryIndex;

    fn prints(pairs: &[(u32, u32)]) -> Vec<Fingerprint> {
        pairs
            .iter()
            .map(|&(hash, t)| Fingerprint { hash, t })
            .collect()
    }

    #[test]
    fn consistent_time_offset_beats_scattered_hash_hits() {
        let mut index = MemoryIndex::default();
        index.insert(0, &prints(&[(1, 100), (2, 101), (3, 102), (4, 103)]));
        index.insert(1, &prints(&[(1, 5), (2, 40), (3, 77), (4, 300)]));
        let query = prints(&[(1, 0), (2, 1), (3, 2), (4, 3)]);
        let result = OffsetHistogramMatcher { max_candidates: 5 }.rank(&index, &query);
        assert_eq!(
            result.candidates[0],
            Candidate {
                track: 0,
                score: 4,
                offset: 100
            }
        );
        assert_eq!(result.candidates[1].score, 1);
        assert_eq!(result.histogram, vec![(100, 4)]);
    }

    #[test]
    fn no_shared_hashes_means_no_candidates() {
        let mut index = MemoryIndex::default();
        index.insert(0, &prints(&[(1, 10)]));
        let result = OffsetHistogramMatcher { max_candidates: 5 }.rank(&index, &prints(&[(9, 0)]));
        assert!(result.candidates.is_empty());
        assert!(result.histogram.is_empty());
    }
}
