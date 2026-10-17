import { useCallback } from "react";
import type { PeakPoint } from "../types";
import { scale } from "../audio/plot";
import { useCanvas } from "./useCanvas";

const WIDTH = 560;
const HEIGHT = 220;
const PAD = 28;
const MAX_HZ = 4000;

interface Props {
  peaks: PeakPoint[];
  duration: number;
}

export function ConstellationCanvas({ peaks, duration }: Props) {
  const draw = useCallback(
    (ctx: CanvasRenderingContext2D) => {
      ctx.fillStyle = "#fbfcfe";
      ctx.fillRect(0, 0, WIDTH, HEIGHT);
      ctx.strokeStyle = "#e5e7eb";
      ctx.fillStyle = "#6b7280";
      ctx.font = "10px system-ui";
      for (let hz = 0; hz <= MAX_HZ; hz += 1000) {
        const y = HEIGHT - PAD - scale(hz, 0, MAX_HZ, HEIGHT - 2 * PAD);
        ctx.beginPath();
        ctx.moveTo(PAD, y);
        ctx.lineTo(WIDTH - 8, y);
        ctx.stroke();
        ctx.fillText(`${hz / 1000}k`, 4, y + 3);
      }
      ctx.fillText(`${duration.toFixed(1)}s`, WIDTH - 36, HEIGHT - 8);
      ctx.fillText("0s", PAD, HEIGHT - 8);
      ctx.fillStyle = "#4f46e5";
      for (const p of peaks) {
        const x = PAD + scale(p.t, 0, duration, WIDTH - PAD - 8);
        const y = HEIGHT - PAD - scale(p.hz, 0, MAX_HZ, HEIGHT - 2 * PAD);
        ctx.beginPath();
        ctx.arc(x, y, 2, 0, Math.PI * 2);
        ctx.fill();
      }
    },
    [peaks, duration],
  );
  const ref = useCanvas(WIDTH, HEIGHT, draw);
  return (
    <figure className="plot">
      <figcaption>Constellation map of the query ({peaks.length} peaks)</figcaption>
      <canvas ref={ref} style={{ width: WIDTH, height: HEIGHT }} />
    </figure>
  );
}
