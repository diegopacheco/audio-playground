import { useEffect, useRef, useState } from "react";
import { isFlashing, lastBeatIndex } from "../audio/beats";

interface Props {
  url: string;
  name: string;
  beats: number[];
  onTime: (time: number) => void;
}

export function Player({ url, name, beats, onTime }: Props) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [time, setTime] = useState(0);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    if (!playing) return;
    let frame = 0;
    const tick = () => {
      const current = audioRef.current?.currentTime ?? 0;
      setTime(current);
      onTime(current);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [playing, onTime]);

  const flashing = playing && isFlashing(beats, time);
  const beatNumber = lastBeatIndex(beats, time) + 1;

  return (
    <section className="card player">
      <div className={`beat-light ${flashing ? "on" : ""}`}>
        <span>{beatNumber > 0 ? ((beatNumber - 1) % 4) + 1 : "-"}</span>
      </div>
      <div className="player-body">
        <div className="muted">{name}</div>
        <audio
          ref={audioRef}
          src={url}
          controls
          onPlay={() => setPlaying(true)}
          onPause={() => setPlaying(false)}
          onEnded={() => setPlaying(false)}
          onSeeked={() => {
            const current = audioRef.current?.currentTime ?? 0;
            setTime(current);
            onTime(current);
          }}
        />
        <div className="muted">
          beat {beatNumber} of {beats.length}
        </div>
      </div>
    </section>
  );
}
