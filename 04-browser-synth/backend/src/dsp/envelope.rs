pub trait Envelope {
    fn level(&self, t: f32, gate: f32) -> f32;
    fn release_time(&self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Adsr {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Adsr {
    fn held_level(&self, t: f32) -> f32 {
        if t < self.attack {
            t / self.attack
        } else if t < self.attack + self.decay {
            1.0 - (1.0 - self.sustain) * (t - self.attack) / self.decay
        } else {
            self.sustain
        }
    }
}

impl Envelope for Adsr {
    fn level(&self, t: f32, gate: f32) -> f32 {
        if t < 0.0 {
            return 0.0;
        }
        if t < gate {
            return self.held_level(t);
        }
        let start = self.held_level(gate);
        let elapsed = t - gate;
        if elapsed >= self.release {
            0.0
        } else {
            start * (1.0 - elapsed / self.release)
        }
    }

    fn release_time(&self) -> f32 {
        self.release
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADSR: Adsr = Adsr {
        attack: 0.1,
        decay: 0.2,
        sustain: 0.5,
        release: 0.4,
    };

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn attack_ramps_linearly_from_silence_to_peak() {
        assert!(close(ADSR.level(0.0, 10.0), 0.0));
        assert!(close(ADSR.level(0.05, 10.0), 0.5));
        assert!(close(ADSR.level(0.1, 10.0), 1.0));
    }

    #[test]
    fn decay_falls_from_peak_to_sustain() {
        assert!(close(ADSR.level(0.2, 10.0), 0.75));
        assert!(close(ADSR.level(0.3, 10.0), 0.5));
    }

    #[test]
    fn sustain_holds_while_key_is_down() {
        assert!(close(ADSR.level(2.0, 10.0), 0.5));
        assert!(close(ADSR.level(9.9, 10.0), 0.5));
    }

    #[test]
    fn release_fades_from_sustain_to_silence() {
        assert!(close(ADSR.level(1.2, 1.0), 0.25));
        assert!(close(ADSR.level(1.4, 1.0), 0.0));
        assert!(close(ADSR.level(3.0, 1.0), 0.0));
    }

    #[test]
    fn early_release_starts_from_current_level_without_a_jump() {
        let at_gate = ADSR.level(0.05, 10.0);
        assert!(close(ADSR.level(0.05, 0.05), at_gate));
        assert!(close(ADSR.level(0.25, 0.05), at_gate * 0.5));
    }
}
