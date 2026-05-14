fn main() {
    let mut attributes = tauri_build::Attributes::new();

    #[cfg(target_os = "windows")]
    {
        attributes = attributes.windows_attributes(
            tauri_build::WindowsAttributes::new().window_icon_path("icons/icon.ico"),
        );
    }

    tauri_build::try_build(attributes).expect("failed to build Tauri app");
}
