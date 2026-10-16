import { useEffect, useRef, useState } from "react";

interface Props {
  original: Blob;
  stego: Blob;
}

type Side = "A" | "B";

export function ABPlayer({ original, stego }: Props) {
  const audio = useRef<HTMLAudioElement>(null);
  const [urls, setUrls] = useState<{ A: string; B: string } | null>(null);
  const [side, setSide] = useState<Side | null>(null);

  useEffect(() => {
    const next = { A: URL.createObjectURL(original), B: URL.createObjectURL(stego) };
    setUrls(next);
    return () => {
      URL.revokeObjectURL(next.A);
      URL.revokeObjectURL(next.B);
    };
  }, [original, stego]);

  function play(next: Side) {
    const el = audio.current;
    if (!el || !urls) return;
    const at = side ? el.currentTime : 0;
    el.src = urls[next];
    el.currentTime = at;
    void el.play();
    setSide(next);
  }

  function stop() {
    audio.current?.pause();
    setSide(null);
  }

  return (
    <div className="ab">
      <audio ref={audio} onEnded={() => setSide(null)} />
      <button type="button" className={side === "A" ? "chip active" : "chip"} onClick={() => play("A")}>
        A original
      </button>
      <button type="button" className={side === "B" ? "chip active" : "chip"} onClick={() => play("B")}>
        B stego
      </button>
      <button type="button" className="chip" onClick={stop}>
        stop
      </button>
      {urls && (
        <a className="download" href={urls.B} download="stego.wav">
          download stego.wav
        </a>
      )}
    </div>
  );
}
