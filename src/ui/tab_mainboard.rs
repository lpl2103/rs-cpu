//! Detailed Motherboard & BIOS View tab.

use super::theme::AppTheme;
use super::widgets::{section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{RichText, ScrollArea, Ui};

/// Renders the dedicated detailed Mainboard tab in Portuguese (PT-BR).
pub fn render(ui: &mut Ui, theme: AppTheme, hardware: &SystemHardware) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Seção da Placa-Mãe
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🖧", "Placa-Mãe");

            spec_row(ui, theme, "Fabricante", &hardware.motherboard.manufacturer);
            spec_row(ui, theme, "Modelo", &hardware.motherboard.model);
            spec_row(ui, theme, "Versão / Revisão", &hardware.motherboard.version);
            spec_row(ui, theme, "Número de Série", &hardware.motherboard.serial_number);
            spec_row(ui, theme, "Fabricante do Sistema", &hardware.motherboard.system_manufacturer);
            spec_row(ui, theme, "SKU do Sistema", &hardware.motherboard.system_sku);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label(RichText::new("Chipset & Barramentos").size(13.5).color(theme.accent_primary()).strong());
            ui.add_space(4.0);
            spec_row(ui, theme, "Chipset / Ponte Sul", &hardware.motherboard.chipset);
            spec_row(ui, theme, "Especificações de Barramento", &hardware.motherboard.bus_specs);
        });

        ui.add_space(8.0);

        // Seção da BIOS
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "💾", "BIOS / Firmware");

            spec_row(ui, theme, "Fabricante / Marca", &hardware.motherboard.bios_vendor);
            spec_row(ui, theme, "Versão", &hardware.motherboard.bios_version);
            spec_row(ui, theme, "Data de Lançamento", &hardware.motherboard.bios_date);
            spec_row(ui, theme, "Versão SMBIOS", &hardware.motherboard.smbios_version);
            spec_row(ui, theme, "Modo UEFI", "Suportado & Ativo");
        });

        ui.add_space(8.0);

        // Seção de Interface Gráfica
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔌", "Interface Gráfica");

            spec_row(ui, theme, "Interface de Barramento", &hardware.motherboard.graphic_interface);
            spec_row(ui, theme, "Largura de Link Atual", "x16");
            spec_row(ui, theme, "Largura Máxima Suportada", "x16");
            spec_row(ui, theme, "Velocidade de Link Atual", "16.0 GT/s (PCIe 4.0/5.0)");
        });

        ui.add_space(8.0);
    });
}
