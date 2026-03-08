# Calpad — Implementation Plan

Open-source natural language calculator inspired by Numi.
Web app + PWA + native CLI + embeddable Rust library.

## Architecture

```
core/
├── calpad-core/         # Rust library — parser, evaluator, unit system
├── calpad-wasm/         # WASM bindings (thin wrapper over core)
└── calpad-cli/          # Native CLI binary + stubbed TUI (ratatui later)
web/                     # SvelteKit frontend (Bun + Vite + vite-plugin-wasm)
├── src/lib/engine/      # WASM loader + TS interface
├── src/lib/components/  # Editor, Sidebar, ThemePicker
├── src/lib/storage/     # IndexedDB notes layer
└── src/routes/          # SvelteKit pages
```

Data flow: User types → Svelte sends full document to WASM → Rust parses all lines, resolves variables/prev/sum → Returns LineResult[] → Svelte renders results right-aligned.

## Phases

### Phase 1 — Arithmetic foundation ✅
- [x] Rust workspace scaffold
- [x] Number parsing: decimal, hex (0x), binary (0b), octal (0o)
- [x] All operators: +, -, *, /, ^, mod, &, |, xor, <<, >>
- [x] Word operators: plus, minus, times, multiplied by, divided by
- [x] Parentheses + implicit multiplication: 6(3) = 18
- [x] Math functions: sqrt, cbrt, root, log, ln, abs, round, ceil, floor, fact, trig, hyperbolic
- [x] Constants: pi, e
- [x] Variables: assignment + reuse across lines
- [x] Cross-line: prev, sum/total, average/avg (stop at blank line)
- [x] Document formatting: # headers, // comments, Label: prefix
- [x] Display formatting: integer/decimal detection, comma grouping, trailing zero trimming
- [x] WASM bindings (wasm-bindgen + serde-wasm-bindgen)
- [x] CLI: expression arg, stdin pipe, `tui` subcommand stubbed
- [x] Svelte frontend: editor with per-line results, sidebar with IndexedDB notes
- [x] 8 themes: monokai (default), dracula, nord, one-dark, catppuccin, gruvbox, solarized-dark, solarized-light
- [x] 21 integration tests passing

### Phase 2 — Units ✅
- [x] Unit registry in Rust: base units, conversion ratios, 100+ unit definitions
- [x] Length: meter, mil, point, line, inch, hand, foot, yard, rod, chain, furlong, mile, cable, nautical mile, league
- [x] Weight: gram, tonne, carat, centner, pound, stone, ounce
- [x] Area: hectare, are, acre + squared length units (sq, square prefix)
- [x] Volume: pint, quart, gallon, tea spoon, table spoon, cup + cubic length units (cu, cb, cubic prefix)
- [x] Data: bit, byte (1000-based), kibibyte etc (1024-based). Lowercase b = bits, uppercase B = bytes
- [x] Temperature: kelvin, celsius, fahrenheit (offset conversion, not ratio)
- [x] Angular: radians, degrees (+ ° sign)
- [x] Time/duration: second, minute, hour, day, week, month (365/12 days), year (365 days)
- [x] CSS: px, pt, em (configurable base), ppi (configurable, default 96)
- [x] Conversion syntax: `in`, `to`, `as`, `into`
- [x] Compound expressions: `1 meter 20 cm`
- [x] Format conversions: `in hex`, `in binary`, `in octal`, `in sci`/`in scientific`
- [x] Unit-aware arithmetic: add/sub with auto-conversion, scalar mul/div preserving units
- [x] 64 integration tests passing

### Phase 3 — Percentages & scales ✅
- [x] Basic: `10 - 40%`, `20% of 10`, `5% on 30`, `6% off 40`
- [x] Relative: `50 as a % of 100`, `70 as a % on 20`, `20 as a % off 70`
- [x] Inverse: `5% of what is 6`, `5% on what is 6`, `5% off what is 6`
- [x] Scales: k/thousand, M/million, billion (case-sensitive single-char)
- [x] Percentage-aware arithmetic: `Number +/- Percentage` applies percent to number
- [x] Unit-preserving: `5 kg + 10%` → `5.5 kg`
- [x] `Value::Percentage` type with `N%` display
- [x] 84 integration tests passing

### Phase 4 — Currency ✅
- [x] 26 fiat currencies with ISO codes, symbols, and natural language phrases
- [x] Currency prefix symbols: $, €, £, ¥, ₹, ₽, ₩, ₪, ฿ + multi-char C$, A$, R$, etc.
- [x] Code override: `$30 CAD` = 30 CAD (explicit code wins over $ symbol)
- [x] `evaluate_document_with_rates()` API for runtime exchange rate injection
- [x] Cross-currency arithmetic: addition/subtraction auto-converts via rates
- [x] Currency-aware sum/total/average aggregation
- [x] Prefix display for symbol currencies ($10, €20) vs suffix for codes (30 CAD)
- [x] WASM: `set_currency_rates()` function, rates stored in static Mutex
- [x] Web: fetches rates from frankfurter.app on init, passes to WASM
- [x] Works with scales ($2k) and percentages ($10 - 5%)
- [x] 97 integration tests passing
- [ ] Crypto via CoinGecko (deferred)

### Phase 5 — Dates & time ✅
- [x] `now` / `time` keywords → current datetime (UTC)
- [x] `fromunix(timestamp)` → formatted date
- [x] DateTime + duration: `now + 3 days`, `fromunix(0) + 1 year`
- [x] DateTime - duration: `now - 2 weeks`
- [x] DateTime - DateTime → duration in seconds
- [x] Display: `Mar 8, 2026 3:45:00 PM` (12-hour, month abbrev)
- [x] Civil date algorithm (Hinnant) — no external crate
- [x] Duration units already work from Phase 2: `round(1 month in days)` → 30
- [x] 109 integration tests passing
- [ ] Timezone support (deferred)
- [ ] Natural dates like `next friday` (deferred)

### Phase 6 — Polish ✅
- [x] Syntax highlighting: numbers, units, keywords, operators, labels, headers, comments
- [x] Shareable notes via URL fragment (#base64 encoded content, copy to clipboard)
- [x] PWA manifest (manifest.json, meta tags)
- [x] Keyboard shortcuts: Ctrl/Cmd+B toggle sidebar, Ctrl/Cmd+N new note
- [x] Sidebar delete with confirmation (double-click)
- [x] Sidebar open/close state persisted in localStorage
- [x] Improved welcome content showcasing all features
- [x] Scroll sync between textarea and highlight layer
- [ ] Service worker for offline (needs vite-plugin-pwa setup)
- [ ] PWA icons (need design)

### Phase 6.5 - TUI ✅
- [x] ratatui 0.30 + crossterm TUI frontend (`calpad-cli tui`)
- [x] Custom text editor: multi-line editing, cursor, scrolling, page up/down
- [x] Syntax highlighting: numbers, units, keywords, operators, labels, headers, comments
- [x] Real-time evaluation with right-aligned results (same core engine as web)
- [x] Sidebar: notes list with create/delete, keyboard navigation (Esc to toggle focus)
- [x] 8 themes matching web (monokai default, dracula, nord, one-dark, catppuccin, gruvbox, solarized-dark/light)
- [x] File-based persistence: ~/.local/share/calpad/ (notes.json, config.json, rates.json)
- [x] Currency rates: cached rates loaded instantly, fresh rates fetched in background thread
- [x] Keyboard shortcuts: ^B sidebar, ^N new, ^T cycle theme, ^Q quit, Esc toggle focus
- [x] Status bar with shortcut hints and rate status

### Phase 7 — Distribution
- [ ] CLI: GitHub Releases (linux-x64, linux-arm64, macos-x64, macos-arm64, windows-x64)
- [ ] `cargo install calpad`
- [ ] npm package: `@calpad/core` (WASM)
- [ ] AUR package, Homebrew tap, Nix flake
- [ ] CI: GitHub Actions for build + deploy
- [ ] Fetch crypto prices

### Future (parked)
- Plugin system via WASM modules
- Collaborative editing (CRDT-based)

## Tech stack

| Layer | Tool |
|-------|------|
| Parser | Rust + winnow (Pratt/precedence climbing) |
| WASM bridge | wasm-bindgen + serde-wasm-bindgen |
| WASM build | wasm-pack |
| Frontend | SvelteKit + Bun + Vite |
| Styling | Plain CSS with semantic custom properties |
| Storage | IndexedDB (idb via raw API) |
| CLI | clap |
| Currency rates | frankfurter.app (fiat), CoinGecko (crypto) |
| Static deploy | @sveltejs/adapter-static |

## Build commands

```sh
# WASM (must run before web)
cd core && wasm-pack build calpad-wasm --target web --out-dir ../../web/src/lib/engine/pkg

# Web dev
cd web && bun run dev

# Web build
cd web && bun run build

# CLI
cd core && cargo build --release --bin calpad-cli

# Tests
cd core && cargo test
```
