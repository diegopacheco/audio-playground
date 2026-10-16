use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Effect;
use super::bitcrusher::Bitcrusher;
use super::chain::Chain;
use super::delay::Delay;
use super::distortion::Distortion;
use super::echo::Echo;
use super::ms_to_samples;
use super::reverb::Reverb;
use crate::error::AppError;

const MAX_TAIL_SECONDS: u32 = 10;
const MAX_CHAIN: usize = 16;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct ParamSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub step: f32,
    pub unit: &'static str,
}

#[derive(Debug, Serialize)]
pub struct EffectSpec {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub params: &'static [ParamSpec],
}

#[derive(Debug, Deserialize, Clone)]
pub struct EffectConfig {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub params: HashMap<String, f32>,
}

const fn p(
    name: &'static str,
    label: &'static str,
    range: (f32, f32),
    default: f32,
    step: f32,
    unit: &'static str,
) -> ParamSpec {
    ParamSpec {
        name,
        label,
        min: range.0,
        max: range.1,
        default,
        step,
        unit,
    }
}

pub const SPECS: &[EffectSpec] = &[
    EffectSpec {
        kind: "delay",
        label: "Delay",
        description: "Single delayed copy mixed with the dry signal",
        params: &[
            p("time_ms", "Time", (1.0, 2000.0), 250.0, 1.0, "ms"),
            p("mix", "Mix", (0.0, 1.0), 0.5, 0.01, ""),
        ],
    },
    EffectSpec {
        kind: "echo",
        label: "Echo",
        description: "Delay line with feedback for repeating, decaying copies",
        params: &[
            p("time_ms", "Time", (1.0, 2000.0), 300.0, 1.0, "ms"),
            p("feedback", "Feedback", (0.0, 0.95), 0.5, 0.01, ""),
            p("mix", "Mix", (0.0, 1.0), 0.4, 0.01, ""),
        ],
    },
    EffectSpec {
        kind: "reverb",
        label: "Reverb",
        description: "Schroeder reverb: 4 parallel combs into 2 series allpass filters",
        params: &[
            p("room_size", "Room size", (0.0, 1.0), 0.7, 0.01, ""),
            p("mix", "Mix", (0.0, 1.0), 0.35, 0.01, ""),
        ],
    },
    EffectSpec {
        kind: "distortion",
        label: "Distortion",
        description: "tanh soft clipping driven by an input gain",
        params: &[
            p("drive", "Drive", (1.0, 50.0), 6.0, 0.1, "x"),
            p("level", "Level", (0.0, 1.0), 0.7, 0.01, ""),
        ],
    },
    EffectSpec {
        kind: "bitcrusher",
        label: "Bitcrusher",
        description: "Bit depth quantization plus sample-and-hold rate reduction",
        params: &[
            p("bits", "Bits", (1.0, 16.0), 6.0, 1.0, "bit"),
            p("downsample", "Downsample", (1.0, 32.0), 4.0, 1.0, "x"),
        ],
    },
];

struct Params<'a> {
    spec: &'static EffectSpec,
    values: &'a HashMap<String, f32>,
}

impl Params<'_> {
    fn get(&self, name: &str) -> Result<f32, AppError> {
        let spec = self
            .spec
            .params
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| AppError::InvalidParam(format!("{} has no {name}", self.spec.kind)))?;
        let value = self.values.get(name).copied().unwrap_or(spec.default);
        if !value.is_finite() || value < spec.min || value > spec.max {
            return Err(AppError::InvalidParam(format!(
                "{}.{name}={value} outside [{}, {}]",
                self.spec.kind, spec.min, spec.max
            )));
        }
        Ok(value)
    }
}

fn check_unknown(spec: &EffectSpec, values: &HashMap<String, f32>) -> Result<(), AppError> {
    match values
        .keys()
        .find(|k| !spec.params.iter().any(|p| p.name == k.as_str()))
    {
        Some(k) => Err(AppError::InvalidParam(format!("{} has no {k}", spec.kind))),
        None => Ok(()),
    }
}

pub fn build(config: &EffectConfig, sample_rate: u32) -> Result<Box<dyn Effect>, AppError> {
    let spec = SPECS
        .iter()
        .find(|s| s.kind == config.kind)
        .ok_or_else(|| AppError::UnknownEffect(config.kind.clone()))?;
    check_unknown(spec, &config.params)?;
    let p = Params {
        spec,
        values: &config.params,
    };
    let cap = (sample_rate * MAX_TAIL_SECONDS) as usize;
    let effect: Box<dyn Effect> = match spec.kind {
        "delay" => Box::new(Delay::new(
            ms_to_samples(p.get("time_ms")?, sample_rate),
            p.get("mix")?,
        )),
        "echo" => Box::new(Echo::new(
            ms_to_samples(p.get("time_ms")?, sample_rate),
            p.get("feedback")?,
            p.get("mix")?,
            cap,
        )),
        "reverb" => Box::new(Reverb::new(
            sample_rate,
            p.get("room_size")?,
            p.get("mix")?,
            cap,
        )),
        "distortion" => Box::new(Distortion::new(p.get("drive")?, p.get("level")?)),
        "bitcrusher" => Box::new(Bitcrusher::new(
            p.get("bits")?.round() as u32,
            p.get("downsample")?.round() as usize,
        )),
        other => return Err(AppError::UnknownEffect(other.to_string())),
    };
    Ok(effect)
}

pub fn build_chain(configs: &[EffectConfig], sample_rate: u32) -> Result<Chain, AppError> {
    if configs.len() > MAX_CHAIN {
        return Err(AppError::Config(format!(
            "chain has {} effects, limit is {MAX_CHAIN}",
            configs.len()
        )));
    }
    let effects = configs
        .iter()
        .map(|c| build(c, sample_rate))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Chain::new(effects))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(kind: &str, params: &[(&str, f32)]) -> EffectConfig {
        EffectConfig {
            kind: kind.into(),
            params: params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    #[test]
    fn every_listed_effect_can_be_built_from_its_defaults() {
        for spec in SPECS {
            assert!(
                build(&config(spec.kind, &[]), 44100).is_ok(),
                "{}",
                spec.kind
            );
            for param in spec.params {
                assert!(param.min <= param.default && param.default <= param.max);
            }
        }
    }

    #[test]
    fn unknown_effect_type_is_rejected() {
        assert!(matches!(
            build(&config("flanger", &[]), 44100),
            Err(AppError::UnknownEffect(_))
        ));
    }

    #[test]
    fn out_of_range_or_unknown_params_are_rejected_instead_of_silently_clamped() {
        assert!(build(&config("echo", &[("feedback", 1.5)]), 44100).is_err());
        assert!(build(&config("delay", &[("speed", 1.0)]), 44100).is_err());
    }

    #[test]
    fn chain_built_from_config_applies_the_configured_delay_time() {
        let mut chain =
            build_chain(&[config("delay", &[("time_ms", 10.0), ("mix", 1.0)])], 1000).unwrap();
        let mut input = vec![0.0; 20];
        input[0] = 1.0;
        let out = chain.render(&input);
        assert_eq!(out[10], 1.0);
    }
}
