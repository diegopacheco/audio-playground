import { useEffect, useState } from "react";
import { inspectWav } from "./api";
import type { AsciiSize, Report } from "./types";
import { DropZone } from "./components/DropZone";
import { HeaderTable } from "./components/HeaderTable";
import { ChunkList } from "./components/ChunkList";
import { AsciiWave } from "./components/AsciiWave";
import { EnvelopeCanvas } from "./components/EnvelopeCanvas";

export function App() {
  const [file, setFile] = useState<File | null>(null);
  const [size, setSize] = useState<AsciiSize>({ width: 100, height: 16 });
  const [report, setReport] = useState<Report | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!file) return;
    let active = true;
    setBusy(true);
    inspectWav(file, size)
      .then((r) => {
        if (!active) return;
        setReport(r);
        setError(null);
      })
      .catch((e: unknown) => {
        if (!active) return;
        setReport(null);
        setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => active && setBusy(false));
    return () => {
      active = false;
    };
  }, [file, size]);

  return (
    <main>
      <header>
        <h1>WAV Inspector</h1>
        <p>RIFF chunks, header fields, levels and waveform of any PCM or float WAV file.</p>
      </header>
      <DropZone onFile={setFile} busy={busy} />
      {file && <p className="file">{file.name}</p>}
      {error && <p className="error">{error}</p>}
      {report && (
        <>
          <div className="grid">
            <HeaderTable report={report} />
            <ChunkList chunks={report.chunks} />
          </div>
          <AsciiWave ascii={report.ascii} size={size} onSize={setSize} />
          <EnvelopeCanvas envelope={report.envelope} />
        </>
      )}
    </main>
  );
}
