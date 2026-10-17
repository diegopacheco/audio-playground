import type { Side } from "../types";
import { Waveform } from "./Waveform";

interface Props {
  original: AudioBuffer | null;
  processed: AudioBuffer | null;
  playing: Side | null;
  busy: boolean;
  canProcess: boolean;
  onProcess: () => void;
  onPlay: (side: Side) => void;
  onStop: () => void;
}

export function Monitor({ original, processed, playing, busy, canProcess, onProcess, onPlay, onStop }: Props) {
  return (
    <section className="panel">
      <h2>A/B</h2>
      <div className="row">
        <button className="primary" disabled={!canProcess || busy} onClick={onProcess}>
          {busy ? "Processing..." : "Process chain"}
        </button>
        <button disabled={!original} className={playing === "A" ? "on" : ""} onClick={() => onPlay("A")}>
          A: Original
        </button>
        <button disabled={!processed} className={playing === "B" ? "on" : ""} onClick={() => onPlay("B")}>
          B: Processed
        </button>
        <button disabled={!playing} onClick={onStop}>
          Stop
        </button>
      </div>
      <Waveform title="A: Original" buffer={original} color="#5b8def" active={playing === "A"} />
      <Waveform title="B: Processed" buffer={processed} color="#e0794c" active={playing === "B"} />
    </section>
  );
}
