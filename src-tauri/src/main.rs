#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{self, Cursor},
    path::{Component, Path, PathBuf},
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
const SETTINGS_FILE: &str = "settings.json";
const CODEX_ACTIVITY_FILE: &str = "codex-activity.json";
const PETPOP_METADATA_SCHEMA_VERSION: u8 = 2;
const PETDEX_PAGE_BASE: &str = "https://petdex.crafter.run/pets";
const CODEX_ACTIVE_EXPIRE_MS: u64 = 5 * 60 * 1000;
const CODEX_FEEDBACK_EXPIRE_MS: u64 = 15 * 1000;

const PET_ACTION_EVENTS: &[&str] = &[
    "drag-left",
    "drag-right",
    "drag-start",
    "drag-end",
    "click",
    "double-click",
    "idle",
    "waiting",
    "task-running",
    "success",
    "error",
    "review",
    "codex-running",
    "codex-waiting",
    "codex-review",
    "codex-success",
    "codex-error",
    "focus-start",
    "focus-pause",
    "focus-resume",
    "focus-complete",
    "focus-cancel",
    "break-start",
    "break-complete",
];

const VISIBLE_PET_ACTION_EVENTS: &[&str] = &[
    "drag-left",
    "drag-right",
    "click",
    "double-click",
    "idle",
    "waiting",
    "codex-running",
    "codex-waiting",
    "codex-review",
    "codex-success",
    "codex-error",
    "focus-start",
    "focus-pause",
    "focus-resume",
    "focus-complete",
    "focus-cancel",
    "break-start",
    "break-complete",
];

const PET_ANIMATION_STATES: &[&str] = &[
    "idle",
    "running-right",
    "running-left",
    "waving",
    "jumping",
    "failed",
    "waiting",
    "running",
    "review",
];

struct AppState {
    runtime: Mutex<RuntimeState>,
    last_codex_activity: Mutex<CodexActivity>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            runtime: Mutex::new(RuntimeState::default()),
            last_codex_activity: Mutex::new(CodexActivity::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeState {
    active_pet_id: Option<String>,
    scene: String,
    scale: f32,
    focus_state: FocusState,
    codex_activity: CodexActivity,
    codex_activity_error: Option<String>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            active_pet_id: None,
            scene: "idle".to_string(),
            scale: 0.5,
            focus_state: FocusState::default(),
            codex_activity: CodexActivity::default(),
            codex_activity_error: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FocusState {
    mode: String,
    status: String,
    last_event: Option<String>,
    remaining_ms: Option<u64>,
    ends_at: Option<u64>,
    updated_at: u64,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            mode: "idle".to_string(),
            status: "idle".to_string(),
            last_event: None,
            remaining_ms: None,
            ends_at: None,
            updated_at: timestamp_ms(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexActivity {
    status: String,
    message: Option<String>,
    updated_at: u64,
}

impl Default for CodexActivity {
    fn default() -> Self {
        Self {
            status: "idle".to_string(),
            message: None,
            updated_at: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    focus_minutes: u32,
    break_minutes: u32,
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
    action_map: HashMap<String, String>,
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
struct RawCodexPetJson {
    id: Option<String>,
    display_name: String,
    description: Option<String>,
    spritesheet_path: String,
}

impl TryFrom<RawCodexPetJson> for CodexPetJson {
    type Error = String;

    fn try_from(value: RawCodexPetJson) -> Result<Self, Self::Error> {
        let id = value
            .id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| "pet.json 缺少 id。".to_string())?;

        Ok(Self {
            id,
            display_name: value.display_name,
            description: value.description,
            spritesheet_path: value.spritesheet_path,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetPopMetadata {
    schema_version: u8,
    source: PetSource,
    scale: f32,
    #[serde(default = "default_action_map")]
    action_map: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    position: Option<PetPosition>,
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
            let show = MenuItem::with_id(app, "show", "显示 PetPop", true, None::<&str>)?;
            let hide_pet = MenuItem::with_id(app, "hide_pet", "隐藏桌宠", true, None::<&str>)?;
            let show_pet = MenuItem::with_id(app, "show_pet", "显示桌宠", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
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
            remove_pet,
            get_pet_sprite_data_url,
            get_pet_window_position,
            set_pet_window_position,
            get_runtime_state,
            set_active_pet,
            set_scene,
            set_scale,
            set_pet_action_map,
            get_app_settings,
            set_app_settings,
            set_focus_state,
            is_left_mouse_button_pressed
        ])
        .run(tauri::generate_context!())
        .expect("PetPop 运行失败");
}

#[tauri::command]
fn list_pets() -> Result<Vec<PetInfo>, String> {
    let pets_dir = pets_dir()?;
    if !pets_dir.exists() {
        fs::create_dir_all(&pets_dir).map_err(to_string)?;
    }

    list_pets_in_dir(&pets_dir)
}

fn list_pets_in_dir(pets_dir: &Path) -> Result<Vec<PetInfo>, String> {
    let mut pets = Vec::new();
    for entry in fs::read_dir(pets_dir).map_err(to_string)? {
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
        return Err("宠物包路径不存在。".to_string());
    }

    import_pet_source(&source, "local", None)
}

#[tauri::command]
fn import_petdex(input: String) -> Result<PetInfo, String> {
    let id = parse_petdex_id(&input)?;
    let url = format!("{PETDEX_PAGE_BASE}/{id}");
    let staging = cache_dir()?.join(format!("petdex-{id}-{}", timestamp()));
    fs::create_dir_all(&staging).map_err(to_string)?;

    let manifest = download(PETDEX_MANIFEST_URL)
        .and_then(|bytes| String::from_utf8(bytes).map_err(to_string))
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).map_err(to_string));
    let manifest_entry = match manifest {
        Ok(value) => find_petdex_manifest_entry(&value, &id).unwrap_or_else(|| {
            (
                id.clone(),
                format!("{PETDEX_RAW_BASE}/{id}/pet.json"),
                format!("{PETDEX_RAW_BASE}/{id}/spritesheet.webp"),
            )
        }),
        Err(_) => (
            id.clone(),
            format!("{PETDEX_RAW_BASE}/{id}/pet.json"),
            format!("{PETDEX_RAW_BASE}/{id}/spritesheet.webp"),
        ),
    };
    let (manifest_slug, pet_url, sprite_url) = manifest_entry;
    let pet_json = download(&pet_url).map_err(|error| {
        format!("{error}。也可以先运行 `npx petdex install {id}`，再使用“扫描 Codex 宠物”。")
    })?;
    let raw_pet = read_raw_pet_json_bytes(&pet_json)?;
    let normalized_pet = normalize_petdex_pet_json(raw_pet, &manifest_slug);
    let spritesheet_path = safe_relative_path(&normalized_pet.spritesheet_path)?;
    let sprite = download(&sprite_url).map_err(|error| {
        format!("{error}。也可以先运行 `npx petdex install {id}`，再使用“扫描 Codex 宠物”。")
    })?;

    fs::write(
        staging.join("pet.json"),
        serde_json::to_vec_pretty(&normalized_pet).map_err(to_string)?,
    )
    .map_err(to_string)?;
    write_staged_spritesheet(&staging, &spritesheet_path, &sprite)?;

    import_pet_source(&staging, "petdex", Some(url))
}

#[tauri::command]
fn get_pet_sprite_data_url(pet_id: String) -> Result<String, String> {
    let pet_dir = pets_dir()?.join(sanitize_id(&pet_id));
    let pet_json = read_pet_json(&pet_dir)?;
    let spritesheet_path = safe_relative_path(&pet_json.spritesheet_path)?;
    let spritesheet = pet_dir.join(spritesheet_path);
    let bytes = fs::read(&spritesheet)
        .map_err(|error| format!("无法读取精灵图 {}：{error}", spritesheet.display()))?;
    let spritesheet_name = pet_json.spritesheet_path.to_ascii_lowercase();
    let mime = if spritesheet_name.ends_with(".png") {
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
        .and_then(|metadata| metadata.position)
        .unwrap_or(PetPosition { x: 1200, y: 580 }))
}

#[tauri::command]
fn set_pet_window_position(x: i32, y: i32) -> Result<(), String> {
    let mut metadata = read_app_metadata().unwrap_or_else(default_app_metadata);
    metadata.position = Some(PetPosition { x, y });
    write_app_metadata(&metadata)
}

#[tauri::command]
fn scan_codex_pets() -> Result<Vec<PetInfo>, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法定位用户主目录。".to_string())?;
    let codex_pets = home.join(".codex").join("pets");
    if !codex_pets.exists() {
        return Err("未找到 Codex 宠物目录。".to_string());
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
        return Err("没有找到有效的 Codex 宠物。".to_string());
    }

    Ok(imported)
}

#[tauri::command]
fn remove_pet(pet_id: String, state: State<AppState>) -> Result<RuntimeState, String> {
    let pets_dir = pets_dir()?;
    remove_pet_from_dir(&pets_dir, &pet_id)?;
    let remaining = list_pets_in_dir(&pets_dir)?;

    {
        let mut runtime = state.runtime.lock().map_err(to_string)?;
        sync_runtime_after_pet_removed(&mut runtime, &pet_id, &remaining);
    }

    runtime_snapshot(state.inner())
}

#[tauri::command]
fn get_runtime_state(state: State<AppState>) -> Result<RuntimeState, String> {
    runtime_snapshot(state.inner())
}

#[tauri::command]
fn set_active_pet(pet_id: String, state: State<AppState>) -> Result<RuntimeState, String> {
    {
        let mut runtime = state.runtime.lock().map_err(to_string)?;
        runtime.active_pet_id = Some(pet_id);
    }
    runtime_snapshot(state.inner())
}

#[tauri::command]
fn set_scene(scene: String, state: State<AppState>) -> Result<RuntimeState, String> {
    if !PET_ANIMATION_STATES.contains(&scene.as_str()) {
        return Err(format!("未知动画状态：{scene}"));
    }
    {
        let mut runtime = state.runtime.lock().map_err(to_string)?;
        runtime.scene = scene;
    }
    runtime_snapshot(state.inner())
}

#[tauri::command]
fn set_scale(scale: f32, state: State<AppState>) -> Result<RuntimeState, String> {
    {
        let mut runtime = state.runtime.lock().map_err(to_string)?;
        runtime.scale = scale.clamp(0.1, 1.0);
    }
    runtime_snapshot(state.inner())
}

#[tauri::command]
fn get_app_settings() -> Result<AppSettings, String> {
    Ok(read_app_settings().unwrap_or_else(default_app_settings))
}

#[tauri::command]
fn set_app_settings(focus_minutes: u32, break_minutes: u32) -> Result<AppSettings, String> {
    let settings = AppSettings {
        focus_minutes: focus_minutes.clamp(1, 180),
        break_minutes: break_minutes.clamp(1, 60),
    };
    write_app_settings(&settings)?;
    Ok(settings)
}

#[tauri::command]
fn set_focus_state(
    mode: String,
    status: String,
    last_event: Option<String>,
    remaining_ms: Option<u64>,
    ends_at: Option<u64>,
    state: State<AppState>,
) -> Result<RuntimeState, String> {
    if !["idle", "focus", "break"].contains(&mode.as_str()) {
        return Err(format!("未知专注模式：{mode}"));
    }
    if !["idle", "running", "paused", "complete"].contains(&status.as_str()) {
        return Err(format!("未知专注状态：{status}"));
    }
    if let Some(event) = &last_event {
        if !PET_ACTION_EVENTS.contains(&event.as_str()) {
            return Err(format!("未知动作事件：{event}"));
        }
    }

    {
        let mut runtime = state.runtime.lock().map_err(to_string)?;
        runtime.focus_state = FocusState {
            mode,
            status,
            last_event,
            remaining_ms,
            ends_at,
            updated_at: timestamp_ms(),
        };
    }
    runtime_snapshot(state.inner())
}

#[tauri::command]
fn is_left_mouse_button_pressed() -> bool {
    left_mouse_button_pressed()
}

#[cfg(target_os = "windows")]
fn left_mouse_button_pressed() -> bool {
    const VK_LBUTTON: i32 = 0x01;
    const KEY_PRESSED_MASK: i16 = i16::MIN;

    unsafe { (GetAsyncKeyState(VK_LBUTTON) & KEY_PRESSED_MASK) != 0 }
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn GetAsyncKeyState(v_key: i32) -> i16;
}

#[cfg(not(target_os = "windows"))]
fn left_mouse_button_pressed() -> bool {
    true
}

#[tauri::command]
fn set_pet_action_map(
    pet_id: String,
    action_map: HashMap<String, String>,
) -> Result<PetInfo, String> {
    let normalized = normalize_stored_action_map(Some(action_map))?;
    let pet_dir = pets_dir()?.join(sanitize_id(&pet_id));
    if !pet_dir.exists() {
        return Err("该宠物尚未导入。".to_string());
    }

    let mut metadata =
        read_metadata(&pet_dir).unwrap_or_else(|| default_pet_metadata("unknown", None));
    metadata.schema_version = PETPOP_METADATA_SCHEMA_VERSION;
    metadata.action_map = normalized;
    write_metadata_file(&pet_dir, &metadata)?;
    read_pet_dir_info(&pet_dir)
}

fn import_pet_source(
    path: &Path,
    source_kind: &str,
    source_url: Option<String>,
) -> Result<PetInfo, String> {
    let staging = if path.is_file() {
        extract_zip(path)?
    } else {
        normalize_pet_root(path)?
    };

    let pet_json = read_pet_json(&staging)?;
    validate_pet_dir(&staging)?;

    let pets_dir = pets_dir()?;
    let target = pets_dir.join(sanitize_id(&pet_json.id));
    let temp_target = pets_dir.join(format!(
        ".{}-import-{}",
        sanitize_id(&pet_json.id),
        timestamp()
    ));
    if temp_target.exists() {
        fs::remove_dir_all(&temp_target).map_err(to_string)?;
    }

    copy_dir_all(&staging, &temp_target).map_err(to_string)?;
    write_metadata(&temp_target, source_kind, source_url)?;
    validate_pet_dir(&temp_target)?;

    if target.exists() {
        fs::remove_dir_all(&target).map_err(to_string)?;
    }
    fs::rename(&temp_target, &target).map_err(to_string)?;
    read_pet_dir_info(&target)
}

fn validate_pet_dir(path: &Path) -> Result<PetInfo, String> {
    let pet_json = read_pet_json(path)?;
    let spritesheet_path = safe_relative_path(&pet_json.spritesheet_path)?;
    let spritesheet = path.join(spritesheet_path);
    if !spritesheet.exists() {
        return Err(format!("缺少精灵图：{}", spritesheet.display()));
    }

    let image = image::open(&spritesheet)
        .map_err(|error| format!("无法解码精灵图 {}：{error}", spritesheet.display()))?;
    let (width, height) = image.dimensions();
    if width != ATLAS_WIDTH || height != ATLAS_HEIGHT {
        return Err(format!(
            "图集尺寸无效：{width}x{height}。应为 {ATLAS_WIDTH}x{ATLAS_HEIGHT}。"
        ));
    }

    read_pet_dir_info(path)
}

fn read_pet_dir_info(path: &Path) -> Result<PetInfo, String> {
    let pet_json = read_pet_json(path)?;
    let spritesheet_path = safe_relative_path(&pet_json.spritesheet_path)?;
    let spritesheet = path.join(spritesheet_path);
    if !spritesheet.exists() {
        return Err(format!("缺少精灵图：{}", spritesheet.display()));
    }

    let metadata = read_metadata(path);
    let scale = metadata
        .as_ref()
        .map(|meta| meta.scale.clamp(0.1, 1.0))
        .unwrap_or(0.5);
    let action_map = metadata
        .as_ref()
        .map(|meta| meta.action_map.clone())
        .unwrap_or_else(default_action_map);

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
        action_map,
    })
}

fn read_pet_json(path: &Path) -> Result<CodexPetJson, String> {
    let raw = fs::read_to_string(path.join("pet.json"))
        .map_err(|error| format!("无法读取 {}：{error}", path.join("pet.json").display()))?;
    read_pet_json_str(&raw)
}

fn read_pet_json_str(raw: &str) -> Result<CodexPetJson, String> {
    let raw_pet: RawCodexPetJson =
        serde_json::from_str(raw).map_err(|error| format!("pet.json 无效：{error}"))?;
    raw_pet.try_into()
}

fn read_raw_pet_json_bytes(bytes: &[u8]) -> Result<RawCodexPetJson, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("PetDex pet.json 无效：{error}"))
}

fn normalize_petdex_pet_json(raw: RawCodexPetJson, manifest_slug: &str) -> CodexPetJson {
    CodexPetJson {
        id: raw
            .id
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| manifest_slug.to_string()),
        display_name: raw.display_name,
        description: raw.description,
        spritesheet_path: raw.spritesheet_path,
    }
}

fn write_metadata(path: &Path, kind: &str, url: Option<String>) -> Result<(), String> {
    let metadata = default_pet_metadata(kind, url);
    write_metadata_file(path, &metadata)
}

fn write_metadata_file(path: &Path, metadata: &PetPopMetadata) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(metadata).map_err(to_string)?;
    fs::write(path.join(PETPOP_METADATA_FILE), raw).map_err(to_string)
}

fn default_pet_metadata(kind: &str, url: Option<String>) -> PetPopMetadata {
    PetPopMetadata {
        schema_version: PETPOP_METADATA_SCHEMA_VERSION,
        source: PetSource {
            kind: kind.to_string(),
            url,
        },
        scale: 0.5,
        action_map: default_visible_action_map(),
        position: None,
        imported_at: timestamp(),
    }
}

fn read_metadata(path: &Path) -> Option<PetPopMetadata> {
    let raw = fs::read_to_string(path.join(PETPOP_METADATA_FILE))
        .or_else(|_| fs::read_to_string(path.join(TITLECASE_METADATA_FILE)))
        .or_else(|_| fs::read_to_string(path.join(LEGACY_METADATA_FILE)))
        .ok()?;
    let mut metadata: PetPopMetadata = serde_json::from_str(&raw).ok()?;
    metadata.action_map =
        normalize_action_map(Some(metadata.action_map)).unwrap_or_else(|_| default_action_map());
    Some(metadata)
}

fn default_app_metadata() -> PetPopMetadata {
    PetPopMetadata {
        schema_version: PETPOP_METADATA_SCHEMA_VERSION,
        source: PetSource {
            kind: "app".to_string(),
            url: None,
        },
        scale: 0.5,
        action_map: default_visible_action_map(),
        position: Some(PetPosition { x: 1200, y: 580 }),
        imported_at: timestamp(),
    }
}

fn default_app_settings() -> AppSettings {
    AppSettings {
        focus_minutes: 25,
        break_minutes: 5,
    }
}

fn read_app_settings() -> Option<AppSettings> {
    let app_dir = app_data_dir().ok()?;
    let raw = fs::read_to_string(app_dir.join(SETTINGS_FILE)).ok()?;
    let settings: AppSettings = serde_json::from_str(&raw).ok()?;
    Some(AppSettings {
        focus_minutes: settings.focus_minutes.clamp(1, 180),
        break_minutes: settings.break_minutes.clamp(1, 60),
    })
}

fn write_app_settings(settings: &AppSettings) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(settings).map_err(to_string)?;
    fs::write(app_data_dir()?.join(SETTINGS_FILE), raw).map_err(to_string)
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

fn default_action_map() -> HashMap<String, String> {
    HashMap::from([
        ("drag-left".to_string(), "running-left".to_string()),
        ("drag-right".to_string(), "running-right".to_string()),
        ("drag-start".to_string(), "waving".to_string()),
        ("drag-end".to_string(), "idle".to_string()),
        ("click".to_string(), "waving".to_string()),
        ("double-click".to_string(), "jumping".to_string()),
        ("idle".to_string(), "idle".to_string()),
        ("waiting".to_string(), "waiting".to_string()),
        ("task-running".to_string(), "running".to_string()),
        ("success".to_string(), "jumping".to_string()),
        ("error".to_string(), "failed".to_string()),
        ("review".to_string(), "review".to_string()),
        ("codex-running".to_string(), "running".to_string()),
        ("codex-waiting".to_string(), "waiting".to_string()),
        ("codex-review".to_string(), "review".to_string()),
        ("codex-success".to_string(), "jumping".to_string()),
        ("codex-error".to_string(), "failed".to_string()),
        ("focus-start".to_string(), "running".to_string()),
        ("focus-pause".to_string(), "waiting".to_string()),
        ("focus-resume".to_string(), "running".to_string()),
        ("focus-complete".to_string(), "jumping".to_string()),
        ("focus-cancel".to_string(), "idle".to_string()),
        ("break-start".to_string(), "waving".to_string()),
        ("break-complete".to_string(), "jumping".to_string()),
    ])
}

fn default_visible_action_map() -> HashMap<String, String> {
    default_action_map()
        .into_iter()
        .filter(|(event, _)| VISIBLE_PET_ACTION_EVENTS.contains(&event.as_str()))
        .collect()
}

fn normalize_action_map(
    action_map: Option<HashMap<String, String>>,
) -> Result<HashMap<String, String>, String> {
    let mut normalized = default_action_map();

    if let Some(action_map) = action_map {
        for (event, state) in action_map {
            if !PET_ACTION_EVENTS.contains(&event.as_str()) {
                return Err(format!("未知动作事件：{event}"));
            }

            if !PET_ANIMATION_STATES.contains(&state.as_str()) {
                return Err(format!("未知动画状态：{state}"));
            }

            normalized.insert(event, state);
        }
    }

    Ok(normalized)
}

fn normalize_stored_action_map(
    action_map: Option<HashMap<String, String>>,
) -> Result<HashMap<String, String>, String> {
    let normalized = normalize_action_map(action_map)?;
    Ok(normalized
        .into_iter()
        .filter(|(event, _)| VISIBLE_PET_ACTION_EVENTS.contains(&event.as_str()))
        .collect())
}

fn remove_pet_from_dir(pets_dir: &Path, pet_id: &str) -> Result<(), String> {
    let sanitized = sanitize_id(pet_id);
    if sanitized.is_empty() {
        return Err("宠物 ID 不能为空。".to_string());
    }

    let target = pets_dir.join(sanitized);
    if !target.exists() {
        return Err("该宠物尚未导入。".to_string());
    }
    if !target.is_dir() {
        return Err("宠物副本不是目录，无法移除。".to_string());
    }

    read_pet_json(&target)?;
    fs::remove_dir_all(&target).map_err(to_string)
}

fn sync_runtime_after_pet_removed(
    runtime: &mut RuntimeState,
    removed_pet_id: &str,
    remaining_pets: &[PetInfo],
) {
    if runtime.active_pet_id.as_deref() != Some(removed_pet_id) {
        return;
    }

    runtime.active_pet_id = remaining_pets.first().map(|pet| pet.id.clone());
    runtime.scene = "idle".to_string();
}

fn runtime_snapshot(state: &AppState) -> Result<RuntimeState, String> {
    let (codex_activity, codex_activity_error) = refresh_codex_activity(state)?;
    let mut runtime = state.runtime.lock().map_err(to_string)?.clone();
    runtime.codex_activity = codex_activity;
    runtime.codex_activity_error = codex_activity_error;
    Ok(runtime)
}

fn refresh_codex_activity(state: &AppState) -> Result<(CodexActivity, Option<String>), String> {
    match read_codex_activity_bridge() {
        Ok(Some(activity)) => {
            let mut last = state.last_codex_activity.lock().map_err(to_string)?;
            *last = activity.clone();
            Ok((activity, None))
        }
        Ok(None) => {
            let last = state.last_codex_activity.lock().map_err(to_string)?.clone();
            Ok((last, None))
        }
        Err(error) => {
            let last = state.last_codex_activity.lock().map_err(to_string)?.clone();
            Ok((last, Some(error)))
        }
    }
}

fn read_codex_activity_bridge() -> Result<Option<CodexActivity>, String> {
    let path = app_data_dir()?.join(CODEX_ACTIVITY_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("无法读取 Codex 状态桥文件 {}：{error}", path.display()))?;
    let mut activity: CodexActivity = serde_json::from_str(&raw)
        .map_err(|error| format!("Codex 状态桥文件 JSON 无效：{error}"))?;
    validate_codex_activity(&mut activity)?;

    if codex_activity_expired(&activity) {
        return Ok(Some(CodexActivity::default()));
    }

    Ok(Some(activity))
}

fn validate_codex_activity(activity: &mut CodexActivity) -> Result<(), String> {
    if !["idle", "running", "waiting", "review", "success", "error"]
        .contains(&activity.status.as_str())
    {
        return Err(format!("未知 Codex 状态：{}", activity.status));
    }

    if activity.updated_at > 0 && activity.updated_at < 1_000_000_000_000 {
        activity.updated_at *= 1000;
    }

    Ok(())
}

fn codex_activity_expired(activity: &CodexActivity) -> bool {
    if activity.status == "idle" || activity.updated_at == 0 {
        return false;
    }

    let age_ms = timestamp_ms().saturating_sub(activity.updated_at);
    let max_age = match activity.status.as_str() {
        "success" | "error" => CODEX_FEEDBACK_EXPIRE_MS,
        _ => CODEX_ACTIVE_EXPIRE_MS,
    };
    age_ms > max_age
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
            .ok_or_else(|| "Zip 中包含不安全路径。".to_string())?
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
        0 => Err("宠物包中没有 pet.json。".to_string()),
        _ => Err("宠物包中包含多个宠物根目录。".to_string()),
    }
}

fn safe_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    if path.as_os_str().is_empty() {
        return Err("spritesheetPath 不能为空。".to_string());
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        )
    }) {
        return Err("spritesheetPath 必须是安全的相对路径。".to_string());
    }

    Ok(path)
}

fn write_staged_spritesheet(
    staging: &Path,
    spritesheet_path: &Path,
    sprite: &[u8],
) -> Result<(), String> {
    let out_path = staging.join(spritesheet_path);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(to_string)?;
    }

    fs::write(out_path, sprite).map_err(to_string)
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
        .ok_or_else(|| "PetDex ID 或链接无效。".to_string())?;

    if id
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_')
    {
        Ok(id.to_ascii_lowercase())
    } else {
        Err("PetDex ID 只能包含字母、数字、短横线或下划线。".to_string())
    }
}

fn find_petdex_manifest_entry(
    value: &serde_json::Value,
    id: &str,
) -> Option<(String, String, String)> {
    let pets = value.get("pets")?.as_array()?;
    let pet = pets
        .iter()
        .find(|pet| pet.get("slug").and_then(|slug| slug.as_str()) == Some(id))?;
    let slug = pet.get("slug")?.as_str()?.to_string();
    let pet_json_url = pet.get("petJsonUrl")?.as_str()?.to_string();
    let spritesheet_url = pet.get("spritesheetUrl")?.as_str()?.to_string();
    Some((slug, pet_json_url, spritesheet_url))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let response = reqwest::blocking::get(url).map_err(|error| format!("下载失败：{error}"))?;
    if !response.status().is_success() {
        return Err(format!("下载失败，HTTP 状态码 {}", response.status()));
    }
    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(to_string)
}

fn app_data_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "无法定位应用数据目录。".to_string())?;
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
        for candidate in [
            PETPOP_METADATA_FILE,
            TITLECASE_METADATA_FILE,
            LEGACY_METADATA_FILE,
        ] {
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

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_petdex_id_from_id_or_url() {
        assert_eq!(parse_petdex_id("Cortana").unwrap(), "cortana");
        assert_eq!(
            parse_petdex_id("https://petdex.crafter.run/pets/kurisu-2/").unwrap(),
            "kurisu-2"
        );
        assert!(parse_petdex_id("bad id").is_err());
    }

    #[test]
    fn petdex_pet_json_uses_slug_when_id_is_missing() {
        let raw = read_raw_pet_json_bytes(
            br#"{
                "displayName": "Kurisu",
                "description": "Custom transparent 8x9 spritesheet.",
                "spritesheetPath": "spritesheet.png"
            }"#,
        )
        .unwrap();
        let normalized = normalize_petdex_pet_json(raw, "kurisu-2");

        assert_eq!(normalized.id, "kurisu-2");
        assert_eq!(normalized.display_name, "Kurisu");
        assert_eq!(normalized.spritesheet_path, "spritesheet.png");
    }

    #[test]
    fn local_pet_json_still_requires_id() {
        let result = read_pet_json_str(
            r#"{
                "displayName": "No Id",
                "spritesheetPath": "spritesheet.webp"
            }"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn writes_spritesheet_to_declared_relative_path() {
        let root = std::env::temp_dir().join(format!("petpop-test-{}", timestamp()));
        let spritesheet_path = safe_relative_path("assets/spritesheet.png").unwrap();

        write_staged_spritesheet(&root, &spritesheet_path, b"sprite").unwrap();

        assert_eq!(
            fs::read(root.join("assets").join("spritesheet.png")).unwrap(),
            b"sprite"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_unsafe_spritesheet_paths() {
        assert!(safe_relative_path("../spritesheet.png").is_err());
        assert!(safe_relative_path("C:\\pets\\spritesheet.png").is_err());
    }

    #[test]
    fn validates_action_map_values() {
        let map = HashMap::from([("click".to_string(), "jumping".to_string())]);
        assert_eq!(normalize_action_map(Some(map)).unwrap()["click"], "jumping");

        let invalid_event = HashMap::from([("tap".to_string(), "jumping".to_string())]);
        assert!(normalize_action_map(Some(invalid_event)).is_err());

        let invalid_state = HashMap::from([("click".to_string(), "dance".to_string())]);
        assert!(normalize_action_map(Some(invalid_state)).is_err());
    }

    #[test]
    fn stores_only_visible_action_map_events() {
        let map = HashMap::from([
            ("click".to_string(), "jumping".to_string()),
            ("success".to_string(), "jumping".to_string()),
            ("drag-start".to_string(), "waving".to_string()),
        ]);
        let normalized = normalize_stored_action_map(Some(map)).unwrap();

        assert_eq!(normalized["click"], "jumping");
        assert!(!normalized.contains_key("success"));
        assert!(!normalized.contains_key("drag-start"));
    }

    #[test]
    fn remove_pet_from_dir_deletes_only_the_imported_pet_copy() {
        let root = std::env::temp_dir().join(format!("petpop-remove-test-{}", timestamp()));
        let pets = root.join("pets");
        let imported_pet = pets.join("alpha");
        let codex_source = root.join("codex-source").join("alpha");
        write_minimal_pet_dir(&imported_pet, "alpha", "Alpha");
        write_minimal_pet_dir(&codex_source, "alpha", "Alpha");

        remove_pet_from_dir(&pets, "alpha").unwrap();

        assert!(!imported_pet.exists());
        assert!(codex_source.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn removing_active_pet_selects_the_next_available_pet() {
        let mut runtime = RuntimeState {
            active_pet_id: Some("alpha".to_string()),
            scene: "running".to_string(),
            ..RuntimeState::default()
        };
        let remaining = vec![test_pet_info("beta", "Beta")];

        sync_runtime_after_pet_removed(&mut runtime, "alpha", &remaining);

        assert_eq!(runtime.active_pet_id.as_deref(), Some("beta"));
        assert_eq!(runtime.scene, "idle");
    }

    #[test]
    fn removing_last_active_pet_clears_runtime_selection() {
        let mut runtime = RuntimeState {
            active_pet_id: Some("alpha".to_string()),
            scene: "running".to_string(),
            ..RuntimeState::default()
        };

        sync_runtime_after_pet_removed(&mut runtime, "alpha", &[]);

        assert_eq!(runtime.active_pet_id, None);
        assert_eq!(runtime.scene, "idle");
    }

    fn write_minimal_pet_dir(path: &Path, id: &str, display_name: &str) {
        fs::create_dir_all(path).unwrap();
        fs::write(
            path.join("pet.json"),
            format!(
                r#"{{
  "id": "{id}",
  "displayName": "{display_name}",
  "description": "Test pet",
  "spritesheetPath": "spritesheet.webp"
}}"#
            ),
        )
        .unwrap();
        fs::write(path.join("spritesheet.webp"), b"sprite").unwrap();
    }

    fn test_pet_info(id: &str, display_name: &str) -> PetInfo {
        PetInfo {
            id: id.to_string(),
            display_name: display_name.to_string(),
            description: String::new(),
            spritesheet_path: "spritesheet.webp".to_string(),
            source_kind: "local".to_string(),
            source_url: None,
            scale: 0.5,
            action_map: default_visible_action_map(),
        }
    }
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}
