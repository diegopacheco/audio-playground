# audio-playground

Hands-on POCs of audio and DSP. Every project has a Rust backend (edition 2024), a React 19 + TypeScript + Vite UI run with bun, its own README and a `scripts/` folder to set up, start, test and stop it.

## 🎛️ Synthesis & Sound Generation

Sound starts as numbers. These POCs make waves, envelopes, drums and melodies from code.
No samples, no audio libraries.

* [01-tone-generator](01-tone-generator/) - Sine, square, saw and triangle waves written to WAV with a hand-made RIFF header
* [04-browser-synth](04-browser-synth/) - Polyphonic Web Audio synth with ADSR and lowpass filter, presets and offline render in Rust
* [08-drum-machine](08-drum-machine/) - 16-step sequencer with synthesized kick, snare, hi-hat and clap, exported to WAV
* [17-generative-music](17-generative-music/) - Markov and rule-based melodies over Roman numeral progressions, exported as MIDI and WAV

## 📼 Files & Formats

Before you can process audio you have to read and write it correctly.
These POCs open the WAV container byte by byte.

* [02-wav-inspector](02-wav-inspector/) - RIFF chunk parser CLI and API with header table, peak/RMS and ASCII waveform

## ⏱️ Timing

Audio is unforgiving about time. A few milliseconds of drift is audible.

* [03-metronome](03-metronome/) - Sample-accurate click track and setInterval vs Web Audio lookahead drift, measured
* [15-low-latency-rust](15-low-latency-rust/) - cpal engine with a lock-free SPSC ring buffer and round-trip latency probes

## 📊 Analysis

Analysis turns raw samples into something we understand: frequencies, notes, tempo.

* [05-fft-spectrum-analyzer](05-fft-spectrum-analyzer/) - Hand-written FFT over WebSocket with live bars and scrolling spectrogram
* [06-pitch-tuner](06-pitch-tuner/) - YIN and autocorrelation pitch detection with note, octave and cents gauge
* [09-bpm-detection](09-bpm-detection/) - Spectral flux onsets, autocorrelation tempo and beat tracking
* [12-audio-fingerprinting](12-audio-fingerprinting/) - Mini Shazam with constellation peaks, pair hashes and offset histogram matching

## 🎚️ Effects & Transformation

Effects change how a sound feels without changing what it is.
Some work on files, some in real time.

* [07-effects-chain](07-effects-chain/) - Delay, echo, Schroeder reverb, distortion and bitcrusher in a reorderable chain
* [13-voice-changer](13-voice-changer/) - Phase vocoder pitch shift, chipmunk, deep and robot voices streamed over WebSocket
* [16-spatial-audio](16-spatial-audio/) - Binaural ITD/ILD and parametric HRTF with circle, figure-8 and Doppler flyby paths

## 📡 Hidden Data & Transmission

Sound can carry more than music.
These POCs hide bytes inside audio and send them through the air.

* [10-audio-steganography](10-audio-steganography/) - Hide text in sample LSBs or near-ultrasonic FSK, with CRC framing
* [11-data-over-sound](11-data-over-sound/) - Bell 202 style BFSK and 4-FSK acoustic modem with Goertzel decoding

## 🧠 Speech & AI

Speech turns audio into language, and an LLM turns language into decisions.

* [14-speech-to-text-summary](14-speech-to-text-summary/) - Local whisper.cpp transcription with Claude summary and action items

## Ports

Each POC has its own ports in `scripts/ports.env`, so all of them can run at the same time.

| POC | Backend | Frontend |
|---|---|---|
| 01 - 17 | 8101 - 8117 | 5101 - 5117 |

## Run any POC

```bash
cd 01-tone-generator
./scripts/setup.sh
./scripts/start-all.sh
./scripts/ui.sh
./scripts/stop-all.sh
```
