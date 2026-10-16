import { bpmToLag } from "../audio/beats";
import type { Analysis } from "../types";
import { LineChart, type Marker } from "./LineChart";

const MIN_BPM = 60;
const MAX_BPM = 200;

interface EnvelopeProps {
  analysis: Analysis;
  time: number;
}

export function EnvelopeChart({ analysis, time }: EnvelopeProps) {
  const toFrame = (sec: number) => sec * analysis.frame_rate;
  const markers: Marker[] = analysis.beats_sec.map((b) => ({ x: toFrame(b), color: "#f59e0b" }));
  markers.push({ x: toFrame(time), color: "#ef4444", label: `${time.toFixed(2)}s` });
  return (
    <LineChart
      title="Onset envelope"
      subtitle="half-wave rectified onset strength per frame, orange lines are tracked beats"
      values={analysis.onset_envelope}
      xMax={analysis.onset_envelope.length}
      yMin={0}
      yMax={1}
      markers={markers}
      color="#2563eb"
      xLabels={["0s", `${analysis.duration_sec.toFixed(1)}s`]}
    />
  );
}

interface AutocorrProps {
  analysis: Analysis;
}

export function AutocorrChart({ analysis }: AutocorrProps) {
  const { autocorr, frame_rate, lag, bpm } = analysis;
  const markers: Marker[] = [
    { x: bpmToLag(MAX_BPM, frame_rate), color: "#94a3b8", dashed: true },
    { x: bpmToLag(MIN_BPM, frame_rate), color: "#94a3b8", dashed: true },
    { x: lag, color: "#16a34a", label: `lag ${lag.toFixed(1)} frames = ${bpm.toFixed(1)} BPM` },
  ];
  const yMin = Math.min(0, ...autocorr);
  return (
    <LineChart
      title="Autocorrelation of the onset envelope"
      subtitle={`dashed lines bound the ${MIN_BPM}-${MAX_BPM} BPM search range`}
      values={autocorr}
      xMax={autocorr.length}
      yMin={yMin}
      yMax={1}
      markers={markers}
      color="#7c3aed"
      xLabels={["lag 0", `lag ${autocorr.length - 1}`]}
    />
  );
}
