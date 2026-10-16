import { hex, logBytes } from "../audio/frame";
import type { ByteEvent } from "../types";

interface Props {
  bytes: ByteEvent[];
}

function printable(value: number): string {
  return value >= 32 && value < 127 ? String.fromCharCode(value) : "·";
}

export function DecodeLog({ bytes }: Props) {
  const rows = logBytes(bytes);
  return (
    <div className="log">
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>role</th>
            <th>hex</th>
            <th>char</th>
            <th>uart bits (start, d0..d7, stop)</th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 && (
            <tr>
              <td colSpan={5} className="muted">
                no bytes yet
              </td>
            </tr>
          )}
          {rows.map((b) => (
            <tr key={b.index} className={b.framing_ok ? `role-${b.role}` : "framing-error"}>
              <td>{b.index}</td>
              <td>{b.role}</td>
              <td>0x{hex(b.value)}</td>
              <td>{printable(b.value)}</td>
              <td className="bits">{b.bits}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
