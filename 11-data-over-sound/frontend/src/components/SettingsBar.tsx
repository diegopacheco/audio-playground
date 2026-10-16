import type { ModemSettings, Scheme, SchemeInfo } from "../types";

interface Props {
  settings: ModemSettings;
  schemes: SchemeInfo[];
  onChange: (settings: ModemSettings) => void;
}

const BAUDS = [100, 150, 300, 600];

export function SettingsBar({ settings, schemes, onChange }: Props) {
  const info = schemes.find((s) => s.scheme === settings.scheme);
  return (
    <div className="settings card">
      <label>
        Scheme
        <select
          value={settings.scheme}
          onChange={(e) => onChange({ ...settings, scheme: e.target.value as Scheme })}
        >
          <option value="bfsk">BFSK (Bell 202 tones)</option>
          <option value="4fsk">4-FSK (2 bits/symbol)</option>
        </select>
      </label>
      <label>
        Baud
        <select
          value={settings.baud}
          onChange={(e) => onChange({ ...settings, baud: Number(e.target.value) })}
        >
          {BAUDS.map((b) => (
            <option key={b} value={b}>
              {b}
            </option>
          ))}
        </select>
      </label>
      {info && (
        <div className="tones">
          tones {info.tones.map((t) => `${t} Hz`).join(" / ")}
          <span className="muted">
            {" "}
            · {settings.baud * info.bits_per_symbol} bit/s raw
          </span>
        </div>
      )}
    </div>
  );
}
