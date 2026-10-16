import { useEffect, useRef, useState } from "react";
import { listPatterns, loadPattern, renderWav, savePattern } from "./api";
import { Sequencer } from "./audio/sequencer";
import { PatternBar } from "./components/PatternBar";
import { StepGrid } from "./components/StepGrid";
import { Transport } from "./components/Transport";
import { emptyPattern, toggleStep } from "./pattern";
import type { InstrumentName, PlaybackSettings } from "./types";

export function App() {
  const [pattern, setPattern] = useState(emptyPattern);
  const [bpm, setBpm] = useState(120);
  const [swing, setSwing] = useState(0);
  const [bars, setBars] = useState(2);
  const [names, setNames] = useState<string[]>([]);
  const [playing, setPlaying] = useState(false);
  const [currentStep, setCurrentStep] = useState(-1);
  const [status, setStatus] = useState("");
  const live = useRef<PlaybackSettings>({ pattern, bpm, swing });
  live.current = { pattern, bpm, swing };
  const sequencer = useRef<Sequencer | null>(null);
  sequencer.current ??= new Sequencer(() => live.current, setCurrentStep);

  const run = async (task: () => Promise<string>) => {
    try {
      setStatus(await task());
    } catch (e) {
      setStatus(e instanceof Error ? e.message : String(e));
    }
  };

  const load = (name: string) =>
    run(async () => {
      setPattern(await loadPattern(name));
      return `loaded ${name}`;
    });

  useEffect(() => {
    run(async () => {
      const list = await listPatterns();
      setNames(list);
      if (list[0]) setPattern(await loadPattern(list[0]));
      return list[0] ? `loaded ${list[0]}` : "";
    });
    return () => sequencer.current?.stop();
  }, []);

  const save = (name: string) =>
    run(async () => {
      await savePattern(name, pattern);
      setNames(await listPatterns());
      return `saved ${name}`;
    });

  const exportWav = () =>
    run(async () => {
      const blob = await renderWav({ pattern, bpm, bars, swing });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `drums-${bpm}bpm.wav`;
      a.click();
      URL.revokeObjectURL(url);
      return `exported ${Math.round(blob.size / 1024)} KB`;
    });

  const play = async () => {
    await sequencer.current?.start();
    setPlaying(true);
  };

  const stop = () => {
    sequencer.current?.stop();
    setPlaying(false);
  };

  return (
    <main>
      <header>
        <h1>Drum Machine</h1>
        <p>Synthesized kick, snare, hihat and clap. No samples.</p>
      </header>
      <Transport
        playing={playing}
        bpm={bpm}
        swing={swing}
        onPlay={play}
        onStop={stop}
        onBpm={setBpm}
        onSwing={setSwing}
      />
      <StepGrid
        pattern={pattern}
        currentStep={currentStep}
        onToggle={(name: InstrumentName, step: number) => setPattern((p) => toggleStep(p, name, step))}
      />
      <PatternBar
        names={names}
        bars={bars}
        onLoad={load}
        onSave={save}
        onBars={setBars}
        onExport={exportWav}
        onClear={() => setPattern(emptyPattern())}
      />
      <p className="status">{status}</p>
    </main>
  );
}
