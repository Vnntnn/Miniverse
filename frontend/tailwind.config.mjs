/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
  theme: {
    extend: {
      colors: {
        cyber: {
          green: '#00ff41',
          blue: '#00d4ff', 
          purple: '#b537f2',
        },
        terminal: {
          bg: '#0a0a0a',
          surface: '#111111',
          border: '#333333',
          text: '#00ff41',
          muted: '#666666',
          prompt: '#00d4ff',
          error: '#ff0080',
        }
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Monaco', 'Consolas', 'monospace'],
      },
      animation: {
        'scan-line': 'scanline 2s linear infinite',
        'glitch': 'glitch 0.3s ease-in-out infinite alternate',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        scanline: {
          '0%': { transform: 'translateY(-100%)' },
          '100%': { transform: 'translateY(100vh)' }
        },
        glitch: {
          '0%': { textShadow: '0.05em 0 0 #00fffc, -0.03em -0.04em 0 #fc00ff, 0.025em 0.04em 0 #fffc00' },
          '100%': { textShadow: '-0.05em 0 0 #00fffc, -0.025em -0.04em 0 #fc00ff, -0.04em -0.025em 0 #fffc00' }
        }
      }
    }
  },
  plugins: []
}
