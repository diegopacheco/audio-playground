import type { Tone } from "../types";

interface Props {
  tone: Tone;
}

export function Player({ tone }: Props) {
  const { waveform, frequency, duration_ms } = tone.request;
  const name = `${waveform}-${frequency}hz-${duration_ms}ms.wav`;
  return (
    <div className="card player">
      <audio controls src={tone.url} autoPlay />
      <a className="primary" href={tone.url} download={name}>
        Download {name}
      </a>
    </div>
  );
}
