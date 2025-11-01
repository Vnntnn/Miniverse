// @ts-check
import { defineConfig } from 'astro/config';

export default defineConfig({
  output: 'static',
  build: {
    assets: '_astro'
  },
  vite: {
    optimizeDeps: {
      include: ['xterm']
    }
  }
});
