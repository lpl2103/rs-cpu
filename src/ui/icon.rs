//! Application Icon Generation for M-CPU.

use eframe::egui::IconData;

/// Creates a custom 64x64 RGBA application icon representing M-CPU.
#[must_use]
pub fn create_app_icon() -> IconData {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 64;
    let mut rgba = Vec::with_capacity(WIDTH * HEIGHT * 4);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let fx = x as f32;
            let fy = y as f32;

            // Background rounded rectangle (chip body)
            let in_chip = (6.0..=57.0).contains(&fx) && (6.0..=57.0).contains(&fy);
            let is_corner_cut = (fx < 12.0 && fy < 12.0 && (fx - 12.0).powi(2) + (fy - 12.0).powi(2) > 36.0)
                || (fx > 51.0 && fy < 12.0 && (fx - 51.0).powi(2) + (fy - 12.0).powi(2) > 36.0)
                || (fx < 12.0 && fy > 51.0 && (fx - 12.0).powi(2) + (fy - 51.0).powi(2) > 36.0)
                || (fx > 51.0 && fy > 51.0 && (fx - 51.0).powi(2) + (fy - 51.0).powi(2) > 36.0);

            // Pins on outer edges
            let is_pin_x = (fx < 6.0 || fx > 57.0) && (14.0..=49.0).contains(&fy) && (y / 5).is_multiple_of(2);
            let is_pin_y = (fy < 6.0 || fy > 57.0) && (14.0..=49.0).contains(&fx) && (x / 5).is_multiple_of(2);

            // Letter 'M' drawing logic in the center
            let is_m = ((18.0..=22.0).contains(&fx) && (20.0..=44.0).contains(&fy))
                || ((42.0..=46.0).contains(&fx) && (20.0..=44.0).contains(&fy))
                || ((22.0..=32.0).contains(&fx) && (fy - (20.0 + (fx - 22.0) * 1.3)).abs() <= 2.2 && fy <= 34.0)
                || ((32.0..=42.0).contains(&fx) && (fy - (33.0 - (fx - 32.0) * 1.3)).abs() <= 2.2 && fy <= 34.0);

            // Central core glow dot
            let is_core_glow = (fx - 32.0).powi(2) + (fy - 37.0).powi(2) <= 10.0;

            if is_m {
                // Neon Cyan 'M'
                rgba.extend_from_slice(&[0, 220, 255, 255]);
            } else if is_core_glow {
                // Emerald accent glow
                rgba.extend_from_slice(&[16, 225, 140, 255]);
            } else if in_chip && !is_corner_cut {
                // Sleek dark metallic chip surface
                if fx <= 8.0 || fx >= 55.0 || fy <= 8.0 || fy >= 55.0 {
                    // Border stroke
                    rgba.extend_from_slice(&[0, 160, 230, 255]);
                } else {
                    let grad = (fy / 64.0 * 20.0) as u8;
                    rgba.extend_from_slice(&[20 + grad, 25 + grad, 36 + grad, 255]);
                }
            } else if is_pin_x || is_pin_y {
                // Gold / Copper connector pins
                rgba.extend_from_slice(&[245, 175, 45, 255]);
            } else {
                // Transparent
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    IconData {
        rgba,
        width: WIDTH as u32,
        height: HEIGHT as u32,
    }
}
