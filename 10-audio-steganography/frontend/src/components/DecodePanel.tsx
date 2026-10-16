import { useState } from "react";
import { decode } from "../api";
import type { DecodeResult, MethodConfig } from "../types";
import { MethodPicker } from "./MethodPicker";

interface Props {
  config: MethodConfig;
  onConfig: (config: MethodConfig) => void;
  lastEncoded: Blob | null;
}

export function DecodePanel({ config, onConfig, lastEncoded }: Props) {
  const [file, setFile] = useState<File | null>(null);
  const [result, setResult] = useState<DecodeResult | null>(null);
  const [error, setError] = useState("");
  const source = file ?? lastEncoded;

  async function run() {
    if (!source) return;
    setError("");
    setResult(null);
    try {
      setResult(await decode(config, source));
    } catch (e) {
      setError((e as Error).message);
    }
  }

  return (
    <section className="card form">
      <h2>Reveal a message</h2>
      <label>
        stego wav
        <span className="row">
          <input type="file" accept=".wav,audio/wav" onChange={(e) => setFile(e.target.files?.[0] ?? null)} />
          <span className="muted">
            {file ? file.name : lastEncoded ? "last encoded clip" : "choose a file or encode one first"}
          </span>
        </span>
      </label>
      <MethodPicker value={config} onChange={onConfig} />
      <button type="button" className="primary" disabled={!source} onClick={run}>
        decode
      </button>
      {result && (
        <div className="revealed">
          <span className="label">{result.bytes} bytes recovered, crc32 verified</span>
          <p>{result.message}</p>
        </div>
      )}
      {error && <p className="error">{error}</p>}
    </section>
  );
}
