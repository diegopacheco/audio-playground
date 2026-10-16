import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const frontend = Number(env.FRONTEND_PORT ?? 5102);
  const backend = Number(env.BACKEND_PORT ?? 8102);
  return {
    plugins: [react()],
    server: {
      port: frontend,
      strictPort: true,
      proxy: {
        "/api": `http://127.0.0.1:${backend}`,
      },
    },
  };
});
