/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,ts}",
  ],
  theme: {
    extend: {
      fontFamily: {
        'sans': ['IBM Plex Sans', 'system-ui', 'sans-serif'],
        'mono': ['IBM Plex Mono', 'Menlo', 'monospace'],
        'stencil': ['Black Ops One', 'Impact', 'sans-serif'],
      },
      colors: {
        // Military camouflage palette
        'camo': {
          'olive': '#4A5D23',      // Olive drab
          'forest': '#2D4022',     // Dark forest green
          'tan': '#C4A35A',        // Desert tan
          'brown': '#5C4033',      // Earth brown
          'khaki': '#8B7355',      // Khaki
          'sand': '#D4C5A9',       // Sand
          'mud': '#3D2B1F',        // Dark mud
          'moss': '#556B2F',       // Dark olive green
        },
        'army': {
          50: '#f5f5e8',
          100: '#e8e6d5',
          200: '#d4d0b8',
          300: '#b8b08f',
          400: '#9a8f6a',
          500: '#7d7350',
          600: '#5c5338',
          700: '#4a4530',
          800: '#3d3928',
          900: '#2d2b1f',
          950: '#1a1910',
        },
      },
      backgroundImage: {
        'camo-pattern': `
          radial-gradient(ellipse at 20% 30%, #4A5D23 0%, transparent 50%),
          radial-gradient(ellipse at 80% 20%, #2D4022 0%, transparent 45%),
          radial-gradient(ellipse at 40% 70%, #5C4033 0%, transparent 50%),
          radial-gradient(ellipse at 70% 60%, #556B2F 0%, transparent 45%),
          radial-gradient(ellipse at 10% 80%, #3D2B1F 0%, transparent 40%),
          radial-gradient(ellipse at 90% 90%, #4A5D23 0%, transparent 50%)
        `,
        'camo-subtle': `
          radial-gradient(ellipse at 15% 25%, rgba(74, 93, 35, 0.15) 0%, transparent 50%),
          radial-gradient(ellipse at 85% 15%, rgba(45, 64, 34, 0.12) 0%, transparent 45%),
          radial-gradient(ellipse at 35% 75%, rgba(92, 64, 51, 0.1) 0%, transparent 50%),
          radial-gradient(ellipse at 65% 55%, rgba(85, 107, 47, 0.12) 0%, transparent 45%)
        `,
      },
      boxShadow: {
        'military': '0 4px 6px -1px rgba(45, 43, 31, 0.3), 0 2px 4px -2px rgba(45, 43, 31, 0.2)',
      },
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
  ],
}
