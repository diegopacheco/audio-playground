import type { DeviceInfo } from "../types";

interface Props {
  label: string;
  devices: DeviceInfo[];
  value: string;
  disabled: boolean;
  onChange: (id: string) => void;
}

export function DevicePicker({ label, devices, value, disabled, onChange }: Props) {
  return (
    <label className="field">
      <span>{label}</span>
      <select value={value} disabled={disabled} onChange={(e) => onChange(e.target.value)}>
        {devices.map((d) => (
          <option key={d.id} value={d.id}>
            {d.name} ({d.backend})
          </option>
        ))}
      </select>
    </label>
  );
}
