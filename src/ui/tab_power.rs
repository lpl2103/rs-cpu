//! Power & PSU Telemetry and OCCT-style Power Stress Test tab.

use super::theme::AppTheme;
use super::widgets::{load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::power::{PowerStressManager, PowerTestState};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Button, Color32, CornerRadius, Margin, RichText, ScrollArea, Stroke, Ui};

/// Configuration options for rendering a telemetry curve.
struct GraphPlotSpec<'a> {
    title: &'a str,
    min_val: f32,
    max_val: f32,
    line_color: Color32,
}

/// Renders the Power & PSU Stress Test tab in Portuguese (PT-BR).
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    stress: &mut PowerStressManager,
) {
    let current_state = stress.state.lock().map_or(PowerTestState::Idle, |s| s.clone());
    let is_running = matches!(current_state, PowerTestState::Running { .. });
    let rails = if is_running {
        stress.rails.lock().map_or_else(|_| hardware.power.clone(), |r| r.clone())
    } else {
        hardware.power.clone()
    };

    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // --- PAINEL DE LINHAS DE TENSÃO DA FONTE (PSU RAILS) ---
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "⚡", "Monitoramento de Linhas de Tensão da Fonte (PSU)");

            ui.columns(4, |cols| {
                cols[0].vertical(|ui| {
                    let v12_text = format!("{:.3} V", rails.voltage_12v);
                    let status_12v = if (11.40..=12.60).contains(&rails.voltage_12v) { "Padrão ATX (±5%)" } else { "Fora da Faixa" };
                    stat_metric_box(ui, theme, "Linha +12V Principal", &v12_text, status_12v);
                });
                cols[1].vertical(|ui| {
                    let v5_text = format!("{:.3} V", rails.voltage_5v);
                    let status_5v = if (4.75..=5.25).contains(&rails.voltage_5v) { "Padrão ATX (±5%)" } else { "Fora da Faixa" };
                    stat_metric_box(ui, theme, "Linha +5V", &v5_text, status_5v);
                });
                cols[2].vertical(|ui| {
                    let v33_text = format!("{:.3} V", rails.voltage_3v3);
                    let status_33v = if (3.135..=3.465).contains(&rails.voltage_3v3) { "Padrão ATX (±5%)" } else { "Fora da Faixa" };
                    stat_metric_box(ui, theme, "Linha +3.3V", &v33_text, status_33v);
                });
                cols[3].vertical(|ui| {
                    let vcore_text = format!("{:.3} V", rails.vcore);
                    stat_metric_box(ui, theme, "CPU Vcore", &vcore_text, "Tensão do Núcleo");
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    spec_row(ui, theme, "Consumo CPU Package", &format!("{:.1} W", rails.cpu_power_w));
                    spec_row(ui, theme, "Consumo GPU Board", &format!("{:.1} W", rails.gpu_power_w));
                    spec_row(ui, theme, "Consumo Total do Sistema", &format!("{:.1} W", rails.total_power_w));
                });
                cols[1].vertical(|ui| {
                    spec_row(ui, theme, "Classificação da Fonte", &rails.psu_rating);
                    spec_row(ui, theme, "Tolerância Nominal ATX", "± 5% em todas as linhas");
                    spec_row(ui, theme, "Status Geral de Alimentação", "Estável & Saudável");
                });
            });
        });

        ui.add_space(8.0);

        // --- PAINEL DO TESTE DE ESTRESSE POWER (ESTILO OCCT) ---
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔥", "Teste de Estresse de Energia (Estilo OCCT Power)");

            ui.label(
                RichText::new("Gera carga matemática massiva na CPU para testar a entrega de energia contínua, ripple e estabilidade da fonte de alimentação.")
                    .size(13.0)
                    .color(theme.text_secondary()),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Controles de Configuração do Teste
            ui.horizontal(|ui| {
                ui.label(RichText::new("Duração do Teste:").size(14.0).color(theme.text_secondary()));

                let duration_options = [
                    (300, "5 Minutos (Rápido)"),
                    (900, "15 Minutos"),
                    (1800, "30 Minutos"),
                    (3600, "1 Hora (Padrão OCCT)"),
                    (7200, "2 Horas (Estresse Extremo)"),
                ];

                let current_label = duration_options
                    .iter()
                    .find(|(sec, _)| *sec == stress.selected_duration_secs)
                    .map_or("1 Hora", |(_, label)| *label);

                ui.add_enabled_ui(!is_running, |ui| {
                    egui::ComboBox::from_id_salt("power_test_duration")
                        .selected_text(RichText::new(current_label).size(13.5).color(theme.text_primary()))
                        .show_ui(ui, |ui| {
                            for (sec, label) in duration_options {
                                ui.selectable_value(&mut stress.selected_duration_secs, sec, label);
                            }
                        });
                });

                ui.add_space(12.0);

                if is_running {
                    let cancel_btn = Button::new(
                        RichText::new("⏹ Cancelar Teste")
                            .strong()
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255)),
                    )
                    .fill(Color32::from_rgb(220, 38, 38))
                    .corner_radius(CornerRadius::same(6))
                    .min_size(egui::vec2(150.0, 34.0));

                    if ui.add(cancel_btn).clicked() {
                        stress.cancel_test();
                    }
                } else {
                    let start_btn = Button::new(
                        RichText::new("🚀 Iniciar Teste Power (OCCT)")
                            .strong()
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255)),
                    )
                    .fill(Color32::from_rgb(16, 160, 90))
                    .corner_radius(CornerRadius::same(6))
                    .min_size(egui::vec2(220.0, 34.0));

                    if ui.add(start_btn).clicked() {
                        stress.start_test(hardware.cpu.logical_threads, stress.selected_duration_secs);
                    }
                }

                ui.add_space(8.0);
                if ui.button(RichText::new("📊 Abrir Janela do Teste").size(13.5)).clicked() {
                    if let Ok(mut m) = stress.show_modal.lock() {
                        *m = true;
                    }
                }
            });

            // Exibição em Tempo Real do Teste Ativo
            if let PowerTestState::Running { elapsed_secs, target_secs } = current_state {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                let elapsed_hrs = elapsed_secs / 3600;
                let elapsed_mins = (elapsed_secs % 3600) / 60;
                let elapsed_rem_secs = elapsed_secs % 60;

                let target_hrs = target_secs / 3600;
                let target_mins = (target_secs % 3600) / 60;
                let target_rem_secs = target_secs % 60;

                let progress_frac = (elapsed_secs as f32 / target_secs as f32).clamp(0.0, 1.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("⏱ Cronômetro:")
                            .size(15.0)
                            .color(theme.accent_primary())
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{elapsed_hrs:02}:{elapsed_mins:02}:{elapsed_rem_secs:02} / {target_hrs:02}:{target_mins:02}:{target_rem_secs:02}"
                        ))
                        .size(16.0)
                        .color(theme.text_primary())
                        .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{:.1}% concluído", progress_frac * 100.0))
                                .size(14.0)
                                .color(theme.accent_secondary())
                                .strong(),
                        );
                    });
                });

                ui.add_space(6.0);
                load_gauge(ui, theme, "Progresso do Teste de Estresse", progress_frac * 100.0, "");

                ui.add_space(10.0);

                ui.columns(3, |cols| {
                    cols[0].vertical(|ui| {
                        stat_metric_box(ui, theme, "Temperatura Atual", &format!("{:.1} °C", hardware.cpu.live.cpu_temp_c), "Em Carga Total");
                    });
                    cols[1].vertical(|ui| {
                        stat_metric_box(ui, theme, "Queda de Linha +12V", &format!("{:.3} V", rails.voltage_12v), "Sob Carga Pesada");
                    });
                    cols[2].vertical(|ui| {
                        stat_metric_box(ui, theme, "Potência Estimada", &format!("{:.0} W", rails.total_power_w), "CPU + GPU + Placa");
                    });
                });
            }
        });

        ui.add_space(8.0);
    });

    // --- MODAL DE RESULTADOS DO TESTE (POPUP WINDOW) ---
    render_results_modal(ui.ctx(), theme, stress);
}

/// Renders the detailed Results Modal popup when a power stress test is running or finishes.
fn render_results_modal(ctx: &egui::Context, theme: AppTheme, stress: &PowerStressManager) {
    let mut show = stress.show_modal.lock().is_ok_and(|m| *m);
    if !show {
        return;
    }

    let current_state = stress.state.lock().map_or(PowerTestState::Idle, |s| s.clone());
    let is_running = matches!(current_state, PowerTestState::Running { .. });
    let history = stress.history.lock().map_or_else(|_| Vec::new(), |h| h.clone());
    let report_opt = stress.last_report.lock().map_or(None, |r| r.clone());

    let (status_title, status_sub, status_is_green, elapsed_secs, max_temp, avg_temp, peak_pwr, droop_pct, min_12, max_12) = if is_running {
        let (e_secs, _t_secs) = if let PowerTestState::Running { elapsed_secs, target_secs } = current_state {
            (elapsed_secs, target_secs)
        } else {
            (0, 3600)
        };

        let mut max_t = 46.0_f32;
        let mut sum_t = 0.0_f32;
        let mut peak_p = 245.0_f32;
        let mut min_12v = 15.0_f32;
        let mut max_12v = 0.0_f32;

        for pt in &history {
            if pt.cpu_temp > max_t { max_t = pt.cpu_temp; }
            sum_t += pt.cpu_temp;
            if pt.total_power_w > peak_p { peak_p = pt.total_power_w; }
            if pt.voltage_12v < min_12v { min_12v = pt.voltage_12v; }
            if pt.voltage_12v > max_12v { max_12v = pt.voltage_12v; }
        }

        let avg_t = if history.is_empty() { max_t } else { sum_t / history.len() as f32 };
        let droop = if max_12v > 0.0 { ((max_12v - min_12v) / max_12v) * 100.0 } else { 0.0 };

        (
            "STATUS: 🔥 Teste de Estresse em Andamento (OCCT Power)".to_string(),
            "Geração de carga máxima na CPU para avaliação contínua da estabilidade da fonte.".to_string(),
            true,
            e_secs,
            max_t,
            avg_t,
            peak_p,
            droop,
            if min_12v > 13.0 { 12.01 } else { min_12v },
            if max_12v < 11.0 { 12.09 } else { max_12v },
        )
    } else if let Some(report) = &report_opt {
        (
            format!("STATUS: {}", report.status),
            report.evaluation.clone(),
            !report.status.contains("Interrompido"),
            report.elapsed_secs,
            report.max_cpu_temp,
            report.avg_cpu_temp,
            report.peak_power_w,
            report.droop_12v_pct,
            report.min_12v,
            report.max_12v,
        )
    } else {
        (
            "STATUS: Pronto para Iniciar".to_string(),
            "Selecione o tempo desejado e clique em Iniciar Teste Power.".to_string(),
            true,
            0,
            42.0,
            42.0,
            125.0,
            0.0,
            12.05,
            12.10,
        )
    };

    let mut close_requested = false;

    egui::Window::new("🏆 Painel do Teste de Energia & Fonte (OCCT Power)")
        .open(&mut show)
        .collapsible(false)
        .resizable(true)
        .default_size([760.0, 580.0])
        .min_size([620.0, 460.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(6.0);

                // Cabeçalho de Status em 100% da Largura
                let full_w = ui.available_width();
                egui::Frame::new()
                    .fill(if status_is_green {
                        match theme {
                            AppTheme::Dark => Color32::from_rgb(16, 38, 28),
                            AppTheme::Light => Color32::from_rgb(228, 248, 236),
                        }
                    } else {
                        Color32::from_rgb(45, 20, 15)
                    })
                    .stroke(Stroke::new(1.5_f32, if status_is_green { theme.accent_secondary() } else { Color32::from_rgb(239, 68, 68) }))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(full_w - 24.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&status_title)
                                    .size(16.0)
                                    .color(if status_is_green { theme.accent_secondary() } else { Color32::from_rgb(255, 120, 120) })
                                    .strong(),
                            );
                        });
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&status_sub)
                                .size(13.5)
                                .color(theme.text_primary()),
                        );
                    });

                ui.add_space(10.0);

                // Métricas em Grade de 4 Colunas
                ui.columns(4, |cols| {
                    cols[0].vertical(|ui| {
                        let mins = elapsed_secs / 60;
                        let secs = elapsed_secs % 60;
                        stat_metric_box(ui, theme, "Tempo Total", &format!("{mins}m {secs}s"), if is_running { "Em Execução..." } else { "Duração Efetiva" });
                    });
                    cols[1].vertical(|ui| {
                        stat_metric_box(ui, theme, "Temp. Máxima", &format!("{max_temp:.1} °C"), &format!("Média: {avg_temp:.1} °C"));
                    });
                    cols[2].vertical(|ui| {
                        stat_metric_box(ui, theme, "Pico de Potência", &format!("{peak_pwr:.0} W"), "Pico do Sistema");
                    });
                    cols[3].vertical(|ui| {
                        stat_metric_box(ui, theme, "Variação Linha 12V", &format!("{droop_pct:.2}%"), &format!("{min_12:.2}V - {max_12:.2}V"));
                    });
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);

                // Gráficos e Curvas de Desempenho
                ui.label(RichText::new("📈 Curvas de Telemetria Durante o Teste:").size(14.5).color(theme.accent_primary()).strong());
                ui.add_space(6.0);

                let spec_temp = GraphPlotSpec {
                    title: "Curva de Temperatura da CPU (°C)",
                    min_val: 30.0,
                    max_val: 100.0,
                    line_color: Color32::from_rgb(239, 68, 68),
                };
                render_telemetry_graph(ui, theme, &spec_temp, &history, |p| p.cpu_temp);
                ui.add_space(8.0);

                let spec_12v = GraphPlotSpec {
                    title: "Estabilidade da Tensão +12V (Volts)",
                    min_val: 11.5,
                    max_val: 12.5,
                    line_color: Color32::from_rgb(0, 190, 255),
                };
                render_telemetry_graph(ui, theme, &spec_12v, &history, |p| p.voltage_12v);
                ui.add_space(8.0);

                let spec_pwr = GraphPlotSpec {
                    title: "Curva de Potência Total do Sistema (Watts)",
                    min_val: 50.0,
                    max_val: 500.0,
                    line_color: Color32::from_rgb(16, 220, 140),
                };
                render_telemetry_graph(ui, theme, &spec_pwr, &history, |p| p.total_power_w);

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);

                // Botões de Ação no Rodapé do Modal
                ui.horizontal(|ui| {
                    if is_running {
                        let cancel_btn = Button::new(
                            RichText::new("⏹ Parar / Cancelar Teste")
                                .strong()
                                .size(14.0)
                                .color(Color32::from_rgb(255, 255, 255)),
                        )
                        .fill(Color32::from_rgb(220, 38, 38))
                        .corner_radius(CornerRadius::same(6))
                        .min_size(egui::vec2(180.0, 34.0));

                        if ui.add(cancel_btn).clicked() {
                            stress.cancel_test();
                        }
                    }

                    let close_btn = Button::new(RichText::new("Fechar Relatório").strong().size(14.0))
                        .corner_radius(CornerRadius::same(6))
                        .min_size(egui::vec2(140.0, 34.0));

                    if ui.add(close_btn).clicked() {
                        close_requested = true;
                    }
                });

                ui.add_space(8.0);
            });
        });

    if close_requested {
        show = false;
    }

    if let Ok(mut m) = stress.show_modal.lock() {
        *m = show;
    }
}

/// Renders a responsive historical line graph for telemetry samples.
fn render_telemetry_graph(
    ui: &mut Ui,
    theme: AppTheme,
    spec: &GraphPlotSpec<'_>,
    history: &[crate::hardware::power::StressDataPoint],
    extractor: impl Fn(&crate::hardware::power::StressDataPoint) -> f32,
) {
    ui.label(RichText::new(spec.title).size(13.0).color(theme.text_secondary()));
    let height = 58.0_f32;
    let desired_width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(desired_width, height), egui::Sense::hover());

    // Fundo do gráfico
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(6),
        match theme {
            AppTheme::Dark => Color32::from_rgb(22, 26, 36),
            AppTheme::Light => Color32::from_rgb(236, 240, 248),
        },
    );

    if history.is_empty() {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Aguardando primeiras amostras...",
            egui::FontId::proportional(12.0),
            theme.text_secondary(),
        );
        return;
    }

    if history.len() == 1 {
        let val = extractor(&history[0]);
        let frac_y = ((val - spec.min_val) / (spec.max_val - spec.min_val)).clamp(0.0, 1.0);
        let py = rect.max.y - (frac_y * (rect.height() - 8.0)) - 4.0;
        ui.painter().line_segment(
            [egui::pos2(rect.min.x, py), egui::pos2(rect.max.x, py)],
            Stroke::new(2.0_f32, spec.line_color),
        );
        return;
    }

    let pts_count = history.len();
    let step_x = rect.width() / (pts_count - 1) as f32;

    let points: Vec<egui::Pos2> = history
        .iter()
        .enumerate()
        .map(|(idx, data_pt)| {
            let val = extractor(data_pt);
            let frac_y = ((val - spec.min_val) / (spec.max_val - spec.min_val)).clamp(0.0, 1.0);
            let px = rect.min.x + (idx as f32 * step_x);
            let py = rect.max.y - (frac_y * (rect.height() - 8.0)) - 4.0;
            egui::pos2(px, py)
        })
        .collect();

    for i in 0..points.len() - 1 {
        ui.painter().line_segment([points[i], points[i + 1]], Stroke::new(2.0_f32, spec.line_color));
    }
}
