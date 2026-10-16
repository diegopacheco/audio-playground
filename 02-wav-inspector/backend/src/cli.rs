use crate::error::WavError;
use crate::inspect::Report;
use std::fmt;

pub const USAGE: &str =
    "usage:\n  wav-inspector <file.wav> [--width N] [--height N]\n  wav-inspector serve [--port N]";

#[derive(Debug, PartialEq)]
pub enum Command {
    Inspect {
        path: String,
        width: usize,
        height: usize,
    },
    Serve {
        port: u16,
    },
}

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Io(std::io::Error),
    Wav(WavError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(msg) => write!(f, "{msg}\n{USAGE}"),
            CliError::Io(e) => write!(f, "io error: {e}"),
            CliError::Wav(e) => write!(f, "invalid wav: {e}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<WavError> for CliError {
    fn from(e: WavError) -> Self {
        CliError::Wav(e)
    }
}

fn number<T: std::str::FromStr>(flag: &str, value: Option<&String>) -> Result<T, CliError> {
    value
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| CliError::Usage(format!("{flag} needs a positive number")))
}

pub fn parse_args(args: &[String], default_port: u16) -> Result<Command, CliError> {
    let first = args
        .first()
        .ok_or_else(|| CliError::Usage("missing argument".into()))?;
    let mut width = 80;
    let mut height = 16;
    let mut port = default_port;
    let mut i = 1;
    while i < args.len() {
        let flag = args[i].as_str();
        let value = args.get(i + 1);
        match flag {
            "--width" => width = number(flag, value)?,
            "--height" => height = number(flag, value)?,
            "--port" => port = number(flag, value)?,
            other => return Err(CliError::Usage(format!("unknown option {other}"))),
        }
        i += 2;
    }
    if first == "serve" {
        Ok(Command::Serve { port })
    } else {
        Ok(Command::Inspect {
            path: first.clone(),
            width,
            height,
        })
    }
}

fn db(v: f32) -> String {
    if v <= 0.0 {
        "-inf dBFS".into()
    } else {
        format!("{:.2} dBFS", 20.0 * v.log10())
    }
}

pub fn text_report(r: &Report) -> String {
    let h = &r.header;
    let mut out = String::new();
    out.push_str(&format!("encoding     {}\n", h.encoding));
    out.push_str(&format!("sample rate  {} Hz\n", h.sample_rate));
    out.push_str(&format!("channels     {}\n", h.channels));
    out.push_str(&format!("bit depth    {}\n", h.bits_per_sample));
    out.push_str(&format!("byte rate    {}\n", h.byte_rate));
    out.push_str(&format!("block align  {}\n", h.block_align));
    out.push_str(&format!("frames       {}\n", h.frames));
    out.push_str(&format!("duration     {:.3} s\n", r.duration));
    out.push_str(&format!("peak         {:.4} ({})\n", r.peak, db(r.peak)));
    out.push_str(&format!("rms          {:.4} ({})\n", r.rms, db(r.rms)));
    out.push_str("chunks\n");
    for c in &r.chunks {
        out.push_str(&format!(
            "  '{}' offset {} size {}\n",
            c.id, c.offset, c.size
        ));
    }
    out.push_str("waveform\n");
    out.push_str(&r.ascii);
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn file_argument_uses_default_dimensions() {
        assert_eq!(
            parse_args(&args(&["a.wav"]), 1).unwrap(),
            Command::Inspect {
                path: "a.wav".into(),
                width: 80,
                height: 16
            }
        );
    }

    #[test]
    fn width_and_height_flags_are_applied() {
        assert_eq!(
            parse_args(&args(&["a.wav", "--width", "120", "--height", "10"]), 1).unwrap(),
            Command::Inspect {
                path: "a.wav".into(),
                width: 120,
                height: 10
            }
        );
    }

    #[test]
    fn serve_uses_given_port() {
        assert_eq!(
            parse_args(&args(&["serve", "--port", "9000"]), 1).unwrap(),
            Command::Serve { port: 9000 }
        );
    }

    #[test]
    fn bad_input_is_a_usage_error() {
        assert!(matches!(parse_args(&[], 1), Err(CliError::Usage(_))));
        assert!(matches!(
            parse_args(&args(&["a.wav", "--width", "wide"]), 1),
            Err(CliError::Usage(_))
        ));
        assert!(matches!(
            parse_args(&args(&["a.wav", "--color"]), 1),
            Err(CliError::Usage(_))
        ));
    }

    #[test]
    fn decibels_of_full_scale_and_half_scale() {
        assert_eq!(db(1.0), "0.00 dBFS");
        assert_eq!(db(0.5), "-6.02 dBFS");
        assert_eq!(db(0.0), "-inf dBFS");
    }
}
