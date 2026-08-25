//! Build script to embed Windows Application Icon and Administrator Elevation Manifest.

use std::fs;
use std::path::Path;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let assets_dir = Path::new("assets");
        if !assets_dir.exists() {
            let _ = fs::create_dir_all(assets_dir);
        }

        let icon_path = assets_dir.join("icon.ico");
        generate_ico_file(&icon_path);

        let mut res = winres::WindowsResource::new();
        if let Some(p_str) = icon_path.to_str() {
            res.set_icon(p_str);
        } else {
            res.set_icon("assets/icon.ico");
        }

        // UAC Administrator Elevation Manifest for Release builds
        let profile = std::env::var("PROFILE").unwrap_or_default();
        let uac_level = if profile == "release" {
            "requireAdministrator"
        } else {
            "asInvoker"
        };

        let manifest = format!(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="{uac_level}" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
<application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
        <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
        <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </windowsSettings>
</application>
</assembly>
"#
        );
        res.set_manifest(&manifest);

        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resource: {e}");
        }
    }
}

/// Generates a valid standard 32-bit Windows ICO file.
fn generate_ico_file(path: &Path) {
    const SIZES: &[usize] = &[16, 32, 48, 64];
    let mut images_data = Vec::new();

    for &size in SIZES {
        let bmp_data = generate_icon_bmp(size);
        images_data.push((size, bmp_data));
    }

    let count = images_data.len() as u16;
    let mut ico = Vec::new();

    // ICONDIR header
    ico.extend_from_slice(&0u16.to_le_bytes()); // Reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // Type 1 = Icon
    ico.extend_from_slice(&count.to_le_bytes()); // Number of images

    let mut offset = 6 + (count as usize * 16);

    for (size, data) in &images_data {
        let b_size = if *size >= 256 { 0u8 } else { *size as u8 };
        ico.push(b_size); // Width
        ico.push(b_size); // Height
        ico.push(0); // Color count
        ico.push(0); // Reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // Color planes
        ico.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
        ico.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Size of image data
        ico.extend_from_slice(&(offset as u32).to_le_bytes()); // Offset
        offset += data.len();
    }

    for (_, data) in images_data {
        ico.extend_from_slice(&data);
    }

    let _ = fs::write(path, ico);
}

/// Generates BITMAPINFOHEADER + BGRA raw image data for an icon of given size.
fn generate_icon_bmp(size: usize) -> Vec<u8> {
    let mut data = Vec::new();
    let header_size: u32 = 40;
    let width = size as i32;
    let height = (size * 2) as i32; // Double height for icon mask
    let planes: u16 = 1;
    let bpp: u16 = 32;
    let image_size = (size * size * 4) as u32;

    data.extend_from_slice(&header_size.to_le_bytes());
    data.extend_from_slice(&width.to_le_bytes());
    data.extend_from_slice(&height.to_le_bytes());
    data.extend_from_slice(&planes.to_le_bytes());
    data.extend_from_slice(&bpp.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    data.extend_from_slice(&image_size.to_le_bytes());
    data.extend_from_slice(&0i32.to_le_bytes());
    data.extend_from_slice(&0i32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    // Write pixels bottom-to-top in BGRA format
    for y_inv in 0..size {
        let y = size - 1 - y_inv;
        for x in 0..size {
            let (r, g, b, a) = calculate_icon_pixel(x, y, size);
            data.push(b);
            data.push(g);
            data.push(r);
            data.push(a);
        }
    }

    // 1-bit AND mask (empty/all transparent since 32-bit uses alpha channel)
    let mask_row_bytes = size.div_ceil(32) * 4;
    for _ in 0..size {
        data.extend(std::iter::repeat_n(0u8, mask_row_bytes));
    }

    data
}

fn calculate_icon_pixel(x: usize, y: usize, size: usize) -> (u8, u8, u8, u8) {
    let scale = (size as f32) / 64.0;
    let fx = (x as f32) / scale;
    let fy = (y as f32) / scale;

    let in_chip = (6.0..=57.0).contains(&fx) && (6.0..=57.0).contains(&fy);
    let is_corner_cut = (fx < 12.0 && fy < 12.0 && (fx - 12.0).powi(2) + (fy - 12.0).powi(2) > 36.0)
        || (fx > 51.0 && fy < 12.0 && (fx - 51.0).powi(2) + (fy - 12.0).powi(2) > 36.0)
        || (fx < 12.0 && fy > 51.0 && (fx - 12.0).powi(2) + (fy - 51.0).powi(2) > 36.0)
        || (fx > 51.0 && fy > 51.0 && (fx - 51.0).powi(2) + (fy - 51.0).powi(2) > 36.0);

    let is_pin_x = (fx < 6.0 || fx > 57.0) && (14.0..=49.0).contains(&fy) && (y / 5).is_multiple_of(2);
    let is_pin_y = (fy < 6.0 || fy > 57.0) && (14.0..=49.0).contains(&fx) && (x / 5).is_multiple_of(2);

    let is_m = ((18.0..=22.0).contains(&fx) && (20.0..=44.0).contains(&fy))
        || ((42.0..=46.0).contains(&fx) && (20.0..=44.0).contains(&fy))
        || ((22.0..=32.0).contains(&fx) && (fy - (20.0 + (fx - 22.0) * 1.3)).abs() <= 2.2 && fy <= 34.0)
        || ((32.0..=42.0).contains(&fx) && (fy - (33.0 - (fx - 32.0) * 1.3)).abs() <= 2.2 && fy <= 34.0);

    let is_core_glow = (fx - 32.0).powi(2) + (fy - 37.0).powi(2) <= 10.0;

    if is_m {
        (0, 220, 255, 255)
    } else if is_core_glow {
        (16, 225, 140, 255)
    } else if in_chip && !is_corner_cut {
        if fx <= 8.0 || fx >= 55.0 || fy <= 8.0 || fy >= 55.0 {
            (0, 160, 230, 255)
        } else {
            let grad = (fy / 64.0 * 20.0) as u8;
            (20 + grad, 25 + grad, 36 + grad, 255)
        }
    } else if is_pin_x || is_pin_y {
        (245, 175, 45, 255)
    } else {
        (0, 0, 0, 0)
    }
}
