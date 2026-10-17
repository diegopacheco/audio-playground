import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const frontendPort = Number(process.env.FRONTEND_PORT ?? 5108);
const backendPort = Number(process.env.BACKEND_PORT ?? 8108);

export default defineConfig({
  plugins: [react()],
  server: {
    port: frontendPort,
    strictPort: true,
    proxy: {
      "/api": `http://127.0.0.1:${backendPort}`,
    },
  },
});
