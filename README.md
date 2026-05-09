# PetDesk

PetDesk is a lightweight Windows-first desktop runtime for Codex-compatible pets.

## MVP

- Tauri 2 desktop shell with a transparent always-on-top pet window.
- Svelte control surface for pet import, selection, preview, scale, and state switching.
- Compatible with Hatch Pet, Codex custom pets, and PetDex packages that contain `pet.json` and `spritesheet.webp`.
- Stores imported pets under `%APPDATA%/PetDesk/pets/` and leaves `~/.codex/pets` unchanged.

## Development

```powershell
npm install
npm run dev
```

To run the desktop app, install Rust and the Tauri prerequisites, then run:

```powershell
npm run tauri:dev
```

## Package Format

PetDesk validates the Codex atlas contract:

- 8 columns x 9 rows.
- 192 x 208 pixels per cell.
- 1536 x 1872 pixels total.
- `pet.json` with `id`, `displayName`, `description`, and `spritesheetPath`.

PetDesk writes its own `petdesk.pet.json` next to imported pets for source and display metadata.
