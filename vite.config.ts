import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
    plugins: [sveltekit()],
    clearScreen: false,
    server: {
        port: 5173,
        strictPort: true,
        watch: {
            ignored: ['**/src-tauri/**']
        }
    },
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
        target: 'esnext',
        minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
        sourcemap: !!process.env.TAURI_DEBUG
    }
});