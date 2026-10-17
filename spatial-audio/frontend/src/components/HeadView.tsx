import { useEffect, useRef } from "react";
import type { PointerEvent } from "react";
import { canvasToPosition, positionToCanvas } from "../audio/geometry";
import type { PathPoint, Point, Position } from "../types";

const SIZE = 440;
const PIXELS_PER_METER = 60;
const MAX_DISTANCE = 3.4;
const CENTER: Point = { x: SIZE / 2, y: SIZE / 2 };
const HEAD_RADIUS = 22;

interface Props {
  position: Position;
  path: PathPoint[] | null;
  draggable: boolean;
  onMove: (position: Position) => void;
}

function drawGrid(ctx: CanvasRenderingContext2D): void {
  ctx.strokeStyle = "#e3e7ee";
  ctx.lineWidth = 1;
  for (let m = 1; m <= 3; m++) {
    ctx.beginPath();
    ctx.arc(CENTER.x, CENTER.y, m * PIXELS_PER_METER, 0, Math.PI * 2);
    ctx.stroke();
    ctx.fillStyle = "#9aa3b5";
    ctx.font = "11px system-ui";
    ctx.fillText(`${m} m`, CENTER.x + 4, CENTER.y - m * PIXELS_PER_METER + 12);
  }
  ctx.beginPath();
  ctx.moveTo(CENTER.x, 10);
  ctx.lineTo(CENTER.x, SIZE - 10);
  ctx.moveTo(10, CENTER.y);
  ctx.lineTo(SIZE - 10, CENTER.y);
  ctx.stroke();
  ctx.fillStyle = "#7a8399";
  ctx.font = "12px system-ui";
  ctx.fillText("front 0°", CENTER.x + 6, 22);
  ctx.fillText("back 180°", CENTER.x + 6, SIZE - 14);
  ctx.fillText("+90° right", SIZE - 72, CENTER.y - 6);
  ctx.fillText("-90° left", 12, CENTER.y - 6);
}

function drawHead(ctx: CanvasRenderingContext2D): void {
  ctx.fillStyle = "#fde7c8";
  ctx.strokeStyle = "#c79a5e";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(CENTER.x - 7, CENTER.y - HEAD_RADIUS + 3);
  ctx.lineTo(CENTER.x, CENTER.y - HEAD_RADIUS - 9);
  ctx.lineTo(CENTER.x + 7, CENTER.y - HEAD_RADIUS + 3);
  ctx.fill();
  ctx.stroke();
  for (const side of [-1, 1]) {
    ctx.beginPath();
    ctx.ellipse(CENTER.x + side * HEAD_RADIUS, CENTER.y, 5, 9, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
  }
  ctx.beginPath();
  ctx.arc(CENTER.x, CENTER.y, HEAD_RADIUS, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = "#8a6a3c";
  ctx.font = "bold 11px system-ui";
  ctx.fillText("L", CENTER.x - HEAD_RADIUS - 20, CENTER.y + 4);
  ctx.fillText("R", CENTER.x + HEAD_RADIUS + 12, CENTER.y + 4);
}

function drawPath(ctx: CanvasRenderingContext2D, path: PathPoint[]): void {
  ctx.strokeStyle = "rgba(108, 92, 231, 0.35)";
  ctx.lineWidth = 2;
  ctx.setLineDash([5, 5]);
  ctx.beginPath();
  path.forEach((p, i) => {
    const c = positionToCanvas(p, CENTER, PIXELS_PER_METER);
    const x = Math.min(Math.max(c.x, 0), SIZE);
    const y = Math.min(Math.max(c.y, 0), SIZE);
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.stroke();
  ctx.setLineDash([]);
}

function drawSource(ctx: CanvasRenderingContext2D, position: Position): void {
  const c = positionToCanvas(position, CENTER, PIXELS_PER_METER);
  const x = Math.min(Math.max(c.x, 8), SIZE - 8);
  const y = Math.min(Math.max(c.y, 8), SIZE - 8);
  ctx.strokeStyle = "rgba(108, 92, 231, 0.5)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(CENTER.x, CENTER.y);
  ctx.lineTo(x, y);
  ctx.stroke();
  const radius = 10 + position.elevation / 15;
  ctx.fillStyle = "#6c5ce7";
  ctx.beginPath();
  ctx.arc(x, y, Math.max(radius, 4), 0, Math.PI * 2);
  ctx.fill();
}

export function HeadView({ position, path, draggable, onMove }: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const dragging = useRef(false);

  useEffect(() => {
    const ctx = canvasRef.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, SIZE, SIZE);
    drawGrid(ctx);
    if (path) drawPath(ctx, path);
    drawHead(ctx);
    drawSource(ctx, position);
  }, [position, path]);

  function pointFrom(event: PointerEvent<HTMLCanvasElement>): Point {
    const rect = event.currentTarget.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / rect.width) * SIZE,
      y: ((event.clientY - rect.top) / rect.height) * SIZE,
    };
  }

  function move(event: PointerEvent<HTMLCanvasElement>): void {
    onMove(canvasToPosition(pointFrom(event), CENTER, PIXELS_PER_METER, MAX_DISTANCE, position.elevation));
  }

  return (
    <canvas
      ref={canvasRef}
      className={draggable ? "head-view draggable" : "head-view"}
      width={SIZE}
      height={SIZE}
      onPointerDown={(e) => {
        if (!draggable) return;
        dragging.current = true;
        e.currentTarget.setPointerCapture(e.pointerId);
        move(e);
      }}
      onPointerMove={(e) => {
        if (draggable && dragging.current) move(e);
      }}
      onPointerUp={() => {
        dragging.current = false;
      }}
    />
  );
}
