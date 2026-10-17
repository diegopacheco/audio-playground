import { useCallback, useEffect, useRef, useState } from "react";
import { fetchEffects, fetchSource, processAudio } from "./api";
import { createItem, moveItem, removeItem, setParam, toConfig } from "./audio/chain";
import { ABPlayer, decodeWav } from "./audio/playback";
import { ChainEditor } from "./components/ChainEditor";
import { Monitor } from "./components/Monitor";
import { SourcePanel } from "./components/SourcePanel";
import type { ChainItem, Clip, EffectSpec, Side } from "./types";

const BUILT_IN = "built-in plucked tones";

function message(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function App() {
  const [specs, setSpecs] = useState<EffectSpec[]>([]);
  const [clip, setClip] = useState<Clip | null>(null);
  const [original, setOriginal] = useState<AudioBuffer | null>(null);
  const [processed, setProcessed] = useState<AudioBuffer | null>(null);
  const [items, setItems] = useState<ChainItem[]>([]);
  const [playing, setPlaying] = useState<Side | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const player = useRef(new ABPlayer());
  const nextId = useRef(1);

  const stop = useCallback(() => {
    player.current.stop();
    setPlaying(null);
  }, []);

  const loadClip = useCallback(
    async (next: Clip) => {
      try {
        stop();
        setOriginal(await decodeWav(next.bytes));
        setClip(next);
        setProcessed(null);
        setError("");
      } catch (e) {
        setError(`could not decode ${next.name}: ${message(e)}`);
      }
    },
    [stop],
  );

  const loadBuiltIn = useCallback(async () => {
    try {
      await loadClip({ name: BUILT_IN, bytes: await fetchSource() });
    } catch (e) {
      setError(message(e));
    }
  }, [loadClip]);

  useEffect(() => {
    fetchEffects()
      .then(setSpecs)
      .catch((e: unknown) => setError(message(e)));
    void loadBuiltIn();
  }, [loadBuiltIn]);

  const process = async () => {
    if (!clip) return;
    setBusy(true);
    try {
      stop();
      setProcessed(await decodeWav(await processAudio(clip.bytes, toConfig(items))));
      setError("");
    } catch (e) {
      setError(message(e));
    } finally {
      setBusy(false);
    }
  };

  const play = (side: Side) => {
    const buffer = side === "A" ? original : processed;
    if (!buffer) return;
    player.current.play(buffer, side, () => setPlaying(null));
    setPlaying(side);
  };

  const add = (spec: EffectSpec) => {
    setItems((current) => [...current, createItem(spec, `fx-${nextId.current++}`)]);
  };

  return (
    <main>
      <header>
        <h1>Effects Chain</h1>
        <p className="muted">Build a chain of audio effects, render it in Rust, and compare A/B.</p>
      </header>
      {error && <div className="error">{error}</div>}
      <SourcePanel clip={clip} onBuiltIn={loadBuiltIn} onUpload={loadClip} />
      <ChainEditor
        specs={specs}
        items={items}
        onAdd={add}
        onParam={(id, name, value) => setItems((c) => setParam(c, id, name, value))}
        onMove={(id, delta) => setItems((c) => moveItem(c, id, delta))}
        onRemove={(id) => setItems((c) => removeItem(c, id))}
      />
      <Monitor
        original={original}
        processed={processed}
        playing={playing}
        busy={busy}
        canProcess={clip !== null}
        onProcess={process}
        onPlay={play}
        onStop={stop}
      />
    </main>
  );
}
