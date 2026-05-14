import { describe, expect, it } from "vitest";
import buildSource from "../src-tauri/build.rs?raw";
import mainSource from "../src-tauri/src/main.rs?raw";

describe("tauri main window lifecycle", () => {
  it("keeps the main webview available after the user closes the control panel", () => {
    expect(mainSource).toContain("CloseRequested");
    expect(mainSource).toContain("api.prevent_close()");
    expect(mainSource).toContain("window.hide()");
  });

  it("does not create a Windows console for debug builds", () => {
    expect(mainSource).toContain('cfg_attr(target_os = "windows", windows_subsystem = "windows")');
    expect(mainSource).not.toContain("not(debug_assertions)");
  });

  it("uses the packaged app icon for the tray and Windows executable", () => {
    expect(mainSource).toContain("app.default_window_icon().cloned()");
    expect(mainSource).toContain("tray = tray.icon(icon)");
    expect(buildSource).toContain('window_icon_path("icons/icon.ico")');
  });
});

describe("tauri runtime scale persistence", () => {
  it("loads the runtime scale from persisted app settings on startup", () => {
    expect(mainSource).toContain("runtime.scale = settings.pet_scale");
  });

  it("persists scale changes without resetting focus settings", () => {
    const setScaleFunction = mainSource.match(
      /fn set_scale\(scale: f32, state: State<AppState>\) -> Result<RuntimeState, String> \{[\s\S]*?\n\}/,
    )?.[0];

    expect(setScaleFunction).toBeTruthy();
    expect(setScaleFunction).toContain("settings.pet_scale = next_scale");
    expect(setScaleFunction).toContain("write_app_settings(&settings)?");
  });

  it("preserves the persisted scale when saving focus settings", () => {
    const setSettingsFunction = mainSource.match(
      /fn set_app_settings\(focus_minutes: u32, break_minutes: u32\) -> Result<AppSettings, String> \{[\s\S]*?\n\}/,
    )?.[0];

    expect(setSettingsFunction).toBeTruthy();
    expect(setSettingsFunction).toContain("pet_scale: current_settings.pet_scale");
  });
});
