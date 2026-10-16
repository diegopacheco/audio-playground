use super::Renderer;
use crate::compose::Song;
use crate::error::AppError;

pub const TICKS_PER_QUARTER: u16 = 480;
const MAX_VLQ: u32 = 0x0FFF_FFFF;
const MELODY_CHANNEL: u8 = 0;
const CHORD_CHANNEL: u8 = 1;
const PIANO: u8 = 0;
const STRINGS: u8 = 48;
const CHORD_VELOCITY: u8 = 64;

pub struct MidiRenderer;

pub fn vlq(value: u32) -> Result<Vec<u8>, AppError> {
    if value > MAX_VLQ {
        return Err(AppError::InvalidRequest(format!(
            "delta time too large for VLQ: {value}"
        )));
    }
    let mut bytes = vec![(value & 0x7F) as u8];
    let mut rest = value >> 7;
    while rest > 0 {
        bytes.push(((rest & 0x7F) as u8) | 0x80);
        rest >>= 7;
    }
    bytes.reverse();
    Ok(bytes)
}

struct Event {
    tick: u32,
    order: u8,
    data: Vec<u8>,
}

fn ticks(beats: f64) -> u32 {
    (beats * TICKS_PER_QUARTER as f64).round().max(0.0) as u32
}

fn meta(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut data = vec![0xFF, kind, payload.len() as u8];
    data.extend_from_slice(payload);
    data
}

fn track_chunk(mut events: Vec<Event>) -> Result<Vec<u8>, AppError> {
    events.sort_by_key(|e| (e.tick, e.order));
    let mut body = Vec::new();
    let mut last = 0;
    for event in &events {
        body.extend(vlq(event.tick - last)?);
        body.extend_from_slice(&event.data);
        last = event.tick;
    }
    body.extend(vlq(0)?);
    body.extend(meta(0x2F, &[]));
    let mut chunk = b"MTrk".to_vec();
    chunk.extend((body.len() as u32).to_be_bytes());
    chunk.extend(body);
    Ok(chunk)
}

fn header(tracks: u16) -> Vec<u8> {
    let mut chunk = b"MThd".to_vec();
    chunk.extend(6u32.to_be_bytes());
    chunk.extend(1u16.to_be_bytes());
    chunk.extend(tracks.to_be_bytes());
    chunk.extend(TICKS_PER_QUARTER.to_be_bytes());
    chunk
}

fn note_events(channel: u8, pitch: u8, velocity: u8, start: f64, duration: f64) -> [Event; 2] {
    [
        Event {
            tick: ticks(start),
            order: 1,
            data: vec![0x90 | channel, pitch, velocity],
        },
        Event {
            tick: ticks(start + duration),
            order: 0,
            data: vec![0x80 | channel, pitch, 0],
        },
    ]
}

fn named(name: &str, channel: u8, program: u8) -> Vec<Event> {
    vec![
        Event {
            tick: 0,
            order: 0,
            data: meta(0x03, name.as_bytes()),
        },
        Event {
            tick: 0,
            order: 0,
            data: vec![0xC0 | channel, program],
        },
    ]
}

fn conductor(bpm: f64) -> Vec<Event> {
    let micros = (60_000_000.0 / bpm).round() as u32;
    let tempo = micros.to_be_bytes();
    vec![
        Event {
            tick: 0,
            order: 0,
            data: meta(0x58, &[4, 2, 24, 8]),
        },
        Event {
            tick: 0,
            order: 0,
            data: meta(0x51, &tempo[1..]),
        },
    ]
}

impl Renderer for MidiRenderer {
    fn content_type(&self) -> &'static str {
        "audio/midi"
    }

    fn render(&self, song: &Song) -> Result<Vec<u8>, AppError> {
        let mut melody = named("Melody", MELODY_CHANNEL, PIANO);
        for note in &song.notes {
            melody.extend(note_events(
                MELODY_CHANNEL,
                note.pitch,
                note.velocity,
                note.start,
                note.duration,
            ));
        }
        let mut chords = named("Chords", CHORD_CHANNEL, STRINGS);
        for chord in &song.chords {
            for pitch in &chord.pitches {
                let pitch = (*pitch).clamp(0, 127) as u8;
                chords.extend(note_events(
                    CHORD_CHANNEL,
                    pitch,
                    CHORD_VELOCITY,
                    chord.start,
                    chord.duration,
                ));
            }
        }
        let mut file = header(3);
        file.extend(track_chunk(conductor(song.bpm))?);
        file.extend(track_chunk(melody)?);
        file.extend(track_chunk(chords)?);
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::{GenerateRequest, compose};

    fn song() -> Song {
        compose(&GenerateRequest {
            key: "C".into(),
            scale: "major".into(),
            progression: "I-IV-V-I".into(),
            bars: 4,
            bpm: 120.0,
            generator: "rule".into(),
            seed: 9,
            order: 1,
        })
        .unwrap()
    }

    fn chunks(bytes: &[u8]) -> Vec<(&[u8], &[u8])> {
        let mut out = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            let len =
                u32::from_be_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]])
                    as usize;
            out.push((&bytes[at..at + 4], &bytes[at + 8..at + 8 + len]));
            at += 8 + len;
        }
        out
    }

    #[test]
    fn vlq_matches_the_smf_specification_boundaries() {
        assert_eq!(vlq(0).unwrap(), vec![0x00]);
        assert_eq!(vlq(0x7F).unwrap(), vec![0x7F]);
        assert_eq!(vlq(0x80).unwrap(), vec![0x81, 0x00]);
        assert_eq!(vlq(0x2000).unwrap(), vec![0xC0, 0x00]);
        assert_eq!(vlq(0x3FFF).unwrap(), vec![0xFF, 0x7F]);
        assert_eq!(vlq(0x4000).unwrap(), vec![0x81, 0x80, 0x00]);
        assert_eq!(vlq(0x200000).unwrap(), vec![0x81, 0x80, 0x80, 0x00]);
        assert_eq!(vlq(0x0FFF_FFFF).unwrap(), vec![0xFF, 0xFF, 0xFF, 0x7F]);
        assert!(vlq(0x1000_0000).is_err());
    }

    #[test]
    fn header_declares_type_one_with_three_tracks_at_480_ppq() {
        let bytes = MidiRenderer.render(&song()).unwrap();
        assert_eq!(
            &bytes[..14],
            &[b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 1, 0, 3, 0x01, 0xE0]
        );
    }

    #[test]
    fn track_chunks_have_correct_lengths_and_end_of_track() {
        let bytes = MidiRenderer.render(&song()).unwrap();
        let parsed = chunks(&bytes[14..]);
        assert_eq!(parsed.len(), 3);
        for (id, body) in &parsed {
            assert_eq!(*id, b"MTrk");
            assert_eq!(&body[body.len() - 4..], &[0x00, 0xFF, 0x2F, 0x00]);
        }
    }

    #[test]
    fn conductor_track_carries_tempo_for_120_bpm() {
        let bytes = MidiRenderer.render(&song()).unwrap();
        let parsed = chunks(&bytes[14..]);
        let tempo = [0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20];
        assert!(parsed[0].1.windows(6).any(|w| w == tempo));
    }

    fn read_vlq(body: &[u8], at: &mut usize) -> u32 {
        let mut value = 0u32;
        loop {
            let byte = body[*at];
            *at += 1;
            value = (value << 7) | (byte & 0x7F) as u32;
            if byte & 0x80 == 0 {
                return value;
            }
        }
    }

    fn events(body: &[u8]) -> Vec<(u32, u8, u8)> {
        let mut out = Vec::new();
        let mut at = 0;
        let mut tick = 0;
        while at < body.len() {
            tick += read_vlq(body, &mut at);
            let status = body[at];
            if status == 0xFF {
                let len = body[at + 2] as usize;
                at += 3 + len;
            } else if status & 0xF0 == 0xC0 {
                at += 2;
            } else {
                out.push((tick, status, body[at + 1]));
                at += 3;
            }
        }
        out
    }

    #[test]
    fn note_events_decode_back_to_the_song_timing() {
        let s = song();
        let bytes = MidiRenderer.render(&s).unwrap();
        let parsed = chunks(&bytes[14..]);
        let melody = events(parsed[1].1);
        let ons: Vec<(u32, u8)> = melody
            .iter()
            .filter(|e| e.1 == 0x90)
            .map(|e| (e.0, e.2))
            .collect();
        let expected: Vec<(u32, u8)> = s.notes.iter().map(|n| (ticks(n.start), n.pitch)).collect();
        assert_eq!(ons, expected);
        assert_eq!(melody.iter().filter(|e| e.1 == 0x80).count(), s.notes.len());
        let last_off = melody
            .iter()
            .filter(|e| e.1 == 0x80)
            .map(|e| e.0)
            .max()
            .unwrap();
        assert_eq!(last_off, 4 * 4 * TICKS_PER_QUARTER as u32);
        let chords = events(parsed[2].1);
        assert_eq!(
            chords.iter().filter(|e| e.1 == 0x91).count(),
            s.chords.len() * 3
        );
    }
}
