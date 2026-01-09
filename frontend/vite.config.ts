import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";

export default defineConfig({
  server: {
    host: "0.0.0.0",
    port: 9000,
    hmr: {
      protocol: "ws",
    },
  },
  resolve: {
    alias: {
      "~bootstrap": path.resolve(__dirname, "node_modules/bootstrap"),
    },
  },
  plugins: [react({})],
  build: {
    sourcemap: true,
  },
});
