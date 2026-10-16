import { useEffect, useRef } from "react";
import { computePeaks } from "../audio/waveform";

interface Props {
  title: string;
  buffer: AudioBuffer | null;
  color: string;
  active: boolean;
}

const WIDTH = 900;
const HEIGHT = 140;

export function Waveform({ title, buffer, color, active }: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const ctx = canvas.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, WIDTH, HEIGHT);
    ctx.strokeStyle = "#d8dce3";
    ctx.beginPath();
    ctx.moveTo(0, HEIGHT / 2);
    ctx.lineTo(WIDTH, HEIGHT / 2);
    ctx.stroke();
    if (!buffer) return;
    ctx.fillStyle = color;
    const mid = HEIGHT / 2;
    computePeaks(buffer.getChannelData(0), WIDTH).forEach((peak, x) => {
      const top = mid - peak.max * mid;
      const height = Math.max(1, (peak.max - peak.min) * mid);
      ctx.fillRect(x, top, 1, height);
    });
  }, [buffer, color]);

  return (
    <div className={active ? "wave active" : "wave"}>
      <div className="wave-head">
        <span>{title}</span>
        <span className="muted">{buffer ? `${buffer.duration.toFixed(2)} s` : "empty"}</span>
      </div>
      <canvas ref={canvas} width={WIDTH} height={HEIGHT} />
    </div>
  );
}
