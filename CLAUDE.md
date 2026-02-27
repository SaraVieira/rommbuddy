# RoMM Buddy

A Tauri 2.0 desktop app for managing, cataloging, and playing ROM files from multiple sources.

## Tech Stack

- **Frontend**: React 18 + TypeScript 5.6 + Vite 6.4 + Tailwind CSS 4.2 + shadcn/ui
- **Backend**: Rust (Tauri 2.0, tokio async runtime, SQLx + SQLite WAL mode)
- **State**: Jotai atoms for global filters/view state, React context for toast/sync
- **Package Manager**: pnpm

## Commands

```bash
pnpm dev              # Vite dev server (port 1420)
pnpm build            # TypeScript check + Vite build
pnpm tauri dev        # Full Tauri dev (runs pnpm dev + opens native window)
pnpm tauri build      # Production build (native binary)
npx tsc --noEmit      # Type check without emitting
```

## Project Structure

```
src/                          # React frontend
├── App.tsx                   # Main layout: sidebar (240px) + content area
├── main.tsx                  # React Router v7 setup
├── types.ts                  # Frontend TypeScript interfaces
├── index.css                 # Tailwind + design system tokens (THE source of truth for theme)
├── pages/                    # Route pages
│   ├── Library.tsx           # Main ROM library (grid/list + search + platform filter)
│   ├── Platforms.tsx         # Platform browser
│   ├── Search.tsx            # Global search (text + platform filter)
│   ├── Sources.tsx           # Source configuration (Local, ROMM)
│   ├── Settings.tsx          # App settings (emulators, metadata, RetroAchievements)
│   └── RomDetailPage.tsx     # Individual ROM detail view
├── components/
│   ├── RomCard.tsx           # ROM card with cover art
│   ├── RomGrid.tsx           # Virtualized grid (@tanstack/react-virtual)
│   ├── RomList.tsx           # List view
│   ├── PlatformFilter.tsx    # Searchable combobox (cmdk + radix popover)
│   ├── ViewToggle.tsx        # Grid/list toggle
│   ├── Pagination.tsx        # Offset-based pagination
│   ├── sources/              # Source-specific UI (Local, ROMM sections)
│   ├── Achievements/         # RetroAchievements UI
│   └── ui/                   # shadcn components (button, dialog, command, popover)
├── hooks/                    # useSyncState, useToast, useProxiedImage
├── store/library.ts          # Jotai atoms (search, filters, view mode, offset)
├── lib/utils.ts              # cn() utility (clsx + tailwind-merge)
└── utils/platformIcons.ts    # Platform icon mappings

src-tauri/src/                # Rust backend
├── lib.rs                    # Tauri app builder + plugin init
├── commands.rs               # 100+ IPC commands (library, sources, emulators, metadata)
├── models.rs                 # Data structures (Platform, RomEntry, Source, etc.)
├── db.rs                     # SQLite pool + migration runner
├── sources/                  # Pluggable ROM sources
│   ├── local.rs / local_sync.rs  # Local filesystem scanner + file watcher
│   └── romm.rs               # ROMM REST API client
├── metadata/                 # Metadata enrichment backends
│   ├── igdb.rs               # IGDB API
│   ├── screenscraper.rs      # ScreenScraper API
│   ├── libretro_thumbnails.rs # LibRetro artwork
│   └── ...                   # launchbox, hasheous, dat
└── hash.rs, dedup.rs, retroachievements.rs, error.rs

migrations/                   # 13 SQLite migration files (001-013)
```

## Design System

**Theme**: Industrial Terminal — dark blacks + neon green, sharp corners (radius: 0)

### Key Tokens (defined in `src/index.css` `@theme` block)
- **Backgrounds**: `bg-page` (#0C0C0C), `bg-sidebar` (#080808), `bg-card` (#0A0A0A), `bg-elevated` (#141414)
- **Accent**: `accent` (#00FF88 neon green), tints at 40/20/10 opacity
- **Text**: `text-primary` (#FFF), `text-secondary` (#8a8a8a), `text-muted` (#6a6a6a), `text-dim` (#4a4a4a)
- **Borders**: `border` (#2f2f2f), `border-light` (#3f3f3f)
- **Fonts**: `font-mono` (JetBrains Mono), `font-display` (Space Grotesk for headlines)
- **Spacing**: xs(2) sm(4) md(8) lg(12) xl(16) 2xl(20) 3xl(24) 4xl(28) 5xl(32) 6xl(40) 7xl(48)

### CSS Classes (in `@layer components`)
- `.btn`, `.btn-primary`, `.btn-secondary`, `.btn-danger`, `.btn-sm` — all uppercase, mono, sharp corners
- `.card` — bg-card, border, padding 3xl, radius 0
- `.form-group` — label + input styling with accent focus
- `.status-ok` / `.status-missing` — green/red bracket status indicators

### shadcn Integration
- shadcn CSS variables are mapped to design system colors in `:root` block at bottom of `index.css`
- `--radius: 0` ensures all shadcn components have sharp corners
- Components: button, dialog, command, popover (in `src/components/ui/`)
- Path alias: `@/` → `./src/` (configured in tsconfig.json + vite.config.ts)

## Architecture Notes

### Frontend-Backend Communication
- All IPC via `invoke('command_name', { args })` from `@tauri-apps/api/core`
- Streaming progress uses Tauri `Channel<ScanProgress>` for sync operations
- Cancellation via `CancelTokenMap` in Rust backend

### ROM Grid Virtualization
- `RomGrid` splits into parent + child `VirtualGrid` component
- `VirtualGrid` uses a `key` prop that changes when roms change, forcing remount to fix stale virtualizer measurements
- Grid gap: 16px (`gap-xl`), scroll height: `calc(100vh - 260px)`

### Two Source Types
1. **Local**: Filesystem scanning, optional file watching, lazy hash computation
2. **ROMM**: REST API client with OAuth2 bearer auth

### Database
- SQLite with WAL mode for concurrent reads during sync
- 13 progressive migrations in `/migrations/`
- Key tables: platforms, roms, sources, rom_sources, metadata_artwork, igdb_cache

## Conventions

- All UI text is uppercase for labels/nav/buttons (industrial terminal aesthetic)
- Font imports via Google Fonts `<link>` tags in `index.html`
- Use design token CSS variables (`var(--color-*)`, `var(--spacing-*)`) — never raw hex values
- Use Tailwind token classes (`bg-bg-elevated`, `text-text-muted`, `border-border`) in JSX
- All border-radius: 0 (sharp corners everywhere)
- Main content padding: `py-5xl px-6xl` (32px vertical, 40px horizontal)
- Page headers: `font-display`, `text-page-title`, bold, uppercase + muted subtitle below
