use image::imageops::FilterType;
use image::ImageFormat;
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP_ICON: &str = "bc84c63b-6e18-4fd5-82f9-54d1ee493479.png";

fn main() {
    println!("cargo:rerun-if-changed={APP_ICON}");

    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }

    let Ok(out_dir) = env::var("OUT_DIR") else {
        return;
    };

    let out_dir = PathBuf::from(out_dir);
    let icon_path = out_dir.join("aissistant.ico");
    let rc_path = out_dir.join("aissistant.rc");
    let res_path = out_dir.join("aissistant.res");

    if create_icon(APP_ICON, &icon_path).is_err() {
        println!("cargo:warning=failed to create application icon resource");
        return;
    }

    let escaped_icon = icon_path.to_string_lossy().replace('\\', "\\\\");
    if fs::write(&rc_path, format!("1 ICON \"{escaped_icon}\"\n")).is_err() {
        println!("cargo:warning=failed to write application icon resource script");
        return;
    }

    let status = Command::new("windres")
        .arg(&rc_path)
        .arg("-O")
        .arg("coff")
        .arg("-o")
        .arg(&res_path)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("cargo:rustc-link-arg-bin=aissistant={}", res_path.display());
        }
        _ => println!("cargo:warning=windres was not available; exe icon was not embedded"),
    }
}

fn create_icon(source: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), String> {
    let image = image::open(source).map_err(|err| err.to_string())?;
    let resized = image.resize_exact(256, 256, FilterType::Lanczos3);

    let mut png = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|err| err.to_string())?;

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

    fs::write(output, ico).map_err(|err| err.to_string())
}
