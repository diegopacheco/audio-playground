# audio-playground

Hands-on POCs of audio and DSP. Every project has a Rust backend (edition 2024), a React 19 + TypeScript + Vite UI run with bun, its own README and a `scripts/` folder to set up, start, test and stop it.

## 🎛️ Synthesis & Sound Generation

Sound starts as numbers. These POCs make waves, envelopes, drums and melodies from code.
No samples, no audio libraries.

* [tone-generator](tone-generator/) - Sine, square, saw and triangle waves written to WAV with a hand-made RIFF header
* [browser-synth](browser-synth/) - Polyphonic Web Audio synth with ADSR and lowpass filter, presets and offline render in Rust
* [drum-machine](drum-machine/) - step sequencer with synthesized kick, snare, hi-hat and clap, exported to WAV
* [generative-music](generative-music/) - Markov and rule-based melodies over Roman numeral progressions, exported as MIDI and WAV

## 📼 Files & Formats

Before you can process audio you have to read and write it correctly.
These POCs open the WAV container byte by byte.

* [wav-inspector](wav-inspector/) - RIFF chunk parser CLI and API with header table, peak/RMS and ASCII waveform

## ⏱️ Timing

Audio is unforgiving about time. A few milliseconds of drift is audible.

* [metronome](metronome/) - Sample-accurate click track and setInterval vs Web Audio lookahead drift, measured
* [low-latency-rust](low-latency-rust/) - cpal engine with a lock-free SPSC ring buffer and round-trip latency probes

## 📊 Analysis

Analysis turns raw samples into something we understand: frequencies, notes, tempo.

* [fft-spectrum-analyzer](fft-spectrum-analyzer/) - Hand-written FFT over WebSocket with live bars and scrolling spectrogram
* [pitch-tuner](pitch-tuner/) - YIN and autocorrelation pitch detection with note, octave and cents gauge
* [bpm-detection](bpm-detection/) - Spectral flux onsets, autocorrelation tempo and beat tracking
* [audio-fingerprinting](audio-fingerprinting/) - Mini Shazam with constellation peaks, pair hashes and offset histogram matching

## 🎚️ Effects & Transformation

Effects change how a sound feels without changing what it is.
Some work on files, some in real time.

* [effects-chain](effects-chain/) - Delay, echo, Schroeder reverb, distortion and bitcrusher in a reorderable chain
* [voice-changer](voice-changer/) - Phase vocoder pitch shift, chipmunk, deep and robot voices streamed over WebSocket
* [spatial-audio](spatial-audio/) - Binaural ITD/ILD and parametric HRTF with circle, figure-8 and Doppler flyby paths

## 📡 Hidden Data & Transmission

Sound can carry more than music.
These POCs hide bytes inside audio and send them through the air.

* [audio-steganography](audio-steganography/) - Hide text in sample LSBs or near-ultrasonic FSK, with CRC framing
* [data-over-sound](data-over-sound/) - Bell 202 style BFSK and 4-FSK acoustic modem with Goertzel decoding

## 🧠 Speech & AI

Speech turns audio into language, and an LLM turns language into decisions.

* [speech-to-text-summary](speech-to-text-summary/) - Local whisper.cpp transcription with Claude summary and action items

## Ports

Each POC has its own ports in `scripts/ports.env`, so all of them can run at the same time.

| POC | Backend | Frontend |
|---|---|---|
| [audio-fingerprinting](audio-fingerprinting/) | 8112 | 5112 |
| [audio-steganography](audio-steganography/) | 8110 | 5110 |
| [bpm-detection](bpm-detection/) | 8109 | 5109 |
| [browser-synth](browser-synth/) | 8104 | 5104 |
| [data-over-sound](data-over-sound/) | 8111 | 5111 |
| [drum-machine](drum-machine/) | 8108 | 5108 |
| [effects-chain](effects-chain/) | 8107 | 5107 |
| [fft-spectrum-analyzer](fft-spectrum-analyzer/) | 8105 | 5105 |
| [generative-music](generative-music/) | 8117 | 5117 |
| [low-latency-rust](low-latency-rust/) | 8115 | 5115 |
| [metronome](metronome/) | 8103 | 5103 |
| [pitch-tuner](pitch-tuner/) | 8106 | 5106 |
| [spatial-audio](spatial-audio/) | 8116 | 5116 |
| [speech-to-text-summary](speech-to-text-summary/) | 8114 | 5114 |
| [tone-generator](tone-generator/) | 8101 | 5101 |
| [voice-changer](voice-changer/) | 8113 | 5113 |
| [wav-inspector](wav-inspector/) | 8102 | 5102 |

## Run any POC

```bash
cd tone-generator
./scripts/setup.sh
./scripts/start-all.sh
./scripts/ui.sh
./scripts/stop-all.sh
```
