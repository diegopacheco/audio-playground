import { useEffect, useState } from "react";
import { fetchHealth, summarize, transcribe } from "./api";
import { AudioInput } from "./components/AudioInput";
import { HealthBadges } from "./components/HealthBadges";
import { SummaryPanel } from "./components/SummaryPanel";
import { TranscriptView } from "./components/TranscriptView";
import type { Health, SummaryMode, SummaryResult, Transcript } from "./types";

function message(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function App() {
  const [health, setHealth] = useState<Health | null>(null);
  const [transcript, setTranscript] = useState<Transcript | null>(null);
  const [source, setSource] = useState("");
  const [transcribing, setTranscribing] = useState(false);
  const [transcribeError, setTranscribeError] = useState<string | null>(null);
  const [summary, setSummary] = useState<SummaryResult | null>(null);
  const [summarizing, setSummarizing] = useState(false);
  const [summaryError, setSummaryError] = useState<string | null>(null);

  useEffect(() => {
    fetchHealth()
      .then(setHealth)
      .catch((e) => setTranscribeError(message(e)));
  }, []);

  async function onAudio(audio: Blob, label: string) {
    setTranscribing(true);
    setTranscribeError(null);
    setSummary(null);
    setSummaryError(null);
    try {
      setTranscript(await transcribe(audio));
      setSource(label);
    } catch (e) {
      setTranscribeError(message(e));
    } finally {
      setTranscribing(false);
    }
  }

  async function onSummarize(mode: SummaryMode) {
    if (!transcript) return;
    setSummarizing(true);
    setSummaryError(null);
    try {
      setSummary(await summarize(transcript.text, mode));
    } catch (e) {
      setSummary(null);
      setSummaryError(message(e));
    } finally {
      setSummarizing(false);
    }
  }

  return (
    <main>
      <header>
        <h1>Speech to Text Summary</h1>
        <HealthBadges health={health} />
      </header>
      <AudioInput busy={transcribing} onAudio={onAudio} onError={setTranscribeError} />
      {transcribing && <p className="muted">transcribing with whisper.cpp...</p>}
      {transcribeError && <p className="error">{transcribeError}</p>}
      <TranscriptView transcript={transcript} source={source} />
      <SummaryPanel
        enabled={transcript !== null && transcript.text.length > 0}
        busy={summarizing}
        result={summary}
        error={summaryError}
        onRun={onSummarize}
      />
    </main>
  );
}
