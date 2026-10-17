import { useEffect, useMemo, useRef } from "react";
import { spectrogram } from "../audio/spectrogram";
import type { PcmAudio } from "../types";

interface Props {
  audio: PcmAudio;
}

function color(v: number): [number, number, number] {
  const t = Math.min(1, Math.max(0, v));
  return [Math.round(255 - 200 * t), Math.round(255 - 160 * t * t), Math.round(255 - 60 * t)];
}

export function SpectrogramView({ audio }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);
  const spec = useMemo(() => spectrogram(audio), [audio]);

  useEffect(() => {
    const ctx = ref.current?.getContext("2d");
    if (!ctx) return;
    const image = ctx.createImageData(spec.frames, spec.bins);
    for (let f = 0; f < spec.frames; f++) {
      for (let b = 0; b < spec.bins; b++) {
        const [r, g, bl] = color(spec.values[f * spec.bins + b]);
        const p = ((spec.bins - 1 - b) * spec.frames + f) * 4;
        image.data[p] = r;
        image.data[p + 1] = g;
        image.data[p + 2] = bl;
        image.data[p + 3] = 255;
      }
    }
    ctx.putImageData(image, 0, 0);
  }, [spec]);

  return (
    <div className="spectro">
      <canvas ref={ref} width={spec.frames} height={spec.bins} className="plot spectro-canvas" />
      <div className="axis">
        <span>{Math.round(spec.maxHz)} Hz</span>
        <span>0 Hz</span>
      </div>
    </div>
  );
}
