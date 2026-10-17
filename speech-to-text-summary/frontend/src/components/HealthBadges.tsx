import type { Health } from "../types";

interface Props {
  health: Health | null;
}

const LABELS: { key: keyof Health; label: string }[] = [
  { key: "whisper", label: "whisper-cli" },
  { key: "model", label: "ggml model" },
  { key: "claude_key", label: "Claude key" },
];

export function HealthBadges({ health }: Props) {
  return (
    <div className="badges">
      {LABELS.map(({ key, label }) => {
        const state = health === null ? "unknown" : health[key] ? "ok" : "missing";
        return (
          <span key={key} className={`badge ${state}`}>
            {label}: {state}
          </span>
        );
      })}
    </div>
  );
}
