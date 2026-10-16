import { useCallback, useEffect, useRef, useState } from "react";
import { fetchCues, fetchPath, fetchTrajectories, renderWav } from "./api";
import { LivePanner, WavPlayer } from "./audio/engine";
import { pointAt } from "./audio/path";
import { HeadView } from "./components/HeadView";
import { Readouts } from "./components/Readouts";
import { RenderPanel } from "./components/RenderPanel";
import type {
  CueReport,
  LiveSourceKind,
  Method,
  PathPoint,
  Position,
  SourceKind,
  Trajectory,
  TrajectoryInfo,
  TrajectoryType,
} from "./types";

const LIVE_SOURCES: LiveSourceKind[] = ["noise", "tone", "clicks"];

export function App() {
  const [position, setPosition] = useState<Position>({ azimuth: 45, elevation: 0, distance: 1.5 });
  const [cues, setCues] = useState<CueReport | null>(null);
  const [liveSource, setLiveSource] = useState<LiveSourceKind>("noise");
  const [liveOn, setLiveOn] = useState(false);
  const [trajectories, setTrajectories] = useState<TrajectoryInfo[]>([]);
  const [source, setSource] = useState<SourceKind>("voice-like");
  const [method, setMethod] = useState<Method>("hrtf");
  const [duration, setDuration] = useState(8);
  const [busy, setBusy] = useState(false);
  const [playing, setPlaying] = useState<TrajectoryType | null>(null);
  const [path, setPath] = useState<PathPoint[] | null>(null);
  const [marker, setMarker] = useState<PathPoint | null>(null);
  const [error, setError] = useState<string | null>(null);

  const live = useRef(new LivePanner());
  const player = useRef(new WavPlayer());
  const inFlight = useRef(false);
  const pending = useRef<Position | null>(null);

  const requestCues = useCallback((p: Position) => {
    if (inFlight.current) {
      pending.current = p;
      return;
    }
    inFlight.current = true;
    fetchCues(p)
      .then(setCues)
      .catch((e: unknown) => setError(String(e)))
      .finally(() => {
        inFlight.current = false;
        const next = pending.current;
        pending.current = null;
        if (next) requestCues(next);
      });
  }, []);

  useEffect(() => {
    fetchTrajectories()
      .then(setTrajectories)
      .catch((e: unknown) => setError(String(e)));
  }, []);

  useEffect(() => {
    requestCues(position);
    live.current.move(position);
  }, [position, requestCues]);

  useEffect(() => {
    if (!playing || !path) return;
    let frame = 0;
    const tick = () => {
      setMarker(pointAt(path, player.current.elapsed()));
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [playing, path]);

  useEffect(() => {
    const livePanner = live.current;
    const wavPlayer = player.current;
    return () => {
      livePanner.stop();
      wavPlayer.stop();
    };
  }, []);

  function toggleLive(): void {
    if (liveOn) {
      live.current.stop();
      setLiveOn(false);
      return;
    }
    stopPlayback();
    live.current.start(liveSource, position);
    setLiveOn(true);
  }

  function changeLiveSource(kind: LiveSourceKind): void {
    setLiveSource(kind);
    if (liveOn) live.current.start(kind, position);
  }

  function stopPlayback(): void {
    player.current.stop();
    setPlaying(null);
    setMarker(null);
  }

  async function render(trajectory: Trajectory): Promise<void> {
    setBusy(true);
    setError(null);
    live.current.stop();
    setLiveOn(false);
    try {
      const [wav, points] = await Promise.all([
        renderWav({ source, trajectory, duration, method }),
        fetchPath(trajectory, duration),
      ]);
      setPath(points);
      setPlaying(trajectory.type);
      await player.current.play(wav, () => {
        setPlaying(null);
        setMarker(null);
      });
    } catch (e) {
      setError(String(e));
      setPlaying(null);
    } finally {
      setBusy(false);
    }
  }

  const shown: Position = marker ?? position;
  const shownCues: CueReport | null = marker ?? cues;
  const staticTrajectory: Trajectory = { type: "static", ...position };

  return (
    <main>
      <header>
        <h1>Spatial Audio</h1>
        <p className="muted">
          Binaural panning with Woodworth ITD, head-shadow ILD and a parametric HRTF, rendered in Rust.
        </p>
        <p className="headphones">Wear headphones: binaural cues only work when each ear gets its own channel.</p>
      </header>

      {error && <p className="error">{error}</p>}

      <section className="panel stage">
        <div>
          <h2>Top-down head view</h2>
          <HeadView
            position={shown}
            path={playing ? path : null}
            draggable={!playing}
            onMove={(p) => setPosition(p)}
          />
          <p className="muted small">
            {playing ? "Following the rendered trajectory." : "Drag the purple source around the head."}
          </p>
        </div>
        <div className="side">
          <h2>Live preview (Web Audio HRTF)</h2>
          <div className="controls column">
            <label>
              Live source
              <select value={liveSource} onChange={(e) => changeLiveSource(e.target.value as LiveSourceKind)}>
                {LIVE_SOURCES.map((s) => (
                  <option key={s} value={s}>
                    {s}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Elevation {position.elevation.toFixed(0)}°
              <input
                type="range"
                min={-60}
                max={60}
                step={1}
                value={position.elevation}
                disabled={playing !== null}
                onChange={(e) => setPosition({ ...position, elevation: Number(e.target.value) })}
              />
            </label>
            <button onClick={toggleLive} disabled={playing !== null}>
              {liveOn ? "Stop live preview" : "Start live preview"}
            </button>
          </div>
          <h2>Cues</h2>
          <Readouts position={shown} cues={shownCues} />
        </div>
      </section>

      <RenderPanel
        trajectories={trajectories}
        source={source}
        method={method}
        duration={duration}
        busy={busy}
        playing={playing}
        onSource={setSource}
        onMethod={setMethod}
        onDuration={setDuration}
        onRender={(t) => void render(t)}
        onStop={stopPlayback}
        staticTrajectory={staticTrajectory}
      />
    </main>
  );
}
