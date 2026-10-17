import type { Method, MethodConfig } from "../types";

interface Props {
  value: MethodConfig;
  onChange: (value: MethodConfig) => void;
}

const METHODS: { id: Method; label: string; hint: string }[] = [
  { id: "lsb", label: "LSB", hint: "hide bits in the lowest bits of each 16-bit sample" },
  { id: "ultrasonic", label: "Ultrasonic FSK", hint: "18.2 / 19.2 kHz tones mixed at low level" },
];

export function MethodPicker({ value, onChange }: Props) {
  const active = METHODS.find((m) => m.id === value.method);
  return (
    <div className="method">
      <div className="chips">
        {METHODS.map((m) => (
          <button
            key={m.id}
            type="button"
            className={m.id === value.method ? "chip active" : "chip"}
            onClick={() => onChange({ ...value, method: m.id })}
          >
            {m.label}
          </button>
        ))}
        {value.method === "lsb" && (
          <label className="bits">
            bits per sample
            <select
              value={value.bits}
              onChange={(e) => onChange({ ...value, bits: Number(e.target.value) })}
            >
              {[1, 2, 3, 4].map((b) => (
                <option key={b} value={b}>
                  {b}
                </option>
              ))}
            </select>
          </label>
        )}
      </div>
      <p className="muted">{active?.hint}</p>
    </div>
  );
}
