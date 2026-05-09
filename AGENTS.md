# AGENTS.md

## Project

PetPop is a Windows-first Tauri 2 + Svelte desktop runtime for Codex-compatible pets. It imports pets produced by Hatch Pet, installed in Codex, or published on PetDex, then renders them as a transparent always-on-top desktop companion.

## Core Requirements

- Keep the app lightweight. Prefer Tauri/Rust/Svelte primitives over Electron-style heavy runtime assumptions.
- Do not bundle third-party PetDex pets into the app. Users must explicitly import them.
- Do not modify `~/.codex/pets` or repository sample pets during normal app operation. Imported pets belong under `%APPDATA%/PetPop/pets/`.
- Preserve compatibility with the Codex pet contract:
  - atlas size: `1536x1872`
  - grid: `8` columns x `9` rows
  - cell size: `192x208`
  - metadata: `pet.json` with `id`, `displayName`, `description`, `spritesheetPath`
- Keep `pet.json` Codex-compatible. Store PetPop-only metadata in `petpop.pet.json`.

## Architecture

- Frontend lives in `src/`.
  - `src/lib/animations.ts` owns the Codex row/frame/duration table.
  - `src/lib/petpop.ts` owns Tauri command wrappers and browser-preview fallbacks.
  - `src/components/ControlApp.svelte` is the control panel.
  - `src/components/PetWindow.svelte` is the transparent always-on-top runtime pet window.
  - `src/components/SpritePet.svelte` renders one atlas frame by CSS background positioning.
- Tauri/Rust lives in `src-tauri/`.
  - `src-tauri/src/main.rs` owns import, validation, PetDex download, runtime state, and persisted window position.
  - `src-tauri/tauri.conf.json` defines the main control window and transparent pet window.
  - `src-tauri/capabilities/default.json` must include permissions for any JS-side Tauri APIs used.

## Runtime Behavior

- The `main` window is a normal control panel.
- The `pet` window must stay transparent, undecorated, always-on-top, and skip the taskbar.
- The pet window must use native `startDragging()` for dragging.
- While dragging the pet window:
  - movement to the right maps to `running-right`
  - movement to the left maps to `running-left`
  - release maps back to `idle`
  - release persists the window position for the next launch
- Local sprites are loaded through the Rust command `get_pet_sprite_data_url` so the WebView does not depend on direct filesystem asset permissions.

## Pet Import Rules

- Local import supports `.zip` packages and folders.
- Codex scan imports valid pets from `%USERPROFILE%/.codex/pets`.
- PetDex import should resolve pets through `https://petdex.crafter.run/api/manifest` and use the returned `petJsonUrl` and `spritesheetUrl`.
- During import, validate spritesheet dimensions with Rust image decoding.
- During normal listing, do not repeatedly decode spritesheets. Listing should be cheap and metadata-oriented.

## Animation Mapping

Use the fixed Codex rows:

- `idle`: default calm state
- `running-right`: horizontal movement to the right
- `running-left`: horizontal movement to the left
- `waving`: greeting / click / drag start
- `jumping`: success / double click
- `failed`: import or task error
- `waiting`: long inactivity
- `running`: active task
- `review`: focus/review mode

Do not rename states without updating both the frontend animation table and any Rust/runtime scene calls.

## Development Commands

```powershell
npm install
npm run check
npm test
npm run build
npm run tauri -- build --debug
```

For desktop development:

```powershell
npm run tauri:dev
```

If `petpop.exe` is already running, stop it before rebuilding on Windows because the executable file will be locked:

```powershell
Get-Process petpop -ErrorAction SilentlyContinue | Stop-Process -Force
```

## Verification Expectations

Before finishing app changes, run:

- `npm run check`
- `npm test`
- `npm run tauri -- build --debug` when Rust/Tauri code, window config, capabilities, or packaging changes

For pet-window changes, also manually verify:

- no visible opaque background behind the pet
- the window is draggable
- drag direction switches to `running-left` / `running-right`
- position persists after restart

## Repository Hygiene

- Do not commit `node_modules/`, `dist/`, `src-tauri/target/`, or `src-tauri/gen/`.
- Commit `src-tauri/Cargo.lock` for reproducible desktop builds.
- Treat untracked pet zip files as user assets unless explicitly told to commit them.
- Keep generated sample/output images out of the repo unless they are intentional documentation assets.
