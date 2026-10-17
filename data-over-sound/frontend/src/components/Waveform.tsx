import { useEffect, useRef } from "react";

interface Props {
  samples: Float32Array;
  color: string;
}

const WIDTH = 720;
const HEIGHT = 110;

export function Waveform({ samples, color }: Props) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const ctx = ref.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, WIDTH, HEIGHT);
    ctx.strokeStyle = "#d8dee8";
    ctx.beginPath();
    ctx.moveTo(0, HEIGHT / 2);
    ctx.lineTo(WIDTH, HEIGHT / 2);
    ctx.stroke();
    if (samples.length === 0) return;
    const per = Math.max(1, Math.floor(samples.length / WIDTH));
    ctx.strokeStyle = color;
    ctx.beginPath();
    for (let x = 0; x < WIDTH; x++) {
      let min = 0;
      let max = 0;
      const start = Math.floor((x * samples.length) / WIDTH);
      for (let i = start; i < Math.min(samples.length, start + per); i++) {
        min = Math.min(min, samples[i]);
        max = Math.max(max, samples[i]);
      }
      ctx.moveTo(x + 0.5, HEIGHT / 2 - max * (HEIGHT / 2));
      ctx.lineTo(x + 0.5, HEIGHT / 2 - min * (HEIGHT / 2));
    }
    ctx.stroke();
  }, [samples, color]);

  return <canvas ref={ref} width={WIDTH} height={HEIGHT} className="plot" />;
}
