import type { EffectSpec, VoiceSettings } from "../types";

interface Props {
  effects: EffectSpec[];
  settings: VoiceSettings;
  onChange: (settings: VoiceSettings) => void;
}

function formatSemitones(value: number): string {
  return value > 0 ? `+${value}` : String(value);
}

export function EffectControls({ effects, settings, onChange }: Props) {
  const spec = effects.find((effect) => effect.name === settings.effect);
  const fixed = !spec || spec.minSemitones === spec.maxSemitones;

  function selectEffect(name: string) {
    const next = effects.find((effect) => effect.name === name);
    if (next) {
      onChange({ effect: next.name, semitones: next.defaultSemitones });
    }
  }

  return (
    <section className="card">
      <h2>Effect</h2>
      <div className="effect-tabs" role="tablist">
        {effects.map((effect) => (
          <button
            key={effect.name}
            role="tab"
            aria-selected={effect.name === settings.effect}
            className={effect.name === settings.effect ? "tab tab-active" : "tab"}
            onClick={() => selectEffect(effect.name)}
          >
            {effect.label}
          </button>
        ))}
      </div>
      <label className="slider">
        <span>
          {settings.effect === "robot" ? "Robot pitch" : "Semitones"}
          <strong>{fixed ? "n/a" : formatSemitones(settings.semitones)}</strong>
        </span>
        <input
          type="range"
          min={spec?.minSemitones ?? 0}
          max={spec?.maxSemitones ?? 0}
          step={1}
          value={settings.semitones}
          disabled={fixed}
          onChange={(event) => onChange({ ...settings, semitones: Number(event.target.value) })}
        />
      </label>
    </section>
  );
}
