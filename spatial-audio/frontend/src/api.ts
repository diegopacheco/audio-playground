import type { CueReport, PathPoint, Position, RenderRequest, Trajectory, TrajectoryInfo } from "./types";

async function check(response: Response): Promise<Response> {
  if (response.ok) return response;
  const body = (await response.json().catch(() => null)) as { error?: string } | null;
  throw new Error(body?.error ?? `request failed with ${response.status}`);
}

function postJson(url: string, body: unknown): Promise<Response> {
  return fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function fetchTrajectories(): Promise<TrajectoryInfo[]> {
  const response = await check(await fetch("/api/trajectories"));
  return (await response.json()) as TrajectoryInfo[];
}

export async function fetchCues(position: Position): Promise<CueReport> {
  const query = new URLSearchParams({
    azimuth: position.azimuth.toFixed(2),
    elevation: position.elevation.toFixed(2),
    distance: position.distance.toFixed(3),
  });
  const response = await check(await fetch(`/api/itd?${query.toString()}`));
  return (await response.json()) as CueReport;
}

export async function renderWav(request: RenderRequest): Promise<ArrayBuffer> {
  const response = await check(await postJson("/api/render", request));
  return response.arrayBuffer();
}

export async function fetchPath(trajectory: Trajectory, duration: number): Promise<PathPoint[]> {
  const response = await check(await postJson("/api/path", { trajectory, duration }));
  return (await response.json()) as PathPoint[];
}
