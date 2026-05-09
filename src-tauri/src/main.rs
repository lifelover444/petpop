#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Cursor},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, State,
};
use zip::ZipArchive;

const CELL_WIDTH: u32 = 192;
const CELL_HEIGHT: u32 = 208;
const ATLAS_WIDTH: u32 = CELL_WIDTH * 8;
const ATLAS_HEIGHT: u32 = CELL_HEIGHT * 9;
const PETDEX_RAW_BASE: &str =
    "https://raw.githubusercontent.com/crafter-station/petdex/main/public/pets";
const PETDEX_MANIFEST_URL: &str = "https://petdex.crafter.run/api/manifest";
const APP_DATA_DIR_NAME: &str = "PetPop";
const LEGACY_APP_DATA_DIR_NAME: &str = "PetDesk";
const PETPOP_METADATA_FILE: &str = "petpop.pet.json";
const TITLECASE_METADATA_FILE: &str = "PetPop.pet.json";
const LEGACY_METADATA_FILE: &str = "petdesk.pet.json";

#[derive(Default)]
struct AppState {
    runtime: Mutex<RuntimeState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeState {
    active_pet_id: Option<String>,
    scene: String,
    scale: f32,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            active_pet_id: None,
            scene: "idle".to_string(),
            scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetInfo {
    id: String,
    display_name: String,
    description: String,
    spritesheet_path: String,
    source_kind: String,
    source_url: Option<String>,
    scale: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexPetJson {
    id: String,
    display_name: String,
    description: Option<String>,
    spritesheet_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetPopMetadata {
    schema_version: u8,
    source: PetSource,
    scale: f32,
    position: PetPosition,
    imported_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetSource {
    kind: String,
    url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PetPosition {
    x: i32,
    y: i32,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "Show PetPop", true, None::<&str>)?;
            let hide_pet = MenuItem::with_id(app, "hide_pet", "Hide Pet", true, None::<&str>)?;
            let show_pet = MenuItem::with_id(app, "show_pet", "Show Pet", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &show_pet, &hide_pet, &quit])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "show_pet" => {
                        if let Some(window) = app.get_webview_window("pet") {
                            let _ = window.show();
                        }
                    }
                    "hide_pet" => {
                        if let Some(window) = app.get_webview_window("pet") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_pets,
            import_pet_from_path,
            import_petdex,
            scan_codex_pets,
            get_pet_sprite_data_url,
            get_pet_window_position,
            set_pet_window_position,
            get_runtime_state,
            set_active_pet,
            set_scene,
            set_scale
        ])
        .run(tauri::generate_context!())
        .expect("error while running PetPop");
}

#[tauri::command]
fn list_pets() -> Result<Vec<PetInfo>, String> {
    let pets_dir = pets_dir()?;
    if !pets_dir.exists() {
        fs::create_dir_all(&pets_dir).map_err(to_string)?;
    }

    let mut pets = Vec::new();
    for entry in fs::read_dir(&pets_dir).map_err(to_string)? {
        let path = entry.map_err(to_string)?.path();
        if path.is_dir() {
            if let Ok(pet) = read_pet_dir_info(&path) {
                pets.push(pet);
            }
        }
    }

    pets.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    Ok(pets)
}

#[tauri::command]
fn import_pet_from_path(path: String) -> Result<PetInfo, String> {
    let source = PathBuf::from(path);
    if !source.exists() {
        return Err("Package path does not exist.".to_string());
    }

    import_pet_source(&source, "local", None)
}

#[tauri::command]
fn import_petdex(input: String) -> Result<PetInfo, String> {
    let id = parse_petdex_id(&input)?;
    let url = format!("https://petdex.crafter.run/pets/{id}");
    let staging = cache_dir()?.join(format!("petdex-{id}-{}", timestamp()));
    fs::create_dir_all(&staging).map_err(to_string)?;

    let manifest = download(PETDEX_MANIFEST_URL)
        .and_then(|bytes| String::from_utf8(bytes).map_err(to_string))
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).map_err(to_string));
    let (pet_url, sprite_url) = match manifest {
        Ok(value) => find_petdex_manifest_urls(&value, &id).unwrap_or_else(|| {
            (
                format!("{PETDEX_RAW_BASE}/{id}/pet.json"),
                format!("{PETDEX_RAW_BASE}/{id}/spritesheet.webp"),
            )
        }),
        Err(_) => (
            format!("{PETDEX_RAW_BASE}/{id}/pet.json"),
            format!("{PETDEX_RAW_BASE}/{id}/spritesheet.webp"),
        ),
    };
    let pet_json = download(&pet_url).map_err(|error| {
        format!(
            "{error}. Install with `npx petdex install {id}` and then use Scan Codex pets."
        )
    })?;
    let sprite = download(&sprite_url).map_err(|error| {
        format!(
            "{error}. Install with `npx petdex install {id}` and then use Scan Codex pets."
        )
    })?;

    fs::write(staging.join("pet.json"), pet_json).map_err(to_string)?;
    fs::write(staging.join("spritesheet.webp"), sprite).map_err(to_string)?;

    import_pet_source(&staging, "petdex", Some(url))
}

#[tauri::command]
fn get_pet_sprite_data_url(pet_id: String) -> Result<String, String> {
    let pet_dir = pets_dir()?.join(sanitize_id(&pet_id));
    let pet_json = read_pet_json(&pet_dir)?;
    let spritesheet = pet_dir.join(&pet_json.spritesheet_path);
    let bytes = fs::read(&spritesheet)
        .map_err(|error| format!("Cannot read spritesheet {}: {error}", spritesheet.display()))?;
    let mime = if pet_json.spritesheet_path.ends_with(".png") {
        "image/png"
    } else {
        "image/webp"
    };

    Ok(format!(
        "data:{mime};base64,{}",
        BASE64_STANDARD.encode(bytes)
    ))
}

#[tauri::command]
fn get_pet_window_position() -> Result<PetPosition, String> {
    Ok(read_app_metadata()
        .map(|metadata| metadata.position)
        .unwrap_or(PetPosition { x: 1200, y: 580 }))
}

#[tauri::command]
fn set_pet_window_position(x: i32, y: i32) -> Result<(), String> {
    let mut metadata = read_app_metadata().unwrap_or_else(default_app_metadata);
    metadata.position = PetPosition { x, y };
    write_app_metadata(&metadata)
}

#[tauri::command]
fn scan_codex_pets() -> Result<Vec<PetInfo>, String> {
    let home = dirs::home_dir().ok_or_else(|| "Cannot locate home directory.".to_string())?;
    let codex_pets = home.join(".codex").join("pets");
    if !codex_pets.exists() {
        return Err("No Codex pets directory found.".to_string());
    }

    let mut imported = Vec::new();
    for entry in fs::read_dir(codex_pets).map_err(to_string)? {
        let path = entry.map_err(to_string)?.path();
        if path.is_dir() || path.extension().is_some_and(|ext| ext == "zip") {
            if let Ok(pet) = import_pet_source(&path, "codex", None) {
                imported.push(pet);
            }
        }
    }

    if imported.is_empty() {
        return Err("No valid Codex pets were found.".to_string());
    }

    Ok(imported)
}

#[tauri::command]
fn get_runtime_state(state: State<AppState>) -> Result<RuntimeState, String> {
    Ok(state.runtime.lock().map_err(to_string)?.clone())
}

#[tauri::command]
fn set_active_pet(pet_id: String, state: State<AppState>) -> Result<RuntimeState, String> {
    let mut runtime = state.runtime.lock().map_err(to_string)?;
    runtime.active_pet_id = Some(pet_id);
    Ok(runtime.clone())
}

#[tauri::command]
fn set_scene(scene: String, state: State<AppState>) -> Result<RuntimeState, String> {
    let mut runtime = state.runtime.lock().map_err(to_string)?;
    runtime.scene = scene;
    Ok(runtime.clone())
}

#[tauri::command]
fn set_scale(scale: f32, state: State<AppState>) -> Result<RuntimeState, String> {
    let mut runtime = state.runtime.lock().map_err(to_string)?;
    runtime.scale = scale.clamp(0.5, 2.0);
    Ok(runtime.clone())
}

fn import_pet_source(path: &Path, source_kind: &str, source_url: Option<String>) -> Result<PetInfo, String> {
    let staging = if path.is_file() {
        extract_zip(path)?
    } else {
        normalize_pet_root(path)?
    };

    let pet_json = read_pet_json(&staging)?;
    let target = pets_dir()?.join(sanitize_id(&pet_json.id));
    if target.exists() {
        fs::remove_dir_all(&target).map_err(to_string)?;
    }

    copy_dir_all(&staging, &target).map_err(to_string)?;
    write_metadata(&target, source_kind, source_url)?;
    validate_pet_dir(&target)
}

fn validate_pet_dir(path: &Path) -> Result<PetInfo, String> {
    let pet_json = read_pet_json(path)?;
    let spritesheet = path.join(&pet_json.spritesheet_path);
    if !spritesheet.exists() {
        return Err(format!("Missing spritesheet: {}", spritesheet.display()));
    }

    let image = image::open(&spritesheet).map_err(|error| {
        format!(
            "Cannot decode spritesheet {}: {error}",
            spritesheet.display()
        )
    })?;
    let (width, height) = image.dimensions();
    if width != ATLAS_WIDTH || height != ATLAS_HEIGHT {
        return Err(format!(
            "Invalid atlas size {width}x{height}. Expected {ATLAS_WIDTH}x{ATLAS_HEIGHT}."
        ));
    }

    read_pet_dir_info(path)
}

fn read_pet_dir_info(path: &Path) -> Result<PetInfo, String> {
    let pet_json = read_pet_json(path)?;
    let spritesheet = path.join(&pet_json.spritesheet_path);
    if !spritesheet.exists() {
        return Err(format!("Missing spritesheet: {}", spritesheet.display()));
    }

    let metadata = read_metadata(path);
    let scale = metadata.as_ref().map(|meta| meta.scale).unwrap_or(1.0);

    Ok(PetInfo {
        id: pet_json.id,
        display_name: pet_json.display_name,
        description: pet_json.description.unwrap_or_default(),
        spritesheet_path: spritesheet.to_string_lossy().to_string(),
        source_kind: metadata
            .as_ref()
            .map(|meta| meta.source.kind.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        source_url: metadata.and_then(|meta| meta.source.url),
        scale,
    })
}

fn read_pet_json(path: &Path) -> Result<CodexPetJson, String> {
    let raw = fs::read_to_string(path.join("pet.json")).map_err(|error| {
        format!("Cannot read {}: {error}", path.join("pet.json").display())
    })?;
    serde_json::from_str(&raw).map_err(|error| format!("Invalid pet.json: {error}"))
}

fn write_metadata(path: &Path, kind: &str, url: Option<String>) -> Result<(), String> {
    let metadata = PetPopMetadata {
        schema_version: 1,
        source: PetSource {
            kind: kind.to_string(),
            url,
        },
        scale: 1.0,
        position: PetPosition { x: 1200, y: 580 },
        imported_at: timestamp(),
    };
    let raw = serde_json::to_string_pretty(&metadata).map_err(to_string)?;
    fs::write(path.join(PETPOP_METADATA_FILE), raw).map_err(to_string)
}

fn read_metadata(path: &Path) -> Option<PetPopMetadata> {
    let raw = fs::read_to_string(path.join(PETPOP_METADATA_FILE))
        .or_else(|_| fs::read_to_string(path.join(TITLECASE_METADATA_FILE)))
        .or_else(|_| fs::read_to_string(path.join(LEGACY_METADATA_FILE)))
        .ok()?;
    serde_json::from_str(&raw).ok()
}

fn default_app_metadata() -> PetPopMetadata {
    PetPopMetadata {
        schema_version: 1,
        source: PetSource {
            kind: "app".to_string(),
            url: None,
        },
        scale: 1.0,
        position: PetPosition { x: 1200, y: 580 },
        imported_at: timestamp(),
    }
}

fn read_app_metadata() -> Option<PetPopMetadata> {
    let app_dir = app_data_dir().ok()?;
    let raw = fs::read_to_string(app_dir.join(PETPOP_METADATA_FILE))
        .or_else(|_| fs::read_to_string(app_dir.join(TITLECASE_METADATA_FILE)))
        .or_else(|_| fs::read_to_string(app_dir.join(LEGACY_METADATA_FILE)))
        .ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_app_metadata(metadata: &PetPopMetadata) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(metadata).map_err(to_string)?;
    fs::write(app_data_dir()?.join(PETPOP_METADATA_FILE), raw).map_err(to_string)
}

fn extract_zip(path: &Path) -> Result<PathBuf, String> {
    let bytes = fs::read(path).map_err(to_string)?;
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(to_string)?;
    let target = cache_dir()?.join(format!("zip-{}", timestamp()));
    fs::create_dir_all(&target).map_err(to_string)?;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(to_string)?;
        let enclosed = file
            .enclosed_name()
            .ok_or_else(|| "Zip contains an unsafe path.".to_string())?
            .to_owned();
        let out_path = target.join(enclosed);

        if file.is_dir() {
            fs::create_dir_all(&out_path).map_err(to_string)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(to_string)?;
            }
            let mut out = fs::File::create(&out_path).map_err(to_string)?;
            io::copy(&mut file, &mut out).map_err(to_string)?;
        }
    }

    normalize_pet_root(&target)
}

fn normalize_pet_root(path: &Path) -> Result<PathBuf, String> {
    if path.join("pet.json").exists() {
        return Ok(path.to_path_buf());
    }

    let mut candidate_dirs = Vec::new();
    for entry in fs::read_dir(path).map_err(to_string)? {
        let child = entry.map_err(to_string)?.path();
        if child.is_dir() && child.join("pet.json").exists() {
            candidate_dirs.push(child);
        }
    }

    match candidate_dirs.len() {
        1 => Ok(candidate_dirs.remove(0)),
        0 => Err("Package does not contain pet.json.".to_string()),
        _ => Err("Package contains multiple pet roots.".to_string()),
    }
}

fn copy_dir_all(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn parse_petdex_id(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_end_matches('/');
    let id = trimmed
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Invalid PetDex id or URL.".to_string())?;

    if id
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_')
    {
        Ok(id.to_ascii_lowercase())
    } else {
        Err("PetDex id can only contain letters, numbers, dash, or underscore.".to_string())
    }
}

fn find_petdex_manifest_urls(value: &serde_json::Value, id: &str) -> Option<(String, String)> {
    let pets = value.get("pets")?.as_array()?;
    let pet = pets
        .iter()
        .find(|pet| pet.get("slug").and_then(|slug| slug.as_str()) == Some(id))?;
    let pet_json_url = pet.get("petJsonUrl")?.as_str()?.to_string();
    let spritesheet_url = pet.get("spritesheetUrl")?.as_str()?.to_string();
    Some((pet_json_url, spritesheet_url))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let response = reqwest::blocking::get(url).map_err(|error| format!("Download failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Download failed with HTTP {}", response.status()));
    }
    response.bytes().map(|bytes| bytes.to_vec()).map_err(to_string)
}

fn app_data_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "Cannot locate application data directory.".to_string())?;
    let dir = base.join(APP_DATA_DIR_NAME);
    migrate_legacy_app_data(&base, &dir)?;
    fs::create_dir_all(&dir).map_err(to_string)?;
    Ok(dir)
}

fn migrate_legacy_app_data(base: &Path, new_dir: &Path) -> Result<(), String> {
    let legacy_dir = base.join(LEGACY_APP_DATA_DIR_NAME);
    if !legacy_dir.exists() {
        return Ok(());
    }

    if !new_dir.exists() {
        copy_dir_all(&legacy_dir, new_dir).map_err(to_string)?;
        return Ok(());
    }

    let legacy_pets = legacy_dir.join("pets");
    let new_pets = new_dir.join("pets");
    if legacy_pets.exists() {
        fs::create_dir_all(&new_pets).map_err(to_string)?;
        for entry in fs::read_dir(&legacy_pets).map_err(to_string)? {
            let source = entry.map_err(to_string)?.path();
            let Some(name) = source.file_name() else {
                continue;
            };
            let target = new_pets.join(name);
            if !target.exists() {
                if source.is_dir() {
                    copy_dir_all(&source, &target).map_err(to_string)?;
                } else {
                    fs::copy(&source, &target).map_err(to_string)?;
                }
            }
        }
    }

    let new_metadata = new_dir.join(PETPOP_METADATA_FILE);
    if !new_metadata.exists() {
        for candidate in [PETPOP_METADATA_FILE, TITLECASE_METADATA_FILE, LEGACY_METADATA_FILE] {
            let legacy_metadata = legacy_dir.join(candidate);
            if legacy_metadata.exists() {
                fs::copy(legacy_metadata, &new_metadata).map_err(to_string)?;
                break;
            }
        }
    }

    Ok(())
}

fn pets_dir() -> Result<PathBuf, String> {
    let dir = app_data_dir()?.join("pets");
    fs::create_dir_all(&dir).map_err(to_string)?;
    Ok(dir)
}

fn cache_dir() -> Result<PathBuf, String> {
    let dir = app_data_dir()?.join("cache");
    fs::create_dir_all(&dir).map_err(to_string)?;
    Ok(dir)
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char == '-' || char == '_' {
                char
            } else {
                '-'
            }
        })
        .collect()
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}
