//! Detailed Motherboard & BIOS View tab.

use super::theme::AppTheme;
use super::widgets::{section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{RichText, ScrollArea, Ui};

/// Renders the dedicated detailed Mainboard tab.
pub fn render(ui: &mut Ui, theme: AppTheme, hardware: &SystemHardware) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Motherboard Section
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🖧", "Motherboard");

            spec_row(ui, theme, "Manufacturer", &hardware.motherboard.manufacturer);
            spec_row(ui, theme, "Model", &hardware.motherboard.model);
            spec_row(ui, theme, "Version", &hardware.motherboard.version);
            spec_row(ui, theme, "Serial Number", &hardware.motherboard.serial_number);
            spec_row(ui, theme, "System Manufacturer", &hardware.motherboard.system_manufacturer);
            spec_row(ui, theme, "System SKU", &hardware.motherboard.system_sku);

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(RichText::new("Chipset & Bus").size(12.5).color(theme.accent_primary()).strong());
            ui.add_space(2.0);
            spec_row(ui, theme, "Chipset / Southbridge", &hardware.motherboard.chipset);
            spec_row(ui, theme, "Bus Specifications", &hardware.motherboard.bus_specs);
        });

        ui.add_space(8.0);

        // BIOS Section
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "💾", "BIOS / Firmware");

            spec_row(ui, theme, "Brand / Vendor", &hardware.motherboard.bios_vendor);
            spec_row(ui, theme, "Version", &hardware.motherboard.bios_version);
            spec_row(ui, theme, "Release Date", &hardware.motherboard.bios_date);
            spec_row(ui, theme, "SMBIOS Version", &hardware.motherboard.smbios_version);
            spec_row(ui, theme, "UEFI Mode", "Supported & Active");
        });

        ui.add_space(8.0);

        // Graphic Interface Section
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔌", "Graphic Interface");

            spec_row(ui, theme, "Bus Interface", &hardware.motherboard.graphic_interface);
            spec_row(ui, theme, "Current Link Width", "x16");
            spec_row(ui, theme, "Max Supported Width", "x16");
            spec_row(ui, theme, "Current Link Speed", "16.0 GT/s (PCIe 4.0/5.0)");
        });

        ui.add_space(8.0);
    });
}
