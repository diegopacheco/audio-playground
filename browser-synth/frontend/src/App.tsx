import { useCallback, useEffect, useRef, useState } from "react";
import type { Note, Preset } from "./types";
import { Synth } from "./audio/synth";
import { PhraseRecorder } from "./audio/recorder";
import { OCTAVE_DOWN_KEY, OCTAVE_UP_KEY, keyToMidi, shiftOctave } from "./keyboard";
import { deletePreset, listPresets, renderPhrase, savePreset } from "./api";
import { Controls } from "./components/Controls";
import { Piano } from "./components/Piano";
import { PresetPanel } from "./components/PresetPanel";
import { PhrasePanel } from "./components/PhrasePanel";

const INITIAL: Preset = {
  name: "Soft Sine",
  waveform: "sine",
  attack: 0.02,
  decay: 0.2,
  sustain: 0.8,
  release: 0.4,
  cutoff: 8000,
  resonance: 0,
  gain: 0.8,
};

const now = () => performance.now() / 1000;

export default function App() {
  const [preset, setPreset] = useState<Preset>(INITIAL);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [octave, setOctave] = useState(3);
  const [active, setActive] = useState<Set<number>>(new Set());
  const [error, setError] = useState<string | null>(null);
  const [recording, setRecording] = useState(false);
  const [notes, setNotes] = useState<Note[]>([]);
  const [wavUrl, setWavUrl] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const synth = useRef(new Synth(INITIAL));
  const recorder = useRef(new PhraseRecorder());
  const recordingRef = useRef(false);
  const heldKeys = useRef(new Map<string, number>());

  const refresh = useCallback(() => {
    listPresets().then(setPresets).catch((e: Error) => setError(e.message));
  }, []);

  useEffect(refresh, [refresh]);

  useEffect(() => synth.current.setPreset(preset), [preset]);

  const noteOn = useCallback((midi: number) => {
    synth.current.noteOn(midi);
    if (recordingRef.current) recorder.current.noteOn(midi, now());
    setActive((s) => new Set(s).add(midi));
  }, []);

  const noteOff = useCallback((midi: number) => {
    synth.current.noteOff(midi);
    if (recordingRef.current) recorder.current.noteOff(midi, now());
    setActive((s) => {
      const next = new Set(s);
      next.delete(midi);
      return next;
    });
  }, []);

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.repeat || e.metaKey || e.ctrlKey || e.target instanceof HTMLInputElement) return;
      if (e.key === OCTAVE_DOWN_KEY || e.key === OCTAVE_UP_KEY) {
        setOctave((o) => shiftOctave(o, e.key === OCTAVE_UP_KEY ? 1 : -1));
        return;
      }
      const midi = keyToMidi(e.key, octave);
      if (midi === null || heldKeys.current.has(e.key)) return;
      heldKeys.current.set(e.key, midi);
      noteOn(midi);
    };
    const up = (e: KeyboardEvent) => {
      const midi = heldKeys.current.get(e.key);
      if (midi === undefined) return;
      heldKeys.current.delete(e.key);
      noteOff(midi);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, [octave, noteOn, noteOff]);

  const toggleRecord = () => {
    if (recordingRef.current) {
      setNotes(recorder.current.finish(now()));
    } else {
      recorder.current.start(now());
      setNotes([]);
    }
    recordingRef.current = !recordingRef.current;
    setRecording(recordingRef.current);
  };

  const exportPhrase = async () => {
    setBusy(true);
    setError(null);
    try {
      const blob = await renderPhrase(preset, notes);
      if (wavUrl) URL.revokeObjectURL(wavUrl);
      setWavUrl(URL.createObjectURL(blob));
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const save = async (p: Preset) => {
    setError(null);
    try {
      setPreset(await savePreset(p));
      refresh();
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const remove = async (name: string) => {
    setError(null);
    try {
      await deletePreset(name);
      refresh();
    } catch (e) {
      setError((e as Error).message);
    }
  };

  return (
    <main>
      <header>
        <h1>Browser Synth</h1>
        <p>Web Audio voices in the browser, offline WAV rendering in Rust.</p>
      </header>
      {error && (
        <div className="error" role="alert">
          {error}
        </div>
      )}
      <div className="layout">
        <Controls preset={preset} onChange={setPreset} />
        <PresetPanel presets={presets} current={preset} onLoad={setPreset} onSave={save} onDelete={remove} />
      </div>
      <section className="panel keyboard">
        <div className="octave">
          <button onClick={() => setOctave((o) => shiftOctave(o, -1))}>Octave -</button>
          <span>Octave {octave}</span>
          <button onClick={() => setOctave((o) => shiftOctave(o, 1))}>Octave +</button>
          <small>
            Play with Z-M and Q-U rows, {OCTAVE_DOWN_KEY} and {OCTAVE_UP_KEY} shift octave
          </small>
        </div>
        <Piano octave={octave} active={active} onNoteOn={noteOn} onNoteOff={noteOff} />
      </section>
      <PhrasePanel
        recording={recording}
        notes={notes}
        wavUrl={wavUrl}
        busy={busy}
        onToggleRecord={toggleRecord}
        onExport={exportPhrase}
      />
    </main>
  );
}
