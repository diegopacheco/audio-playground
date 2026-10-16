use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

const SPEED_LIMIT: f64 = 120.0;
const MIN_DISTANCE: f64 = 0.3;

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub azimuth: f64,
    pub elevation: f64,
    pub distance: f64,
}

impl Position {
    pub fn from_cartesian(x: f64, y: f64, z: f64) -> Self {
        let horizontal = x.hypot(y);
        Position {
            azimuth: x.atan2(y).to_degrees(),
            elevation: z.atan2(horizontal).to_degrees(),
            distance: horizontal.hypot(z),
        }
    }

    pub fn direction(&self) -> (f64, f64, f64) {
        let az = self.azimuth.to_radians();
        let el = self.elevation.to_radians();
        (az.sin() * el.cos(), az.cos() * el.cos(), el.sin())
    }
}

fn default_period() -> f64 {
    4.0
}

fn default_radius() -> f64 {
    1.5
}

fn default_one() -> f64 {
    1.0
}

fn default_speed() -> f64 {
    20.0
}

fn default_closest() -> f64 {
    2.0
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Trajectory {
    Circle {
        #[serde(default = "default_period")]
        period: f64,
        #[serde(default = "default_radius")]
        distance: f64,
    },
    Figure8 {
        #[serde(default = "default_period")]
        period: f64,
        #[serde(default = "default_radius")]
        size: f64,
    },
    Static {
        #[serde(default)]
        azimuth: f64,
        #[serde(default)]
        elevation: f64,
        #[serde(default = "default_one")]
        distance: f64,
    },
    Flyby {
        #[serde(default = "default_speed")]
        speed: f64,
        #[serde(default = "default_closest")]
        closest: f64,
    },
}

fn require(ok: bool, message: &str) -> Result<(), AppError> {
    if ok {
        Ok(())
    } else {
        Err(AppError::InvalidParam(message.into()))
    }
}

impl Trajectory {
    pub fn validate(&self) -> Result<(), AppError> {
        match *self {
            Trajectory::Circle { period, distance } => {
                require(period > 0.1, "circle period must be above 0.1 s")?;
                require(distance >= MIN_DISTANCE, "circle distance must be >= 0.3 m")
            }
            Trajectory::Figure8 { period, size } => {
                require(period > 0.1, "figure8 period must be above 0.1 s")?;
                require(size >= MIN_DISTANCE, "figure8 size must be >= 0.3 m")
            }
            Trajectory::Static {
                azimuth,
                elevation,
                distance,
            } => {
                require(azimuth.is_finite(), "azimuth must be a number")?;
                require(
                    (-90.0..=90.0).contains(&elevation),
                    "elevation must be within -90..90",
                )?;
                require(distance >= MIN_DISTANCE, "static distance must be >= 0.3 m")
            }
            Trajectory::Flyby { speed, closest } => {
                require(
                    speed > 0.0 && speed <= SPEED_LIMIT,
                    "flyby speed must be within 0..120 m/s",
                )?;
                require(closest >= MIN_DISTANCE, "flyby closest must be >= 0.3 m")
            }
        }
    }

    pub fn position(&self, t: f64, duration: f64) -> Position {
        match *self {
            Trajectory::Circle { period, distance } => {
                let a = TAU * t / period;
                Position::from_cartesian(distance * a.sin(), distance * a.cos(), 0.0)
            }
            Trajectory::Figure8 { period, size } => {
                let a = TAU * t / period;
                Position::from_cartesian(
                    size * a.sin(),
                    size + 0.6 * size * (2.0 * a).sin(),
                    0.5 * size * a.cos(),
                )
            }
            Trajectory::Static {
                azimuth,
                elevation,
                distance,
            } => Position {
                azimuth,
                elevation,
                distance,
            },
            Trajectory::Flyby { speed, closest } => {
                let x = speed * (t - duration / 2.0);
                Position::from_cartesian(x, closest, 0.0)
            }
        }
    }

    pub fn max_distance(&self, duration: f64) -> f64 {
        let steps = 400;
        (0..=steps)
            .map(|i| {
                self.position(duration * i as f64 / steps as f64, duration)
                    .distance
            })
            .fold(0.0, f64::max)
    }
}

pub struct TrajectoryInfo {
    pub label: &'static str,
    pub description: &'static str,
    pub defaults: Trajectory,
}

pub fn catalog() -> Vec<TrajectoryInfo> {
    vec![
        TrajectoryInfo {
            label: "Circle",
            description: "Azimuth sweeps 360 degrees around the head once per period at a fixed distance.",
            defaults: Trajectory::Circle {
                period: default_period(),
                distance: default_radius(),
            },
        },
        TrajectoryInfo {
            label: "Figure 8",
            description: "A figure eight in front of the listener, rising and falling in elevation.",
            defaults: Trajectory::Figure8 {
                period: default_period(),
                size: default_radius(),
            },
        },
        TrajectoryInfo {
            label: "Static",
            description: "A fixed azimuth, elevation and distance.",
            defaults: Trajectory::Static {
                azimuth: 90.0,
                elevation: 0.0,
                distance: 1.0,
            },
        },
        TrajectoryInfo {
            label: "Flyby",
            description: "Passes left to right in front of the head with distance attenuation and Doppler shift.",
            defaults: Trajectory::Flyby {
                speed: default_speed(),
                closest: default_closest(),
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn azimuth_convention_is_front_zero_right_positive() {
        assert!(Position::from_cartesian(0.0, 1.0, 0.0).azimuth.abs() < 1e-9);
        assert!((Position::from_cartesian(1.0, 0.0, 0.0).azimuth - 90.0).abs() < 1e-9);
        assert!((Position::from_cartesian(-1.0, 0.0, 0.0).azimuth + 90.0).abs() < 1e-9);
        assert!((Position::from_cartesian(0.0, 0.0, 1.0).elevation - 90.0).abs() < 1e-9);
    }

    #[test]
    fn circle_visits_every_quadrant_in_one_period() {
        let c = Trajectory::Circle {
            period: 4.0,
            distance: 1.5,
        };
        assert!((c.position(1.0, 4.0).azimuth - 90.0).abs() < 1e-6);
        assert!((c.position(2.0, 4.0).azimuth.abs() - 180.0).abs() < 1e-6);
        assert!((c.position(3.0, 4.0).azimuth + 90.0).abs() < 1e-6);
        assert!((c.position(2.5, 4.0).distance - 1.5).abs() < 1e-9);
    }

    #[test]
    fn flyby_is_closest_at_the_midpoint_and_symmetric() {
        let f = Trajectory::Flyby {
            speed: 20.0,
            closest: 2.0,
        };
        assert!((f.position(2.0, 4.0).distance - 2.0).abs() < 1e-9);
        let early = f.position(0.5, 4.0);
        let late = f.position(3.5, 4.0);
        assert!((early.distance - late.distance).abs() < 1e-9);
        assert!(early.azimuth < 0.0 && late.azimuth > 0.0);
        assert!((f.max_distance(4.0) - 40.0f64.hypot(2.0)).abs() < 1e-6);
    }

    #[test]
    fn supersonic_flyby_is_rejected_because_the_delay_model_breaks() {
        let f = Trajectory::Flyby {
            speed: 400.0,
            closest: 2.0,
        };
        assert!(f.validate().is_err());
    }
}
