import type { PannerCoordinates, Point, Position } from "../types";

export const MIN_DISTANCE = 0.3;

export function wrapAzimuth(azimuth: number): number {
  const wrapped = ((((azimuth + 180) % 360) + 360) % 360) - 180;
  return wrapped === -180 ? 180 : wrapped;
}

export function canvasToPosition(
  point: Point,
  center: Point,
  pixelsPerMeter: number,
  maxDistance: number,
  elevation: number,
): Position {
  const x = (point.x - center.x) / pixelsPerMeter;
  const y = (center.y - point.y) / pixelsPerMeter;
  const horizontal = Math.min(Math.max(Math.hypot(x, y), MIN_DISTANCE), maxDistance);
  const elevationRad = (elevation * Math.PI) / 180;
  return {
    azimuth: (Math.atan2(x, y) * 180) / Math.PI,
    elevation,
    distance: horizontal / Math.cos(elevationRad),
  };
}

export function positionToCanvas(position: Position, center: Point, pixelsPerMeter: number): Point {
  const az = (position.azimuth * Math.PI) / 180;
  const el = (position.elevation * Math.PI) / 180;
  const horizontal = position.distance * Math.cos(el) * pixelsPerMeter;
  return {
    x: center.x + horizontal * Math.sin(az),
    y: center.y - horizontal * Math.cos(az),
  };
}

export function toPanner(position: Position): PannerCoordinates {
  const az = (position.azimuth * Math.PI) / 180;
  const el = (position.elevation * Math.PI) / 180;
  const d = position.distance;
  return {
    x: d * Math.cos(el) * Math.sin(az),
    y: d * Math.sin(el),
    z: -d * Math.cos(el) * Math.cos(az),
  };
}
