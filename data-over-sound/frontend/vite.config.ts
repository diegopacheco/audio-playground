import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";

function portFromEnvFile(name: string, fallback: number): number {
  try {
    const text = readFileSync(new URL("../scripts/ports.env", import.meta.url), "utf8");
    const line = text.split("\n").find((l) => l.trim().startsWith(`${name}=`));
    return line ? Number(line.split("=")[1]) : fallback;
  } catch {
    return fallback;
  }
}

const frontendPort = Number(process.env.FRONTEND_PORT ?? portFromEnvFile("frontend", 5111));
const backendPort = Number(process.env.BACKEND_PORT ?? portFromEnvFile("backend", 8111));

export default defineConfig({
  plugins: [react()],
  server: {
    port: frontendPort,
    strictPort: true,
    proxy: {
      "/api": { target: `http://127.0.0.1:${backendPort}`, ws: true },
    },
  },
});
