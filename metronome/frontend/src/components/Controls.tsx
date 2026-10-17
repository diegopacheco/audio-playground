import { useState } from "react";
import { addTap, bpmFromTaps, MAX_BPM, MIN_BPM } from "../audio/tapTempo";

interface Props {
  bpm: number;
  beatsPerBar: number;
  load: boolean;
  onBpm: (bpm: number) => void;
  onBeatsPerBar: (beats: number) => void;
  onLoad: (load: boolean) => void;
}

export function Controls({ bpm, beatsPerBar, load, onBpm, onBeatsPerBar, onLoad }: Props) {
  const [taps, setTaps] = useState<number[]>([]);

  const tap = () => {
    const next = addTap(taps, performance.now());
    setTaps(next);
    const tapped = bpmFromTaps(next);
    if (tapped !== null) onBpm(tapped);
  };

  return (
    <section className="card controls">
      <label>
        BPM <strong>{bpm}</strong>
        <input type="range" min={MIN_BPM} max={MAX_BPM} step={1} value={bpm} onChange={(e) => onBpm(Number(e.target.value))} />
      </label>
      <label>
        Beats per bar
        <input type="number" min={1} max={16} value={beatsPerBar} onChange={(e) => onBeatsPerBar(Number(e.target.value))} />
      </label>
      <button type="button" onClick={tap}>
        Tap tempo {taps.length > 1 ? `(${taps.length})` : ""}
      </button>
      <label className="check">
        <input type="checkbox" checked={load} onChange={(e) => onLoad(e.target.checked)} />
        Busy main thread (random 0-90 ms blocks)
      </label>
    </section>
  );
}
