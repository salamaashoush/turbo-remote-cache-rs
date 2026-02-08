import path from 'path';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [tailwindcss(), react()],
  resolve: {
    alias: {
      '~': path.resolve(__dirname, './src'),
    },
  },
  envPrefix: ['VITE_', 'SUPER_ADMIN_'],
  server: {
    proxy: {
      '/api': 'http://localhost:4000',
      '/v8': 'http://localhost:4000',
    },
  },
});
