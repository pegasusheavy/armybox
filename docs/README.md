# armybox Documentation Website

A minimal, professional documentation website for armybox built with Angular and Tailwind CSS.

## Tech Stack

- **Angular 21** - Modern web framework
- **Tailwind CSS 3** - Utility-first CSS
- **IBM Plex** - Typography (Sans + Mono)

## Development

### Prerequisites

- Node.js 18+
- npm (or pnpm)

### Install dependencies

```bash
npm install
# or
pnpm install
```

### Development server

```bash
npm start
# or
npm run start
```

Navigate to `http://localhost:4200/`. The app will automatically reload on file changes.

### Build

```bash
npm run build
```

Build artifacts are stored in `dist/armybox-docs/browser/`.

## Project Structure

```
docs/
├── src/
│   ├── app/
│   │   ├── pages/
│   │   │   ├── home/         # Landing page
│   │   │   ├── applets/      # Applet reference
│   │   │   ├── building/     # Build instructions
│   │   │   └── api/          # API reference
│   │   ├── app.ts            # Root component with layout
│   │   ├── app.routes.ts     # Routing configuration
│   │   └── app.config.ts     # App configuration
│   ├── styles.css            # Global styles with Tailwind
│   └── index.html            # HTML entry point
├── tailwind.config.js        # Tailwind configuration
├── postcss.config.js         # PostCSS configuration
└── angular.json              # Angular CLI configuration
```

## Design System

### Colors

The documentation uses a custom "army" color palette:

- `army-50` to `army-950` - Olive/military green scale
- Used for text, backgrounds, and accents

### Typography

- **Headings**: IBM Plex Sans (600-700 weight)
- **Body**: IBM Plex Sans (400-500 weight)
- **Code**: IBM Plex Mono (400-600 weight)

### Components

Custom Tailwind components defined in `styles.css`:

- `.nav-link` - Navigation links with hover states
- `.btn`, `.btn-primary`, `.btn-secondary` - Button variants
- `.code-block` - Styled code blocks
- `.prose-army` - Typography plugin customization

## Deployment

The built site is static HTML/CSS/JS and can be deployed to any static hosting:

```bash
# Build for production
npm run build

# Deploy dist/armybox-docs/browser/ to your hosting
```

### GitHub Pages

```bash
# Build with base href for GitHub Pages
ng build --base-href /armybox/
```

## License

MIT / Apache-2.0
