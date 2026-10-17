mod clap;
mod hihat;
mod kick;
mod snare;

pub use clap::Clap;
pub use hihat::HiHat;
pub use kick::Kick;
pub use snare::Snare;

pub trait Instrument: Send + Sync {
    fn name(&self) -> &'static str;
    fn render(&self, sample_rate: u32) -> Vec<f32>;
}

pub fn all() -> Vec<Box<dyn Instrument>> {
    vec![
        Box::new(Kick),
        Box::new(Snare),
        Box::new(HiHat),
        Box::new(Clap),
    ]
}

pub fn names() -> Vec<&'static str> {
    all().iter().map(|i| i.name()).collect()
}

fn samples_for(seconds: f32, sample_rate: u32) -> usize {
    (seconds * sample_rate as f32) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::{band_energy_ratio, zero_crossing_rate};

    const SR: u32 = 44100;

    fn peak(s: &[f32]) -> f32 {
        s.iter().fold(0.0f32, |m, v| m.max(v.abs()))
    }

    #[test]
    fn kick_energy_must_sit_in_the_bass_so_it_sounds_like_a_kick() {
        let kick = Kick.render(SR);
        let low = band_energy_ratio(&kick, SR, 0.0, 250.0);
        assert!(low > 0.8, "kick low band ratio {low}");
    }

    #[test]
    fn hihat_energy_must_sit_in_the_treble_so_it_sounds_like_cymbals() {
        let hat = HiHat.render(SR);
        let high = band_energy_ratio(&hat, SR, 5000.0, 22050.0);
        assert!(high > 0.7, "hihat high band ratio {high}");
    }

    #[test]
    fn hihat_must_cross_zero_far_more_often_than_kick() {
        let kick_zcr = zero_crossing_rate(&Kick.render(SR)[..2048], SR);
        let hat_zcr = zero_crossing_rate(&HiHat.render(SR)[..2048], SR);
        assert!(kick_zcr < 500.0, "kick zcr {kick_zcr}");
        assert!(hat_zcr > 5000.0, "hihat zcr {hat_zcr}");
        assert!(hat_zcr > 10.0 * kick_zcr);
    }

    #[test]
    fn snare_must_be_brighter_than_kick_and_darker_than_hihat() {
        let zcr = |s: Vec<f32>| zero_crossing_rate(&s[..2048], SR);
        let kick = zcr(Kick.render(SR));
        let snare = zcr(Snare.render(SR));
        let hat = zcr(HiHat.render(SR));
        assert!(kick < snare && snare < hat, "{kick} {snare} {hat}");
    }

    #[test]
    fn kick_pitch_must_sweep_down_so_it_has_a_punchy_attack() {
        let kick = Kick.render(SR);
        let early = zero_crossing_rate(&kick[..1024], SR);
        let late = zero_crossing_rate(&kick[8192..12288], SR);
        assert!(early > 1.5 * late, "early {early} late {late}");
    }

    #[test]
    fn every_instrument_must_decay_to_silence_so_hits_do_not_smear() {
        for inst in all() {
            let s = inst.render(SR);
            let p = peak(&s);
            let tail = peak(&s[s.len() * 9 / 10..]);
            assert!(p > 0.3, "{} peak {p}", inst.name());
            assert!(p <= 1.0, "{} clips {p}", inst.name());
            assert!(tail < 0.05 * p, "{} tail {tail}", inst.name());
        }
    }

    #[test]
    fn hihat_must_be_shorter_than_kick() {
        assert!(HiHat.render(SR).len() < Kick.render(SR).len());
    }
}
