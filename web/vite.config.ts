import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from '@tailwindcss/vite'
import tsconfigPaths from 'vite-tsconfig-paths'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
	plugins: [
    	tailwindcss(),
		tsconfigPaths(),
    	react()
	],
	clearScreen: false,
  
  	server: {
    	port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
		? {
			protocol: "ws",
			host,
			port: 1421,
			}
		: undefined,
		watch: {
			ignored: ["**/src-tauri/**"],
    	},
	},
}));
