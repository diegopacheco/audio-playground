import type { Chunk } from "../types";
import { formatBytes } from "../audio/format";

interface Props {
  chunks: Chunk[];
}

export function ChunkList({ chunks }: Props) {
  return (
    <section className="card">
      <h2>Chunks</h2>
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>Offset</th>
            <th>Size</th>
          </tr>
        </thead>
        <tbody>
          {chunks.map((c) => (
            <tr key={c.offset}>
              <td>
                <code>'{c.id}'</code>
              </td>
              <td>{c.offset}</td>
              <td>{formatBytes(c.size)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
