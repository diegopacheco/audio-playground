import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";

function readPorts(): Record<string, number> {
  const text = readFileSync(new URL("../scripts/ports.env", import.meta.url), "utf8");
  const ports: Record<string, number> = {};
  for (const line of text.split("\n")) {
    const [name, value] = line.split("=");
    if (name && value) {
      ports[name.trim()] = Number(value.trim());
    }
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
      "/api": `http://localhost:${ports.backend}`,
    },
  },
});
