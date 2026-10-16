import { useEffect, useState } from "react";
import { fetchSchemes } from "./api";
import { ReceivePanel } from "./components/ReceivePanel";
import { SendPanel } from "./components/SendPanel";
import { SettingsBar } from "./components/SettingsBar";
import type { ModemSettings, PcmAudio, SchemeInfo } from "./types";
import { useReceiver } from "./useReceiver";

export function App() {
  const [settings, setSettings] = useState<ModemSettings>({ scheme: "bfsk", baud: 300 });
  const [schemes, setSchemes] = useState<SchemeInfo[]>([]);
  const [sent, setSent] = useState<PcmAudio | null>(null);
  const [error, setError] = useState<string | null>(null);
  const receiver = useReceiver(settings);

  useEffect(() => {
    fetchSchemes()
      .then(setSchemes)
      .catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)));
  }, []);

  function change(next: ModemSettings) {
    receiver.close();
    setSent(null);
    setSettings(next);
  }

  return (
    <main>
      <header>
        <h1>Data Over Sound</h1>
        <p className="muted">
          Acoustic modem: preamble, sync word, length, payload and CRC16 framed as UART bytes, sent as FSK tones.
        </p>
      </header>
      {error && <p className="error">{error}</p>}
      <SettingsBar settings={settings} schemes={schemes} onChange={change} />
      <div className="grid">
        <SendPanel settings={settings} sent={sent} onSent={setSent} />
        <ReceivePanel settings={settings} sent={sent} receiver={receiver} />
      </div>
    </main>
  );
}
