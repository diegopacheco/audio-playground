import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const frontendPort = Number(process.env.FRONTEND_PORT ?? 5109);
const backendPort = Number(process.env.BACKEND_PORT ?? 8109);

export default defineConfig({
  plugins: [react()],
  server: {
    port: frontendPort,
    strictPort: true,
    proxy: {
      "/api": `http://localhost:${backendPort}`,
    },
  },
});
