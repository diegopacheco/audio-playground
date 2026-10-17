import { formatTimestamp } from "../audio/format";
import type { Transcript } from "../types";

interface Props {
  transcript: Transcript | null;
  source: string;
}

export function TranscriptView({ transcript, source }: Props) {
  return (
    <section className="card">
      <h2>Transcript</h2>
      {transcript === null ? (
        <p className="muted">Record or upload audio to see the transcript.</p>
      ) : (
        <>
          <p className="muted">
            {source} - {transcript.duration.toFixed(2)} s - {transcript.segments.length} segments
          </p>
          <ul className="segments">
            {transcript.segments.map((s, i) => (
              <li key={i}>
                <span className="time">
                  [{formatTimestamp(s.start)} - {formatTimestamp(s.end)}]
                </span>
                <span>{s.text}</span>
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}
