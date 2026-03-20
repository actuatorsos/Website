/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {
      colors: {
        'cyan-struct': '#00B7C2',
        'orange-active': '#FF6A2A',
      },
      fontFamily: {
        'inter': ['Inter', 'sans-serif'],
        'arabic': ['IBM Plex Sans Arabic', 'sans-serif'],
        'mono': ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [require("daisyui")],
}
