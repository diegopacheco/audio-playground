import { useCallback, useEffect, useState } from "react";
import { addTrack, fetchClip, identifyPcm, identifyWav, listLibrary } from "./api";
import { recordMic } from "./audio/recorder";
import { LibraryPanel } from "./components/LibraryPanel";
import { QueryPanel } from "./components/QueryPanel";
import { ResultPanel } from "./components/ResultPanel";
import type { ClipRequest, Identification, QueryResult, TrackInfo } from "./types";

export function App() {
  const [tracks, setTracks] = useState<TrackInfo[]>([]);
  const [result, setResult] = useState<QueryResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [audioUrl, setAudioUrl] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setTracks(await listLibrary());
  }, []);

  useEffect(() => {
    refresh().catch((e: Error) => setError(e.message));
  }, [refresh]);

  const run = async (label: string, task: () => Promise<Identification>) => {
    setBusy(true);
    setError("");
    setStatus(`${label}...`);
    try {
      setResult({ label, identification: await task() });
      setStatus("");
    } catch (e) {
      setError((e as Error).message);
      setStatus("");
    } finally {
      setBusy(false);
    }
  };

  const showAudio = (wav: ArrayBuffer) => {
    if (audioUrl) {
      URL.revokeObjectURL(audioUrl);
    }
    setAudioUrl(URL.createObjectURL(new Blob([wav], { type: "audio/wav" })));
  };

  const onFile = (file: File) =>
    run(`file ${file.name}`, async () => {
      const wav = await file.arrayBuffer();
      showAudio(wav);
      return identifyWav(wav);
    });

  const onRecord = (seconds: number) =>
    run(`microphone ${seconds}s`, async () => {
      setStatus(`Recording ${seconds}s from the microphone...`);
      const recording = await recordMic(seconds);
      return identifyPcm(recording.samples, recording.rate);
    });

  const onClip = (clip: ClipRequest) => {
    const name = tracks.find((t) => t.id === clip.track)?.name ?? `track ${clip.track}`;
    return run(
      `${clip.len}s clip of ${name} from ${clip.start.toFixed(2)}s, noise ${clip.noise.toFixed(1)}`,
      async () => {
        const wav = await fetchClip(clip);
        showAudio(wav);
        return identifyWav(wav);
      },
    );
  };

  const onAdd = async (name: string, wav: ArrayBuffer) => {
    setBusy(true);
    setError("");
    try {
      await addTrack(name, wav);
      await refresh();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <main>
      <header>
        <h1>Audio Fingerprinting</h1>
        <p>Constellation peaks, paired hashes and offset histograms in Rust: a tiny Shazam.</p>
      </header>
      <div className="layout">
        <div className="column">
          <QueryPanel
            tracks={tracks}
            busy={busy}
            onFile={onFile}
            onRecord={onRecord}
            onClip={onClip}
          />
          {status && <p className="status">{status}</p>}
          {error && <p className="error">{error}</p>}
          {audioUrl && <audio className="player" controls src={audioUrl} />}
          <LibraryPanel tracks={tracks} busy={busy} onAdd={onAdd} />
        </div>
        <div className="column">
          {result ? (
            <ResultPanel result={result} />
          ) : (
            <section className="card empty">Identify a clip to see its constellation map.</section>
          )}
        </div>
      </div>
    </main>
  );
}
