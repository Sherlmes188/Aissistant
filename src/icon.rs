use eframe::egui;
use image::imageops::FilterType;
use image::ImageFormat;
use std::io::Cursor;
use std::path::PathBuf;

const APP_ICON: &str = "bc84c63b-6e18-4fd5-82f9-54d1ee493479.png";

pub fn app_icon_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(APP_ICON)
}

pub fn load_window_icon() -> Option<egui::IconData> {
    let image = image::open(app_icon_path()).ok()?;
    let rgba = image.resize(96, 96, FilterType::Lanczos3).to_rgba8();
    let (width, height) = rgba.dimensions();

    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

pub fn ensure_tray_icon() -> Option<PathBuf> {
    let image = image::open(app_icon_path()).ok()?;
    let resized = image.resize_exact(256, 256, FilterType::Lanczos3);

    let mut png = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .ok()?;

    let icon_path = std::env::temp_dir()
        .join("Aissistant")
        .join("aissistant.ico");
    if let Some(parent) = icon_path.parent() {
        std::fs::create_dir_all(parent).ok()?;
    }

    let mut ico = Vec::with_capacity(22 + png.len());
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.push(0);
    ico.push(0);
    ico.push(0);
    ico.push(0);
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(&png);

    std::fs::write(&icon_path, ico).ok()?;
    Some(icon_path)
}
