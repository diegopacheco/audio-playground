import type { Note } from "../types";
import { noteName } from "../audio/notes";

interface Props {
  recording: boolean;
  notes: Note[];
  wavUrl: string | null;
  busy: boolean;
  onToggleRecord: () => void;
  onExport: () => void;
}

export function PhrasePanel({ recording, notes, wavUrl, busy, onToggleRecord, onExport }: Props) {
  return (
    <section className="panel phrase">
      <h2>Phrase export</h2>
      <div className="row">
        <button className={recording ? "recording" : ""} onClick={onToggleRecord}>
          {recording ? "Stop recording" : "Record phrase"}
        </button>
        <button disabled={recording || busy || notes.length === 0} onClick={onExport}>
          {busy ? "Rendering..." : "Render WAV in Rust"}
        </button>
      </div>
      <p className="notes">
        {notes.length === 0
          ? "Record a phrase, then render it offline with the same synth in the backend."
          : notes.map((n) => `${noteName(n.midi)}@${n.start.toFixed(2)}s`).join("  ")}
      </p>
      {wavUrl && (
        <div className="row">
          <audio controls src={wavUrl} />
          <a href={wavUrl} download="phrase.wav">
            Download phrase.wav
          </a>
        </div>
      )}
    </section>
  );
}
