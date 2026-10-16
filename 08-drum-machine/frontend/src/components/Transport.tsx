interface Props {
  playing: boolean;
  bpm: number;
  swing: number;
  onPlay: () => void;
  onStop: () => void;
  onBpm: (bpm: number) => void;
  onSwing: (swing: number) => void;
}

export function Transport({ playing, bpm, swing, onPlay, onStop, onBpm, onSwing }: Props) {
  return (
    <div className="transport">
      <button className="primary" onClick={playing ? onStop : onPlay}>
        {playing ? "Stop" : "Play"}
      </button>
      <label>
        BPM <strong>{bpm}</strong>
        <input type="range" min={60} max={200} value={bpm} onChange={(e) => onBpm(Number(e.target.value))} />
      </label>
      <label>
        Swing <strong>{Math.round(swing * 100)}%</strong>
        <input
          type="range"
          min={0}
          max={0.5}
          step={0.01}
          value={swing}
          onChange={(e) => onSwing(Number(e.target.value))}
        />
      </label>
    </div>
  );
}
