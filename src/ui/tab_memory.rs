//! Detailed Memory & SPD View tab.

use super::theme::AppTheme;
use super::widgets::{load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::memory::{RamStressManager, RamStressStatus};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Button, Color32, CornerRadius, RichText, ScrollArea, Ui};

/// State for the memory & SPD tab view.
#[derive(Debug, Clone, Default)]
pub struct MemoryTabState {
    /// Currently selected SPD slot index.
    pub selected_slot: usize,
    /// RAM stability stress test manager.
    pub stress: RamStressManager,
}

/// Renders the dedicated detailed Memory & SPD tab in Portuguese (PT-BR).
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut MemoryTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Informações Gerais da Memória
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "💾", "Memória Geral");

            let total_gb = (hardware.memory.total_mb as f32) / 1024.0;
            spec_row(ui, theme, "Tipo", &hardware.memory.memory_type);
            spec_row(ui, theme, "Capacidade Total", &format!("{total_gb:.1} GBytes ({:.0} MB)", hardware.memory.total_mb));
            spec_row(ui, theme, "Canais Ativos", &hardware.memory.channel_mode);
            spec_row(ui, theme, "Modo DC", "Simétrico");
            spec_row(ui, theme, "Frequência Uncore", &format!("{:.1} MHz", hardware.memory.uncore_frequency_mhz));

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label(RichText::new("Timings Primários").size(13.5).color(theme.accent_primary()).strong());
            ui.add_space(4.0);
            spec_row(ui, theme, "Frequência DRAM", &format!("{:.1} MHz", hardware.memory.dram_frequency_mhz));
            spec_row(ui, theme, "Proporção FSB:DRAM", &hardware.memory.fsb_dram_ratio);
            spec_row(ui, theme, "Latência CAS# (CL)", &format!("{} clocks", hardware.memory.cl));
            spec_row(ui, theme, "Atraso RAS# para CAS# (tRCD)", &format!("{} clocks", hardware.memory.trcd));
            spec_row(ui, theme, "Pré-carga RAS# (tRP)", &format!("{} clocks", hardware.memory.trp));
            spec_row(ui, theme, "Tempo de Ciclo (tRAS)", &format!("{} clocks", hardware.memory.tras));
            spec_row(ui, theme, "Ciclo de Banco (tRC)", &format!("{} clocks", hardware.memory.trc));
            spec_row(ui, theme, "Taxa de Comando (CR)", &hardware.memory.command_rate);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            load_gauge(
                ui,
                theme,
                "Uso de Memória RAM",
                hardware.memory.live.usage_pct,
                &format!("{} MB usados / {} MB total ({:.1}%)", hardware.memory.live.used_mb, hardware.memory.total_mb, hardware.memory.live.usage_pct),
            );
        });

        ui.add_space(8.0);

        // --- NOVO PAINEL: TESTE DE ESTABILIDADE & INTEGRIDADE DE MEMÓRIA RAM (MEMTEST) ---
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🧠", "Teste de Estabilidade & Integridade de Memória (Estilo MemTest)");

            let status = state.stress.status.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
            let results = state.stress.results.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
            let is_running = matches!(status, RamStressStatus::Running { .. });

            ui.columns(4, |cols| {
                cols[0].vertical(|ui| {
                    stat_metric_box(ui, theme, "Alocação do Teste", &format!("{} MB", results.allocated_mb), "Memória Sob Teste");
                });
                cols[1].vertical(|ui| {
                    stat_metric_box(ui, theme, "Total Verificado", &format!("{} MB", results.total_verified_mb), &format!("{} Ciclos", results.cycles_completed));
                });
                cols[2].vertical(|ui| {
                    stat_metric_box(ui, theme, "Taxa de Transferência", &format!("{:.0} MB/s", results.speed_mbs), "Throughput de Teste");
                });
                cols[3].vertical(|ui| {
                    let err_color_text = if results.error_count == 0 { "100% Estável (0 Erros)" } else { "Crítico: Falha de Memória" };
                    stat_metric_box(ui, theme, "Erros de Bit-Flip", &format!("{}", results.error_count), err_color_text);
                });
            });

            ui.add_space(10.0);

            if let RamStressStatus::Running { pattern_name, cycle_progress } = &status {
                ui.label(RichText::new(pattern_name).size(13.5).color(theme.accent_primary()).strong());
                ui.add_space(4.0);
                load_gauge(ui, theme, "Progresso do Ciclo Atual", cycle_progress * 100.0, &format!("{:.0}%", cycle_progress * 100.0));
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new("Tamanho da Alocação:").size(13.5).color(theme.text_secondary()));

                let size_options = [
                    (512, "512 MB (Rápido)"),
                    (1024, "1024 MB (1 GB)"),
                    (2048, "2048 MB (2 GB)"),
                    (4096, "4096 MB (4 GB - Intenso)"),
                ];

                let current_label = size_options
                    .iter()
                    .find(|(mb, _)| *mb == state.stress.selected_mb)
                    .map_or("1024 MB", |(_, l)| *l);

                ui.add_enabled_ui(!is_running, |ui| {
                    egui::ComboBox::from_id_salt("ram_stress_size")
                        .selected_text(RichText::new(current_label).size(13.0).color(theme.text_primary()))
                        .show_ui(ui, |ui| {
                            for (mb, label) in size_options {
                                ui.selectable_value(&mut state.stress.selected_mb, mb, label);
                            }
                        });
                });

                ui.add_space(10.0);

                if is_running {
                    let cancel_btn = Button::new(RichText::new("⏹ Parar Teste de RAM").strong().size(13.5).color(Color32::WHITE))
                        .fill(Color32::from_rgb(220, 38, 38))
                        .corner_radius(CornerRadius::same(6))
                        .min_size(egui::vec2(160.0, 32.0));

                    if ui.add(cancel_btn).clicked() {
                        state.stress.cancel();
                    }
                } else {
                    let start_btn = Button::new(RichText::new("🚀 Iniciar Teste de Estabilidade").strong().size(13.5).color(Color32::WHITE))
                        .fill(Color32::from_rgb(16, 160, 90))
                        .corner_radius(CornerRadius::same(6))
                        .min_size(egui::vec2(220.0, 32.0));

                    if ui.add(start_btn).clicked() {
                        state.stress.start_test(state.stress.selected_mb);
                    }
                }
            });
        });

        ui.add_space(8.0);

        // Seção SPD (Serial Presence Detect)
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔍", "SPD dos Slots de Memória (Serial Presence Detect)");

            // Seletor de Slot
            ui.horizontal(|ui| {
                ui.label(RichText::new("Seleção do Slot de Memória:").size(13.5).color(theme.text_secondary()));
                
                let slots_count = hardware.memory.slots.len();
                if state.selected_slot >= slots_count {
                    state.selected_slot = 0;
                }

                let current_label = hardware.memory.slots.get(state.selected_slot).map_or_else(
                    || "Slot #1".to_string(),
                    |s| {
                        if s.size_mb > 0 {
                            format!("{} - {} ({} MB)", s.slot_name, s.memory_type, s.size_mb)
                        } else {
                            format!("{} - [Vazio]", s.slot_name)
                        }
                    },
                );

                egui::ComboBox::from_id_salt("spd_slot_select")
                    .selected_text(RichText::new(current_label).size(13.0).color(theme.text_primary()))
                    .show_ui(ui, |ui| {
                        for (idx, slot) in hardware.memory.slots.iter().enumerate() {
                            let label = if slot.size_mb > 0 {
                                format!("{} - {} ({} MB)", slot.slot_name, slot.memory_type, slot.size_mb)
                            } else {
                                format!("{} - [Vazio]", slot.slot_name)
                            };
                            ui.selectable_value(&mut state.selected_slot, idx, label);
                        }
                    });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            if let Some(slot) = hardware.memory.slots.get(state.selected_slot) {
                if slot.size_mb > 0 {
                    ui.columns(2, |cols| {
                        cols[0].vertical(|ui| {
                            spec_row(ui, theme, "Tamanho do Módulo", &format!("{} MBytes ({} GB)", slot.size_mb, slot.size_mb / 1024));
                            spec_row(ui, theme, "Largura de Banda Máx", &format!("{}-{} ({} MHz)", slot.memory_type, slot.speed_mhz, slot.speed_mhz / 2));
                            spec_row(ui, theme, "Velocidade Configurada", &format!("{} MT/s", slot.configured_speed_mhz));
                            spec_row(ui, theme, "Fabricante do Módulo", &slot.module_manufacturer);
                            spec_row(ui, theme, "Fabricante DRAM", &slot.dram_manufacturer);
                        });

                        cols[1].vertical(|ui| {
                            spec_row(ui, theme, "Part Number", &slot.part_number);
                            spec_row(ui, theme, "Número de Série", &slot.serial_number);
                            spec_row(ui, theme, "Formato", &slot.form_factor);
                            spec_row(ui, theme, "Tensão Nominal", &format!("{:.2} V", slot.voltage));
                            spec_row(ui, theme, "Localizador do Banco", &slot.bank_locator);
                        });
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Tabela de Timings
                    ui.label(RichText::new("Tabela de Timings (Perfis JEDEC / XMP / EXPO)").size(13.5).color(theme.accent_primary()).strong());
                    ui.add_space(6.0);

                    egui::Grid::new("timings_table_grid")
                        .striped(true)
                        .min_col_width(75.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Perfil").strong().color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(&p.name).strong().color(theme.accent_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("Frequência").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{} MHz", p.frequency_mhz)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("Latência CAS#").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.cas_latency)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("RAS# para CAS#").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.trcd)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("Pré-carga RAS#").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.trp)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("tRAS").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.tras)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("tRC").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.trc)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("Tensão").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{:.2} V", p.voltage)).color(theme.text_primary()));
                            }
                            ui.end_row();
                        });
                } else {
                    ui.label(RichText::new("O slot selecionado está vazio / desocupado.").color(theme.text_secondary()));
                }
            }
        });

        ui.add_space(8.0);
    });
}
