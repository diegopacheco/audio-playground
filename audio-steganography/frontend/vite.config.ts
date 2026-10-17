import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

function readPort(name: string): number {
  const file = fileURLToPath(new URL("../scripts/ports.env", import.meta.url));
  const line = readFileSync(file, "utf8")
    .split("\n")
    .find((l) => l.startsWith(`${name}=`));
  const port = Number(line?.split("=")[1]);
  if (!Number.isInteger(port)) {
    throw new Error(`port for ${name} missing in scripts/ports.env`);
  }
  return port;
}

export default defineConfig({
  plugins: [react()],
  server: {
    port: readPort("frontend"),
    strictPort: true,
    proxy: {
      "/api": `http://127.0.0.1:${readPort("backend")}`,
    },
  },
});
