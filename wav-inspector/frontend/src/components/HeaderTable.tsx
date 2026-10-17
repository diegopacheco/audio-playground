import type { Report } from "../types";
import { formatDuration, toDbfs } from "../audio/format";

interface Props {
  report: Report;
}

export function HeaderTable({ report }: Props) {
  const h = report.header;
  const rows: [string, string][] = [
    ["Encoding", h.encoding],
    ["Format tag", `0x${h.format_tag.toString(16).padStart(4, "0").toUpperCase()}`],
    ["Sample rate", `${h.sample_rate} Hz`],
    ["Channels", h.channels === 1 ? "1 (mono)" : `${h.channels} (stereo)`],
    ["Bit depth", `${h.bits_per_sample}`],
    ["Byte rate", `${h.byte_rate} B/s`],
    ["Block align", `${h.block_align} B`],
    ["Frames", `${h.frames}`],
    ["Duration", formatDuration(report.duration)],
    ["Peak", `${report.peak.toFixed(4)}  (${toDbfs(report.peak)})`],
    ["RMS", `${report.rms.toFixed(4)}  (${toDbfs(report.rms)})`],
  ];
  return (
    <section className="card">
      <h2>Header</h2>
      <table>
        <tbody>
          {rows.map(([k, v]) => (
            <tr key={k}>
              <th>{k}</th>
              <td>{v}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
