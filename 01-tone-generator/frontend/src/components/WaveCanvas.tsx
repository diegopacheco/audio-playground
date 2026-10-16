import { useEffect, useRef } from "react";
import { toPoints, visibleSampleCount } from "../audio/samples";

interface Props {
  samples: Int16Array;
  sampleRate: number;
  frequency: number;
}

const WIDTH = 720;
const HEIGHT = 220;
const PERIODS = 4;

export function WaveCanvas({ samples, sampleRate, frequency }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);
  const count = visibleSampleCount(sampleRate, frequency, PERIODS, samples.length);

  useEffect(() => {
    const ctx = ref.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, WIDTH, HEIGHT);
    ctx.strokeStyle = "#d8dee9";
    ctx.beginPath();
    ctx.moveTo(0, HEIGHT / 2);
    ctx.lineTo(WIDTH, HEIGHT / 2);
    ctx.stroke();
    ctx.strokeStyle = "#3b6fd8";
    ctx.lineWidth = 2;
    ctx.beginPath();
    toPoints(samples, count, WIDTH, HEIGHT).forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
    ctx.stroke();
    ctx.lineWidth = 1;
  }, [samples, count]);

  return (
    <div className="card">
      <h2>Waveform</h2>
      <canvas ref={ref} width={WIDTH} height={HEIGHT} className="wave" />
      <p className="muted">
        First {count} of {samples.length} samples ({PERIODS} periods at {frequency} Hz)
      </p>
    </div>
  );
}
