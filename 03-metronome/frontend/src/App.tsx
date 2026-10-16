import { useEffect, useRef, useState } from "react";
import { IntervalMetronome } from "./audio/intervalMetronome";
import { LookaheadMetronome } from "./audio/lookahead";
import { startLoad } from "./audio/mainThreadLoad";
import { ClickTrackPanel } from "./components/ClickTrackPanel";
import { Controls } from "./components/Controls";
import { DriftPanel } from "./components/DriftPanel";
import { JitterChart } from "./components/JitterChart";
import { MetronomePanel } from "./components/MetronomePanel";
import type { BeatSample, Mode } from "./types";

const INTERVAL_COLOR = "#eb6834";
const LOOKAHEAD_COLOR = "#2a78d6";
const HISTORY = 96;

interface Runner {
  start: () => void;
  stop: () => void;
}

export function App() {
  const [bpm, setBpm] = useState(120);
  const [beatsPerBar, setBeatsPerBar] = useState(4);
  const [load, setLoad] = useState(false);
  const [running, setRunning] = useState<Record<Mode, boolean>>({ interval: false, lookahead: false });
  const [samples, setSamples] = useState<Record<Mode, BeatSample[]>>({ interval: [], lookahead: [] });
  const ctxRef = useRef<AudioContext | null>(null);
  const runners = useRef<Record<Mode, Runner | null>>({ interval: null, lookahead: null });

  useEffect(() => (load ? startLoad(100, 90) : undefined), [load]);

  const stopMode = (mode: Mode) => {
    runners.current[mode]?.stop();
    runners.current[mode] = null;
    setRunning((r) => ({ ...r, [mode]: false }));
  };

  const startMode = async (mode: Mode) => {
    const ctx = ctxRef.current ?? new AudioContext();
    ctxRef.current = ctx;
    await ctx.resume();
    setSamples((s) => ({ ...s, [mode]: [] }));
    const onBeat = (sample: BeatSample) =>
      setSamples((s) => ({ ...s, [mode]: [...s[mode], sample].slice(-HISTORY) }));
    const runner =
      mode === "interval"
        ? new IntervalMetronome(ctx, bpm, beatsPerBar, onBeat)
        : new LookaheadMetronome(ctx, bpm, beatsPerBar, onBeat);
    runner.start();
    runners.current[mode] = runner;
    setRunning((r) => ({ ...r, [mode]: true }));
  };

  const toggle = (mode: Mode) => (running[mode] ? stopMode(mode) : void startMode(mode));

  useEffect(() => {
    stopMode("interval");
    stopMode("lookahead");
  }, [bpm, beatsPerBar]);

  useEffect(() => () => {
    runners.current.interval?.stop();
    runners.current.lookahead?.stop();
  }, []);

  return (
    <main>
      <header className="top">
        <h1>Metronome drift</h1>
        <p className="muted">Two browser schedulers side by side, plus a Rust sample-accurate click track and a Rust scheduler drift analysis.</p>
      </header>
      <Controls bpm={bpm} beatsPerBar={beatsPerBar} load={load} onBpm={setBpm} onBeatsPerBar={setBeatsPerBar} onLoad={setLoad} />
      <div className="grid2">
        <MetronomePanel
          title="setInterval"
          description="Plays a click when the timer callback runs. Error is the callback time vs the ideal beat grid, so timer jitter and main-thread stalls land in the audio."
          color={INTERVAL_COLOR}
          running={running.interval}
          beatsPerBar={beatsPerBar}
          samples={samples.interval}
          onToggle={() => toggle("interval")}
        />
        <MetronomePanel
          title="Web Audio lookahead"
          description="A 25 ms timer schedules clicks 100 ms ahead at start + n x interval on AudioContext.currentTime. Error is how late a click was scheduled, so it stays 0 unless a stall exceeds the lookahead."
          color={LOOKAHEAD_COLOR}
          running={running.lookahead}
          beatsPerBar={beatsPerBar}
          samples={samples.lookahead}
          onToggle={() => toggle("lookahead")}
        />
      </div>
      <section className="card">
        <JitterChart
          title="Measured timing error per beat (live)"
          unit="ms"
          series={[
            { name: "setInterval", color: INTERVAL_COLOR, values: samples.interval.map((s) => s.errorMs) },
            { name: "lookahead", color: LOOKAHEAD_COLOR, values: samples.lookahead.map((s) => s.errorMs) },
          ]}
        />
      </section>
      <div className="grid2">
        <ClickTrackPanel bpm={bpm} beatsPerBar={beatsPerBar} />
        <DriftPanel bpm={bpm} naiveColor={INTERVAL_COLOR} absoluteColor={LOOKAHEAD_COLOR} />
      </div>
    </main>
  );
}
