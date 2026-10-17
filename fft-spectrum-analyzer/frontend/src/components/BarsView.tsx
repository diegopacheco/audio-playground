import { useEffect, useRef } from "react";
import { dbToUnit, heatColor, rgbCss } from "../audio/colormap";
import type { SpectrumFrame } from "../types";

const WIDTH = 900;
const HEIGHT = 260;

interface Props {
  frame: SpectrumFrame | null;
}

export function BarsView({ frame }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const ctx = ref.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, WIDTH, HEIGHT);
    ctx.strokeStyle = "#e6e8ee";
    for (let y = 0; y <= HEIGHT; y += HEIGHT / 5) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(WIDTH, y);
      ctx.stroke();
    }
    if (!frame) return;
    const w = WIDTH / frame.bars.length;
    frame.bars.forEach((db, i) => {
      const unit = dbToUnit(db);
      const h = unit * HEIGHT;
      ctx.fillStyle = rgbCss(heatColor(0.35 + unit * 0.65));
      ctx.fillRect(i * w + 1, HEIGHT - h, w - 2, h);
    });
  }, [frame]);

  return (
    <div className="panel">
      <h2>Spectrum</h2>
      <canvas ref={ref} width={WIDTH} height={HEIGHT} />
      <div className="axis">
        <span>20 Hz</span>
        <span>log frequency</span>
        <span>{frame ? `${Math.round(frame.sample_rate / 2)} Hz` : "Nyquist"}</span>
      </div>
    </div>
  );
}
