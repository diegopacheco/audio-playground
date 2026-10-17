import { useRef, useState } from "react";
import { startRecording, type Recording } from "../audio/recorder";

interface Props {
  busy: boolean;
  onAudio: (audio: Blob, label: string) => void;
  onError: (message: string) => void;
}

export function AudioInput({ busy, onAudio, onError }: Props) {
  const recording = useRef<Recording | null>(null);
  const [recordingActive, setRecordingActive] = useState(false);

  async function toggleRecording() {
    if (recording.current) {
      const blob = await recording.current.stop();
      recording.current = null;
      setRecordingActive(false);
      onAudio(blob, "microphone recording");
      return;
    }
    try {
      recording.current = await startRecording();
      setRecordingActive(true);
    } catch (e) {
      onError(e instanceof Error ? e.message : "microphone unavailable");
    }
  }

  function onFile(files: FileList | null) {
    const file = files?.[0];
    if (file) onAudio(file, file.name);
  }

  return (
    <section className="card">
      <h2>Audio</h2>
      <div className="row">
        <button className={recordingActive ? "danger" : "primary"} disabled={busy} onClick={toggleRecording}>
          {recordingActive ? "Stop and transcribe" : "Record from mic"}
        </button>
        <label className={`upload ${busy ? "disabled" : ""}`}>
          Upload WAV / WebM
          <input
            type="file"
            accept="audio/wav,audio/x-wav,audio/webm,audio/*"
            disabled={busy}
            onChange={(e) => onFile(e.target.files)}
          />
        </label>
        {recordingActive && <span className="live">recording...</span>}
      </div>
    </section>
  );
}
