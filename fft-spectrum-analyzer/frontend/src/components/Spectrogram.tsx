import { useEffect, useRef } from "react";
import { dbToUnit, heatColor, rgbCss } from "../audio/colormap";
import type { SpectrumFrame } from "../types";

const WIDTH = 900;
const HEIGHT = 220;
const COLUMN = 2;

interface Props {
  frame: SpectrumFrame | null;
}

export function Spectrogram({ frame }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx || !frame) return;
    ctx.drawImage(canvas, -COLUMN, 0);
    const h = HEIGHT / frame.bars.length;
    frame.bars.forEach((db, i) => {
      ctx.fillStyle = rgbCss(heatColor(dbToUnit(db)));
      ctx.fillRect(WIDTH - COLUMN, HEIGHT - (i + 1) * h, COLUMN, Math.ceil(h));
    });
  }, [frame]);

  return (
    <div className="panel">
      <h2>Spectrogram</h2>
      <canvas ref={ref} width={WIDTH} height={HEIGHT} className="spectrogram" />
      <div className="axis">
        <span>older</span>
        <span>time</span>
        <span>now</span>
      </div>
    </div>
  );
}
