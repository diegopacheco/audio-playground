import { useEffect, useRef, useState } from "react";
import { startMic, type MicCapture } from "../audio/mic";
import type { Receiver } from "../useReceiver";

interface Props {
  receiver: Receiver;
}

export function MicControls({ receiver }: Props) {
  const mic = useRef<MicCapture | null>(null);
  const [level, setLevel] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const listening = mic.current !== null && receiver.active;

  useEffect(() => () => mic.current?.stop(), []);

  async function start() {
    setError(null);
    try {
      const capture = await startMic((frame) => {
        receiver.send(frame);
        let peak = 0;
        for (const s of frame) peak = Math.max(peak, Math.abs(s));
        setLevel(peak);
      });
      mic.current = capture;
      await receiver.open(capture.sampleRate);
    } catch (e) {
      stop();
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  function stop() {
    mic.current?.stop();
    mic.current = null;
    receiver.close();
    setLevel(0);
  }

  return (
    <div className="controls">
      <div className="row">
        {listening ? (
          <button onClick={stop}>Stop listening</button>
        ) : (
          <button onClick={() => void start()}>Listen on microphone</button>
        )}
        <div className="meter" title="input peak level">
          <div style={{ width: `${Math.min(100, level * 100)}%` }} />
        </div>
      </div>
      {error && <p className="error">{error}</p>}
    </div>
  );
}
