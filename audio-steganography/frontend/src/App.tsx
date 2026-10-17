import { useState } from "react";
import { DecodePanel } from "./components/DecodePanel";
import { EncodePanel } from "./components/EncodePanel";
import type { MethodConfig, Tab } from "./types";

export function App() {
  const [tab, setTab] = useState<Tab>("encode");
  const [config, setConfig] = useState<MethodConfig>({ method: "lsb", bits: 1 });
  const [lastEncoded, setLastEncoded] = useState<Blob | null>(null);

  return (
    <main>
      <header>
        <h1>Audio Steganography</h1>
        <p className="muted">Hide text inside a WAV file with LSB embedding or near-inaudible FSK tones.</p>
      </header>
      <nav className="tabs">
        {(["encode", "decode"] as Tab[]).map((t) => (
          <button key={t} type="button" className={t === tab ? "tab active" : "tab"} onClick={() => setTab(t)}>
            {t === "encode" ? "Encode" : "Decode"}
          </button>
        ))}
      </nav>
      <div hidden={tab !== "encode"}>
        <EncodePanel config={config} onConfig={setConfig} onEncoded={setLastEncoded} />
      </div>
      <div hidden={tab !== "decode"}>
        <DecodePanel config={config} onConfig={setConfig} lastEncoded={lastEncoded} />
      </div>
    </main>
  );
}
