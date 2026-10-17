import { useCallback } from "react";
import type { HistogramBin } from "../types";
import { scale, toColumns } from "../audio/plot";
import { useCanvas } from "./useCanvas";

const WIDTH = 560;
const HEIGHT = 180;
const PAD = 24;

interface Props {
  bins: HistogramBin[];
  winner: string;
  offset: number;
}

export function HistogramCanvas({ bins, winner, offset }: Props) {
  const draw = useCallback(
    (ctx: CanvasRenderingContext2D) => {
      ctx.fillStyle = "#fbfcfe";
      ctx.fillRect(0, 0, WIDTH, HEIGHT);
      const plotWidth = WIDTH - 2 * PAD;
      const columns = toColumns(bins, plotWidth);
      const top = Math.max(1, ...columns.map((c) => c.count));
      columns.forEach((c, i) => {
        if (c.count === 0) {
          return;
        }
        const h = scale(c.count, 0, top, HEIGHT - 2 * PAD);
        ctx.fillStyle = c.count === top ? "#16a34a" : "#94a3b8";
        ctx.fillRect(PAD + i, HEIGHT - PAD - h, 1.5, h);
      });
      ctx.fillStyle = "#6b7280";
      ctx.font = "10px system-ui";
      if (bins.length > 0) {
        ctx.fillText(`${bins[0].offset_sec.toFixed(1)}s`, PAD, HEIGHT - 8);
        const last = `${bins[bins.length - 1].offset_sec.toFixed(1)}s`;
        ctx.fillText(last, WIDTH - PAD - 28, HEIGHT - 8);
        ctx.fillText(`peak ${top} at ${offset.toFixed(2)}s`, PAD, 14);
      } else {
        ctx.fillText("no shared hashes", PAD, HEIGHT / 2);
      }
    },
    [bins, offset],
  );
  const ref = useCanvas(WIDTH, HEIGHT, draw);
  return (
    <figure className="plot">
      <figcaption>Time offset histogram for {winner}</figcaption>
      <canvas ref={ref} style={{ width: WIDTH, height: HEIGHT }} />
    </figure>
  );
}
