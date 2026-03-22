/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'Alexandria', 'IBM Plex Sans Arabic', 'sans-serif'],
        arabic: ['Alexandria', 'IBM Plex Sans Arabic', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [require("daisyui"), require("@tailwindcss/typography")],
  daisyui: {
    themes: [{
      actuators: {
        "primary": "#00B7C2",
        "secondary": "#FF6A2A",
        "accent": "#0d9488",
        "neutral": "#0f2035",
        "base-100": "#0B1A2A",
        "base-200": "#0f2035",
        "base-300": "#162a40",
        "base-content": "#e2e8f0",
        "info": "#00B7C2",
        "success": "#16a34a",
        "warning": "#f59e0b",
        "error": "#dc2626",
      },
    }],
    darkTheme: "actuators",
  },
};
