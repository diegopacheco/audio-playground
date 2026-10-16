import { useEffect, useRef } from "react";
import { envelopeOutline } from "../audio/envelope";

interface Props {
  envelope: number[];
}

const WIDTH = 900;
const HEIGHT = 180;

export function EnvelopeCanvas({ envelope }: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const ctx = canvas.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, WIDTH, HEIGHT);
    ctx.strokeStyle = "#cbd5e1";
    ctx.beginPath();
    ctx.moveTo(0, HEIGHT / 2);
    ctx.lineTo(WIDTH, HEIGHT / 2);
    ctx.stroke();
    const points = envelopeOutline(envelope, WIDTH, HEIGHT);
    if (points.length === 0) return;
    ctx.beginPath();
    points.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
    ctx.closePath();
    ctx.fillStyle = "rgba(99, 102, 241, 0.35)";
    ctx.strokeStyle = "#4f46e5";
    ctx.fill();
    ctx.stroke();
  }, [envelope]);

  return (
    <section className="card">
      <h2>Envelope</h2>
      <canvas ref={canvas} width={WIDTH} height={HEIGHT} className="envelope" />
    </section>
  );
}
