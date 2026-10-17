import type { SourceKind } from "../types";

interface Props {
  source: SourceKind;
  busy: boolean;
  onStart: (kind: "mic" | "test") => void;
  onStop: () => void;
}

export function SourceControls({ source, busy, onStart, onStop }: Props) {
  return (
    <section className="card">
      <h2>Live input</h2>
      <div className="buttons">
        <button className={source === "mic" ? "primary active" : "primary"} disabled={busy} onClick={() => onStart("mic")}>
          Microphone
        </button>
        <button className={source === "test" ? "primary active" : "primary"} disabled={busy} onClick={() => onStart("test")}>
          Test voice
        </button>
        <button disabled={source === "off"} onClick={onStop}>
          Stop
        </button>
      </div>
      <p className="hint">
        {source === "off" && "Pick a source. Use headphones with the microphone to avoid feedback."}
        {source === "mic" && "Streaming the microphone through the backend."}
        {source === "test" && "Streaming the synthetic glottal-pulse voice through the backend."}
      </p>
    </section>
  );
}
