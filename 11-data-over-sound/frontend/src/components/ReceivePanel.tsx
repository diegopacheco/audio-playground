import { useState } from "react";
import { liveText } from "../audio/frame";
import type { ModemSettings, PcmAudio, ReceiveSource } from "../types";
import type { Receiver } from "../useReceiver";
import { DecodeLog } from "./DecodeLog";
import { LoopbackControls } from "./LoopbackControls";
import { MicControls } from "./MicControls";

interface Props {
  settings: ModemSettings;
  sent: PcmAudio | null;
  receiver: Receiver;
}

export function ReceivePanel({ settings, sent, receiver }: Props) {
  const [source, setSource] = useState<ReceiveSource>("loopback");

  function choose(next: ReceiveSource) {
    receiver.close();
    setSource(next);
  }

  return (
    <section className="card">
      <h2>Receive</h2>
      <div className="tabs">
        <button className={source === "loopback" ? "tab active" : "tab"} onClick={() => choose("loopback")}>
          Loopback (software noise)
        </button>
        <button className={source === "mic" ? "tab active" : "tab"} onClick={() => choose("mic")}>
          Microphone
        </button>
      </div>
      {source === "mic" ? (
        <MicControls receiver={receiver} />
      ) : (
        <LoopbackControls settings={settings} sent={sent} receiver={receiver} />
      )}
      <div className="status">
        <span className={receiver.active ? "dot on" : "dot"} />
        {receiver.status}
        <button className="link-button" onClick={receiver.clear}>
          clear
        </button>
      </div>
      <div className="live">{liveText(receiver.bytes) || <span className="muted">waiting for data</span>}</div>
      {receiver.frames.length > 0 && (
        <ul className="frames">
          {receiver.frames.map((f, i) => (
            <li key={i} className={f.crc_ok ? "ok" : "error"}>
              "{f.text}" · CRC {f.crc_ok ? "ok" : "FAILED"} · SNR {f.snr} dB
            </li>
          ))}
        </ul>
      )}
      <DecodeLog bytes={receiver.bytes} />
    </section>
  );
}
