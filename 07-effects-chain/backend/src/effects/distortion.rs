use super::Effect;

pub struct Distortion {
    drive: f32,
    level: f32,
}

impl Distortion {
    pub fn new(drive: f32, level: f32) -> Self {
        Distortion { drive, level }
    }
}

impl Effect for Distortion {
    fn process(&mut self, sample: f32) -> f32 {
        (sample * self.drive).tanh() * self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_never_leaves_unit_range_even_for_huge_input() {
        let mut d = Distortion::new(50.0, 1.0);
        for i in -1000..=1000 {
            let y = d.process(i as f32 / 100.0);
            assert!((-1.0..=1.0).contains(&y), "sample {y} escaped [-1,1]");
        }
    }

    #[test]
    fn more_drive_pushes_quiet_signals_closer_to_the_ceiling() {
        let mut soft = Distortion::new(1.0, 1.0);
        let mut hard = Distortion::new(20.0, 1.0);
        let x = 0.1;
        assert!(hard.process(x) > soft.process(x) * 5.0);
    }

    #[test]
    fn shape_is_symmetric_so_no_dc_offset_is_added() {
        let mut d = Distortion::new(8.0, 0.9);
        assert_eq!(d.process(0.3), -d.process(-0.3));
    }
}
