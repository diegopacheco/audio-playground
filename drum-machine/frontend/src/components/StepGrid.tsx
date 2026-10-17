import { INSTRUMENTS, STEPS } from "../pattern";
import type { InstrumentName, Pattern } from "../types";

interface Props {
  pattern: Pattern;
  currentStep: number;
  onToggle: (name: InstrumentName, step: number) => void;
}

const steps = Array.from({ length: STEPS }, (_, i) => i);

export function StepGrid({ pattern, currentStep, onToggle }: Props) {
  return (
    <div className="grid">
      {INSTRUMENTS.map((name) => (
        <div className="row" key={name}>
          <span className={`label ${name}`}>{name}</span>
          {steps.map((step) => {
            const on = pattern.tracks[name][step];
            const classes = [
              "cell",
              name,
              on ? "on" : "",
              step === currentStep ? "now" : "",
              step % 4 === 0 ? "beat" : "",
            ].join(" ");
            return (
              <button
                key={step}
                className={classes}
                aria-label={`${name} step ${step + 1}`}
                aria-pressed={on}
                onClick={() => onToggle(name, step)}
              />
            );
          })}
        </div>
      ))}
    </div>
  );
}
