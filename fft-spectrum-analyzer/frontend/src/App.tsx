import { useState } from "react";
import { BarsView } from "./components/BarsView";
import { Controls } from "./components/Controls";
import { PeakReadout } from "./components/PeakReadout";
import { Spectrogram } from "./components/Spectrogram";
import type { Settings } from "./types";
import { useAnalyzer } from "./useAnalyzer";

const INITIAL: Settings = { window: "hann", fftSize: 2048, source: "tone", toneHz: 440 };

export function App() {
  const [settings, setSettings] = useState<Settings>(INITIAL);
  const analyzer = useAnalyzer(settings);

  return (
    <main>
      <header>
        <h1>FFT Spectrum Analyzer</h1>
        <p>Browser audio is framed and streamed to a hand-written Rust radix-2 FFT over a WebSocket.</p>
      </header>
      <Controls
        settings={settings}
        running={analyzer.running}
        onChange={setSettings}
        onStart={() => void analyzer.start()}
        onStop={() => void analyzer.stop()}
        onSnapshot={() => void analyzer.takeSnapshot()}
      />
      {analyzer.error && <div className="error">{analyzer.error}</div>}
      <PeakReadout frame={analyzer.frame} snapshot={analyzer.snapshot} />
      <BarsView frame={analyzer.frame} />
      <Spectrogram frame={analyzer.frame} />
    </main>
  );
}
