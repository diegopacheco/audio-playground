import { useEffect, useState } from "react";
import { encode, fetchCapacity, fetchCarrier } from "../api";
import { diffSamples, diffStats, utf8Length } from "../audio/diff";
import { parseWav } from "../audio/wav";
import type { Capacity, DiffStats, EncodeResult, MethodConfig } from "../types";
import { ABPlayer } from "./ABPlayer";
import { DiffCanvas } from "./DiffCanvas";
import { MethodPicker } from "./MethodPicker";
import { StatsPanel } from "./StatsPanel";

interface Props {
  config: MethodConfig;
  onConfig: (config: MethodConfig) => void;
  onEncoded: (wav: Blob) => void;
}

const ZOOM = 1024;

interface Outcome {
  original: Blob;
  result: EncodeResult;
  diff: Int16Array;
  stats: DiffStats;
}

export function EncodePanel({ config, onConfig, onEncoded }: Props) {
  const [file, setFile] = useState<File | null>(null);
  const [message, setMessage] = useState("the owl flies at midnight");
  const [capacity, setCapacity] = useState<Capacity | null>(null);
  const [outcome, setOutcome] = useState<Outcome | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    fetchCapacity(config, file)
      .then((c) => {
        if (live) setCapacity(c);
      })
      .catch((e: Error) => {
        if (!live) return;
        setCapacity(null);
        setError(e.message);
      });
    return () => {
      live = false;
    };
  }, [config, file]);

  const used = utf8Length(message);
  const over = capacity !== null && used > capacity.bytes;

  async function run() {
    setBusy(true);
    setError("");
    try {
      const original = file ?? (await fetchCarrier());
      const result = await encode(config, message, file);
      const a = parseWav(await original.arrayBuffer()).samples;
      const b = parseWav(await result.wav.arrayBuffer()).samples;
      setOutcome({ original, result, diff: diffSamples(a, b), stats: diffStats(a, b) });
      onEncoded(result.wav);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <section className="card form">
        <h2>Hide a message</h2>
        <label>
          carrier
          <span className="row">
            <input type="file" accept=".wav,audio/wav" onChange={(e) => setFile(e.target.files?.[0] ?? null)} />
            <span className="muted">{file ? file.name : "built-in 6 s chord clip"}</span>
          </span>
        </label>
        <MethodPicker value={config} onChange={onConfig} />
        <label>
          message
          <textarea rows={3} value={message} onChange={(e) => setMessage(e.target.value)} />
        </label>
        <div className={over ? "capacity over" : "capacity"}>
          <div className="bar">
            <div style={{ width: `${capacity ? Math.min(100, (100 * used) / Math.max(1, capacity.bytes)) : 0}%` }} />
          </div>
          <span>
            {used} / {capacity ? capacity.bytes : "?"} bytes
            {capacity && ` · ${capacity.sampleRate} Hz · ${capacity.seconds.toFixed(1)} s`}
          </span>
        </div>
        <button type="button" className="primary" disabled={busy || over || message.length === 0} onClick={run}>
          {busy ? "encoding..." : "encode"}
        </button>
        {error && <p className="error">{error}</p>}
      </section>
      {outcome && (
        <section className="card">
          <h2>Result</h2>
          <StatsPanel result={outcome.result} stats={outcome.stats} totalSamples={outcome.diff.length} />
          <ABPlayer original={outcome.original} stego={outcome.result.wav} />
          <DiffCanvas
            diff={outcome.diff}
            maxDiff={outcome.stats.maxDiff}
            caption="stego minus original over the whole clip"
          />
          <DiffCanvas
            diff={outcome.diff.subarray(0, ZOOM)}
            maxDiff={outcome.stats.maxDiff}
            caption={`zoom on the first ${ZOOM} samples where the payload starts`}
          />
        </section>
      )}
    </>
  );
}
