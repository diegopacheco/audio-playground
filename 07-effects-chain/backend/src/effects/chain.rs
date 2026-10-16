use super::Effect;

const SILENCE: f32 = 1e-4;

pub struct Chain {
    effects: Vec<Box<dyn Effect>>,
}

impl Chain {
    pub fn new(effects: Vec<Box<dyn Effect>>) -> Self {
        Chain { effects }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.effects
            .iter_mut()
            .fold(sample, |acc, effect| effect.process(acc))
    }

    pub fn tail_samples(&self) -> usize {
        self.effects.iter().map(|e| e.tail_samples()).sum()
    }

    pub fn render(&mut self, input: &[f32]) -> Vec<f32> {
        let tail = self.tail_samples();
        let mut out: Vec<f32> = input
            .iter()
            .copied()
            .chain(std::iter::repeat_n(0.0, tail))
            .map(|s| self.process(s))
            .collect();
        let last = out.iter().rposition(|s| s.abs() > SILENCE).unwrap_or(0) + 1;
        out.truncate(last.max(input.len()));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::bitcrusher::Bitcrusher;
    use crate::effects::delay::Delay;
    use crate::effects::distortion::Distortion;
    use crate::effects::reverb::Reverb;

    fn energy(v: &[f32]) -> f32 {
        v.iter().map(|s| s * s).sum()
    }

    fn short_burst() -> Vec<f32> {
        let mut v = vec![0.0; 200];
        for s in v.iter_mut().take(20) {
            *s = 0.3;
        }
        v
    }

    #[test]
    fn empty_chain_is_a_bypass() {
        let mut chain = Chain::new(vec![]);
        let input = vec![0.1, -0.4, 0.7];
        assert_eq!(chain.render(&input), input);
    }

    #[test]
    fn effects_run_in_the_order_given() {
        let mut chain = Chain::new(vec![
            Box::new(Delay::new(3, 1.0)),
            Box::new(Distortion::new(10.0, 1.0)),
        ]);
        let out = chain.render(&[0.5, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(out[0], 0.0);
        assert!((out[3] - (5.0f32).tanh()).abs() < 1e-6);
    }

    #[test]
    fn distortion_after_reverb_saturates_the_tail_unlike_distortion_before_reverb() {
        let rate = 8000;
        let input = short_burst();
        let cap = rate as usize * 10;
        let mut reverb_then_drive = Chain::new(vec![
            Box::new(Reverb::new(rate, 0.8, 1.0, cap)),
            Box::new(Distortion::new(20.0, 1.0)),
        ]);
        let mut drive_then_reverb = Chain::new(vec![
            Box::new(Distortion::new(20.0, 1.0)),
            Box::new(Reverb::new(rate, 0.8, 1.0, cap)),
        ]);
        let a = reverb_then_drive.render(&input);
        let b = drive_then_reverb.render(&input);
        let from = 2000;
        assert!(energy(&a[from..]) > energy(&b[from..b.len().min(a.len())]) * 2.0);
    }

    #[test]
    fn crushing_before_delay_differs_from_crushing_after_delay() {
        let input: Vec<f32> = (0..400).map(|i| (i as f32 * 0.05).sin() * 0.8).collect();
        let mut crush_then_delay = Chain::new(vec![
            Box::new(Bitcrusher::new(2, 1)),
            Box::new(Delay::new(37, 0.5)),
        ]);
        let mut delay_then_crush = Chain::new(vec![
            Box::new(Delay::new(37, 0.5)),
            Box::new(Bitcrusher::new(2, 1)),
        ]);
        let a = crush_then_delay.render(&input);
        let b = delay_then_crush.render(&input);
        let diff: f32 = a.iter().zip(&b).map(|(x, y)| (x - y).abs()).sum();
        assert!(diff > 1.0);
    }

    #[test]
    fn render_extends_output_to_hold_the_effect_tail() {
        let mut chain = Chain::new(vec![Box::new(Delay::new(50, 1.0))]);
        let out = chain.render(&[1.0; 10]);
        assert_eq!(out.len(), 60);
        assert_eq!(out[59], 1.0);
    }
}
