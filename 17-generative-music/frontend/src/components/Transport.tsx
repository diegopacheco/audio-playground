import type { ExportKind } from "../types";

interface Props {
  playing: boolean;
  busy: boolean;
  hasSong: boolean;
  onGenerate: () => void;
  onNewSeed: () => void;
  onPlay: () => void;
  onStop: () => void;
  onDownload: (kind: ExportKind) => void;
}

export function Transport({ playing, busy, hasSong, onGenerate, onNewSeed, onPlay, onStop, onDownload }: Props) {
  return (
    <div className="transport">
      <button onClick={onGenerate} disabled={busy}>
        Generate
      </button>
      <button onClick={onNewSeed} disabled={busy}>
        New seed
      </button>
      {playing ? (
        <button className="primary" onClick={onStop}>
          Stop
        </button>
      ) : (
        <button className="primary" onClick={onPlay} disabled={!hasSong}>
          Play
        </button>
      )}
      <span className="spacer" />
      <button onClick={() => onDownload("midi")} disabled={!hasSong || busy}>
        Download MIDI
      </button>
      <button onClick={() => onDownload("wav")} disabled={!hasSong || busy}>
        Download WAV
      </button>
    </div>
  );
}
