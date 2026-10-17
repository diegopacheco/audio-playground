import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

function readPorts(): Record<string, number> {
  const file = fileURLToPath(new URL("../scripts/ports.env", import.meta.url));
  const ports: Record<string, number> = {};
  for (const line of readFileSync(file, "utf8").split("\n")) {
    const [name, value] = line.split("=");
    if (name && value) ports[name.trim()] = Number(value.trim());
  }
  return ports;
}

const ports = readPorts();

export default defineConfig({
  plugins: [react()],
  server: {
    port: ports.frontend,
    strictPort: true,
    proxy: {
      "/api": {
        target: `http://localhost:${ports.backend}`,
        ws: true,
      },
    },
  },
});
