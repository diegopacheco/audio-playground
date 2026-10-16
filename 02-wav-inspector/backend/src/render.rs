use crate::analysis::buckets;

pub trait Renderer {
    fn render(&self, mono: &[f32]) -> String;
}

pub struct AsciiRenderer {
    pub width: usize,
    pub height: usize,
}

impl AsciiRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        AsciiRenderer {
            width: width.clamp(8, 400),
            height: height.clamp(3, 100),
        }
    }

    fn row_of(&self, v: f32) -> usize {
        let v = v.clamp(-1.0, 1.0);
        ((1.0 - v) / 2.0 * (self.height - 1) as f32).round() as usize
    }
}

impl Renderer for AsciiRenderer {
    fn render(&self, mono: &[f32]) -> String {
        let center = (self.height - 1) / 2;
        let mut grid = vec![vec![' '; self.width]; self.height];
        for row in grid[center].iter_mut() {
            *row = '-';
        }
        if !mono.is_empty() {
            for (col, (start, end)) in buckets(mono.len(), self.width).into_iter().enumerate() {
                let slice = &mono[start..end.max(start + 1).min(mono.len())];
                let max = slice.iter().copied().fold(f32::MIN, f32::max);
                let min = slice.iter().copied().fold(f32::MAX, f32::min);
                if max.abs() < 1e-4 && min.abs() < 1e-4 {
                    continue;
                }
                for row in grid
                    .iter_mut()
                    .take(self.row_of(min) + 1)
                    .skip(self.row_of(max))
                {
                    row[col] = '#';
                }
            }
        }
        grid.into_iter()
            .map(|r| r.into_iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<&str> {
        s.split('\n').collect()
    }

    #[test]
    fn output_has_exact_requested_dimensions() {
        let out = AsciiRenderer::new(40, 9).render(&[0.3; 500]);
        let l = lines(&out);
        assert_eq!(l.len(), 9);
        assert!(l.iter().all(|r| r.chars().count() == 40));
    }

    #[test]
    fn silence_draws_only_the_center_line() {
        let out = AsciiRenderer::new(20, 7).render(&[0.0; 100]);
        let l = lines(&out);
        assert_eq!(l[3], "-".repeat(20));
        assert!(
            l.iter()
                .enumerate()
                .all(|(i, r)| i == 3 || r.trim().is_empty())
        );
    }

    #[test]
    fn full_scale_signal_fills_every_row() {
        let s: Vec<f32> = (0..400)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let out = AsciiRenderer::new(10, 5).render(&s);
        assert!(lines(&out).iter().all(|r| *r == "#".repeat(10)));
    }

    #[test]
    fn positive_dc_only_draws_in_upper_half() {
        let out = AsciiRenderer::new(10, 9).render(&[0.9; 100]);
        let l = lines(&out);
        assert!(l[0].contains('#'));
        assert!(l[5..].iter().all(|r| !r.contains('#')));
    }

    #[test]
    fn louder_section_is_taller_than_quiet_section() {
        let mut s = vec![0.1f32; 100];
        s.extend(vec![0.9f32; 100]);
        let s: Vec<f32> = s
            .iter()
            .enumerate()
            .map(|(i, v)| if i % 2 == 0 { *v } else { -*v })
            .collect();
        let out = AsciiRenderer::new(8, 21).render(&s);
        let count = |col: usize| {
            lines(&out)
                .iter()
                .filter(|r| r.chars().nth(col) == Some('#'))
                .count()
        };
        assert!(count(7) > count(0) * 3);
    }

    #[test]
    fn dimensions_are_clamped_to_sane_bounds() {
        let r = AsciiRenderer::new(0, 1000);
        assert_eq!((r.width, r.height), (8, 100));
    }
}
