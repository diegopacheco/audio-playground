import type { PathPoint } from "../types";

function lerp(a: number, b: number, f: number): number {
  return a + (b - a) * f;
}

function lerpAngle(a: number, b: number, f: number): number {
  let delta = b - a;
  if (delta > 180) delta -= 360;
  if (delta < -180) delta += 360;
  const value = a + delta * f;
  return value > 180 ? value - 360 : value < -180 ? value + 360 : value;
}

export function pointAt(path: PathPoint[], t: number): PathPoint | null {
  if (path.length === 0) return null;
  const first = path[0];
  const last = path[path.length - 1];
  if (t <= first.t) return first;
  if (t >= last.t) return last;
  let hi = path.findIndex((p) => p.t >= t);
  if (hi <= 0) hi = 1;
  const a = path[hi - 1];
  const b = path[hi];
  const f = (t - a.t) / (b.t - a.t);
  return {
    t,
    azimuth: lerpAngle(a.azimuth, b.azimuth, f),
    elevation: lerp(a.elevation, b.elevation, f),
    distance: lerp(a.distance, b.distance, f),
    itd_ms: lerp(a.itd_ms, b.itd_ms, f),
    ild_db: lerp(a.ild_db, b.ild_db, f),
    ild_probe_hz: a.ild_probe_hz,
  };
}
