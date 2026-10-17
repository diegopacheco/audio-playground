import { useEffect, useRef } from "react";
import { columns } from "../audio/diff";

interface Props {
  diff: Int16Array;
  maxDiff: number;
  caption: string;
}

const WIDTH = 760;
const HEIGHT = 160;

export function DiffCanvas({ diff, maxDiff, caption }: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const ctx = canvas.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, WIDTH, HEIGHT);
    ctx.fillStyle = "#fbfcfe";
    ctx.fillRect(0, 0, WIDTH, HEIGHT);
    const mid = HEIGHT / 2;
    ctx.strokeStyle = "#d5dce6";
    ctx.beginPath();
    ctx.moveTo(0, mid);
    ctx.lineTo(WIDTH, mid);
    ctx.stroke();
    const scale = (mid - 6) / Math.max(1, maxDiff);
    ctx.strokeStyle = "#7c5cff";
    ctx.beginPath();
    columns(diff, WIDTH).forEach((c, x) => {
      ctx.moveTo(x + 0.5, mid - c.max * scale);
      ctx.lineTo(x + 0.5, mid - c.min * scale + 1);
    });
    ctx.stroke();
  }, [diff, maxDiff]);

  return (
    <div>
      <canvas ref={canvas} width={WIDTH} height={HEIGHT} className="diff" />
      <p className="muted">
        {caption}, vertical scale ±{maxDiff} sample units
      </p>
    </div>
  );
}
