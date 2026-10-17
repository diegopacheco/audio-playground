use crate::fingerprint::peaks::Peak;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint {
    pub hash: u32,
    pub t: u32,
}

pub trait Hasher: Send + Sync {
    fn hash(&self, peaks: &[Peak]) -> Vec<Fingerprint>;
}

pub struct PairHasher {
    pub min_dt: u32,
    pub max_dt: u32,
    pub max_df: u32,
    pub fan_out: usize,
}

pub fn pack(f1: u32, f2: u32, dt: u32) -> u32 {
    ((f1 & 0x3FF) << 22) | ((f2 & 0x3FF) << 12) | (dt & 0xFFF)
}

impl Hasher for PairHasher {
    fn hash(&self, peaks: &[Peak]) -> Vec<Fingerprint> {
        let mut sorted = peaks.to_vec();
        sorted.sort();
        let mut prints = Vec::new();
        for (i, anchor) in sorted.iter().enumerate() {
            let mut paired = 0;
            for target in &sorted[i + 1..] {
                let dt = target.t - anchor.t;
                if dt > self.max_dt || paired >= self.fan_out {
                    break;
                }
                if dt < self.min_dt || anchor.f.abs_diff(target.f) > self.max_df {
                    continue;
                }
                prints.push(Fingerprint {
                    hash: pack(anchor.f, target.f, dt),
                    t: anchor.t,
                });
                paired += 1;
            }
        }
        prints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hasher() -> PairHasher {
        PairHasher {
            min_dt: 1,
            max_dt: 10,
            max_df: 50,
            fan_out: 3,
        }
    }

    fn peaks() -> Vec<Peak> {
        vec![
            Peak { t: 0, f: 100 },
            Peak { t: 2, f: 120 },
            Peak { t: 3, f: 400 },
            Peak { t: 5, f: 90 },
            Peak { t: 30, f: 100 },
        ]
    }

    #[test]
    fn hashing_is_deterministic_and_order_independent() {
        let a = hasher().hash(&peaks());
        let mut shuffled = peaks();
        shuffled.reverse();
        let b = hasher().hash(&shuffled);
        assert_eq!(a, b);
        assert_eq!(a, hasher().hash(&peaks()));
    }

    #[test]
    fn pairs_only_peaks_inside_the_target_zone() {
        let prints = hasher().hash(&peaks());
        assert_eq!(
            prints,
            vec![
                Fingerprint {
                    hash: pack(100, 120, 2),
                    t: 0
                },
                Fingerprint {
                    hash: pack(100, 90, 5),
                    t: 0
                },
                Fingerprint {
                    hash: pack(120, 90, 3),
                    t: 2
                },
            ]
        );
    }

    #[test]
    fn hash_is_invariant_to_absolute_time_so_clips_match_songs() {
        let shifted: Vec<Peak> = peaks()
            .iter()
            .map(|p| Peak {
                t: p.t + 1000,
                f: p.f,
            })
            .collect();
        let a: Vec<u32> = hasher().hash(&peaks()).iter().map(|p| p.hash).collect();
        let b: Vec<u32> = hasher().hash(&shifted).iter().map(|p| p.hash).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn each_field_changes_the_hash() {
        let base = pack(10, 20, 3);
        assert_ne!(base, pack(11, 20, 3));
        assert_ne!(base, pack(10, 21, 3));
        assert_ne!(base, pack(10, 20, 4));
    }
}
