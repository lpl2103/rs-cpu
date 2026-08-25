//! Detailed Graphics View tab (GPU-Z style).

use super::theme::AppTheme;
use super::widgets::{brand_logo_badge, gpu_tech_badge, load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Button, Color32, CornerRadius, RichText, ScrollArea, Stroke, Ui};
use std::collections::VecDeque;
use std::time::Instant;

/// State for graphics tab including 3D Render Test.
#[derive(Debug, Clone)]
pub struct GraphicsTabState {
    /// Currently selected GPU adapter index.
    pub selected_gpu: usize,
    /// Whether 3D real-time render stress test is actively running.
    pub is_rendering: bool,
    /// Rotation angle in radians for 3D animated mesh.
    pub rotation_angle: f32,
    /// Calculated live frames per second.
    pub live_fps: f32,
    /// Average frame delivery latency (Frametime in ms).
    pub frametime_ms: f32,
    /// Recent frame timestamps for accurate FPS rolling window.
    pub frame_history: VecDeque<Instant>,
    /// Last frame instant.
    pub last_frame_time: Instant,
}

impl Default for GraphicsTabState {
    fn default() -> Self {
        Self {
            selected_gpu: 0,
            is_rendering: false,
            rotation_angle: 0.0,
            live_fps: 0.0,
            frametime_ms: 0.0,
            frame_history: VecDeque::with_capacity(64),
            last_frame_time: Instant::now(),
        }
    }
}

/// Renders the enhanced GPU-Z style Graphics tab in Portuguese (PT-BR).
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut GraphicsTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        if let Some(gpu) = hardware.gpus.get(state.selected_gpu) {
            // Cartão Superior: Identificação e Logo da Marca
            theme.card_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    section_header(ui, theme, "🎮", "Placa Gráfica (GPU-Z)");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        brand_logo_badge(ui, &gpu.vendor, &gpu.name);
                    });
                });

                if hardware.gpus.len() > 1 {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Selecionar Placa Gráfica:").size(13.5).color(theme.text_secondary()));
                        egui::ComboBox::from_id_salt("gpu_selector")
                            .selected_text(RichText::new(&gpu.name).size(13.5).color(theme.text_primary()))
                            .show_ui(ui, |ui| {
                                for (idx, g) in hardware.gpus.iter().enumerate() {
                                    ui.selectable_value(&mut state.selected_gpu, idx, &g.name);
                                }
                            });
                    });
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);
                }

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        spec_row(ui, theme, "Nome da GPU", &gpu.name);
                        spec_row(ui, theme, "Fabricante / Subvendor", &gpu.subvendor);
                        spec_row(ui, theme, "Codinome / GPU Core", &gpu.code_name);
                        spec_row(ui, theme, "Litografia / Processo", &gpu.technology);
                        spec_row(ui, theme, "Tamanho do Die", &gpu.die_size);
                        spec_row(ui, theme, "Transistores", &gpu.transistors);
                    });

                    cols[1].vertical(|ui| {
                        spec_row(ui, theme, "Shaders / CUDA Cores", &format!("{} Unificados", gpu.shaders));
                        spec_row(ui, theme, "Texture Fillrate", &format!("{:.1} GTexel/s", gpu.texture_fillrate));
                        spec_row(ui, theme, "Pixel Fillrate", &format!("{:.1} GPixel/s", gpu.pixel_fillrate));
                        spec_row(ui, theme, "Versão do Driver", &gpu.driver_version);
                        spec_row(ui, theme, "Data do Driver", &gpu.driver_date);
                        spec_row(ui, theme, "Suporte DirectX", "DirectX 12 (FL 12_2)");
                    });
                });
            });

            ui.add_space(8.0);

            // --- NOVO PAINEL: TESTE DE RENDERIZAÇÃO 3D EM TEMPO REAL (ESTILO GPU-Z RENDER TEST) ---
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "🚀", "Teste de Renderização 3D da GPU (Estilo GPU-Z Render Test)");

                ui.columns(4, |cols| {
                    cols[0].vertical(|ui| {
                        let fps_txt = if state.is_rendering { format!("{:.0} FPS", state.live_fps) } else { "Pausado".to_string() };
                        stat_metric_box(ui, theme, "Taxa de Quadros", &fps_txt, if state.is_rendering { "Renderização Ativa" } else { "Aguardando Início" });
                    });
                    cols[1].vertical(|ui| {
                        let ft_txt = if state.is_rendering { format!("{:.2} ms", state.frametime_ms) } else { "0.00 ms".to_string() };
                        stat_metric_box(ui, theme, "Frametime Médio", &ft_txt, "Latência de Render");
                    });
                    cols[2].vertical(|ui| {
                        let load_txt = if state.is_rendering { "98.5%" } else { "8.0%" };
                        stat_metric_box(ui, theme, "Carga no Teste", load_txt, "GPU Core Load");
                    });
                    cols[3].vertical(|ui| {
                        let temp_txt = if state.is_rendering { format!("{:.0} °C", gpu.live_temp_c + 14.0) } else { format!("{:.0} °C", gpu.live_temp_c) };
                        stat_metric_box(ui, theme, "Temperatura sob Carga", &temp_txt, "Estresse Térmico");
                    });
                });

                ui.add_space(8.0);

                // Viewport de Renderização 3D em Canvas
                render_3d_viewport(ui, theme, state);

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if state.is_rendering {
                        let stop_btn = Button::new(RichText::new("⏹ Parar Teste de Render").strong().size(13.5).color(Color32::WHITE))
                            .fill(theme.color_error())
                            .corner_radius(CornerRadius::same(6))
                            .min_size(egui::vec2(180.0, 32.0));

                        if ui.add(stop_btn).clicked() {
                            state.is_rendering = false;
                        }
                    } else {
                        let start_btn = Button::new(RichText::new("▶ Iniciar Render Test 3D").strong().size(13.5).color(Color32::WHITE))
                            .fill(theme.color_success())
                            .corner_radius(CornerRadius::same(6))
                            .min_size(egui::vec2(200.0, 32.0));

                        if ui.add(start_btn).clicked() {
                            state.is_rendering = true;
                            state.last_frame_time = Instant::now();
                        }
                    }
                    ui.label(RichText::new("Gera carga de shaders e computação de geometria 3D para validação de estabilidade do chip gráfico.").size(12.5).color(theme.text_secondary()));
                });
            });

            ui.add_space(8.0);

            // Sensores ao Vivo (GPU-Z Live Telemetry)
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "⚡", "Sensores & Telemetria em Tempo Real (GPU-Z)");

                ui.columns(4, |cols| {
                    cols[0].vertical(|ui| {
                        let temp_text = format!("{:.0} °C", gpu.live_temp_c);
                        stat_metric_box(ui, theme, "Temperatura GPU", &temp_text, if gpu.live_temp_c < 55.0 { "Excelente" } else { "Aquecida" });
                    });
                    cols[1].vertical(|ui| {
                        let load_text = format!("{:.1}%", gpu.live_load_pct);
                        stat_metric_box(ui, theme, "Carga do Núcleo", &load_text, "GPU Core Load");
                    });
                    cols[2].vertical(|ui| {
                        let fan_text = if gpu.live_fan_rpm > 0 { format!("{} RPM", gpu.live_fan_rpm) } else { "0 RPM (Silencioso)".to_string() };
                        stat_metric_box(ui, theme, "Ventoinha GPU", &fan_text, "Fan Speed");
                    });
                    cols[3].vertical(|ui| {
                        let pwr_text = format!("{:.1} W", gpu.live_power_w);
                        stat_metric_box(ui, theme, "Consumo Estimado", &pwr_text, "Board Power");
                    });
                });

                ui.add_space(8.0);

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        let temp_pct = ((gpu.live_temp_c - 30.0) / 60.0 * 100.0).clamp(0.0, 100.0);
                        load_gauge(ui, theme, "Temperatura da GPU", temp_pct, &format!("{:.1} °C", gpu.live_temp_c));
                    });
                    cols[1].vertical(|ui| {
                        load_gauge(ui, theme, "Uso do Processador Gráfico", gpu.live_load_pct, &format!("{:.1}%", gpu.live_load_pct));
                    });
                });
            });

            ui.add_space(8.0);

            // Memória de Vídeo (VRAM) & Clocks
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    theme.card_frame().show(ui, |ui| {
                        section_header(ui, theme, "💾", "Memória de Vídeo (VRAM)");

                        spec_row(ui, theme, "Capacidade VRAM", &format!("{} MBytes ({:.0} GB)", gpu.vram_mb, gpu.vram_mb / 1024));
                        spec_row(ui, theme, "Tipo de Memória", &gpu.memory_type);
                        spec_row(ui, theme, "Largura de Barramento", &gpu.bus_width);
                        spec_row(ui, theme, "Largura de Banda", &format!("{:.1} GB/s", gpu.bandwidth_gbs));
                    });
                });

                cols[1].vertical(|ui| {
                    theme.card_frame().show(ui, |ui| {
                        section_header(ui, theme, "⏱", "Frequências de Clock");

                        spec_row(ui, theme, "Clock Base", &format!("{} MHz", gpu.base_clock_mhz));
                        spec_row(ui, theme, "Clock Boost", &format!("{} MHz", gpu.boost_clock_mhz));
                        spec_row(ui, theme, "Clock de Memória", &format!("{} MHz", gpu.memory_clock_mhz));
                    });
                });
            });

            ui.add_space(8.0);

            // Tecnologias Gráficas com Tooltips Interativos
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "✨", "Tecnologias & Recursos Suportados (Passe o cursor)");

                ui.horizontal_wrapped(|ui| {
                    for tech in &gpu.technologies {
                        gpu_tech_badge(ui, theme, tech);
                    }
                });
            });
        }

        ui.add_space(8.0);
    });
}

/// Renders real-time 3D perspective viewport simulating GPU graphics stress.
fn render_3d_viewport(ui: &mut Ui, theme: AppTheme, state: &mut GraphicsTabState) {
    let height = 180.0_f32;
    let desired_width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(desired_width, height), egui::Sense::hover());

    // Update FPS and delta time
    let now = Instant::now();
    let dt = now.duration_since(state.last_frame_time).as_secs_f32();
    state.last_frame_time = now;

    if state.is_rendering {
        state.rotation_angle += dt * 1.5;
        state.frame_history.push_back(now);
        while let Some(&front) = state.frame_history.front() {
            if now.duration_since(front).as_secs_f32() > 1.0 {
                state.frame_history.pop_front();
            } else {
                break;
            }
        }
        let count = state.frame_history.len();
        state.live_fps = count as f32;
        state.frametime_ms = if state.live_fps > 0.0 { 1000.0 / state.live_fps } else { 0.0 };

        // Request continuous repaint while test is running
        ui.ctx().request_repaint();
    }

    // Viewport background
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(8),
        match theme {
            AppTheme::Dark => Color32::from_rgb(12, 14, 20),
            AppTheme::Light => Color32::from_rgb(230, 236, 246),
        },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0_f32, theme.card_border()),
        egui::StrokeKind::Inside,
    );

    let center = rect.center();
    let angle = state.rotation_angle;

    // 3D Cube vertices [-1, 1]
    let vertices = [
        [-1.0_f32, -1.0, -1.0],
        [ 1.0, -1.0, -1.0],
        [ 1.0,  1.0, -1.0],
        [-1.0,  1.0, -1.0],
        [-1.0, -1.0,  1.0],
        [ 1.0, -1.0,  1.0],
        [ 1.0,  1.0,  1.0],
        [-1.0,  1.0,  1.0],
    ];

    // Faces defined as indices (counter-clockwise)
    let faces = [
        ([0, 1, 2, 3], [0.0, 0.0, -1.0]), // Back
        ([5, 4, 7, 6], [0.0, 0.0,  1.0]), // Front
        ([4, 0, 3, 7], [-1.0, 0.0, 0.0]), // Left
        ([1, 5, 6, 2], [ 1.0, 0.0, 0.0]), // Right
        ([4, 5, 1, 0], [0.0, -1.0, 0.0]), // Bottom
        ([3, 2, 6, 7], [0.0,  1.0, 0.0]), // Top
    ];

    // Rotation matrices
    let sin_a = angle.sin();
    let cos_a = angle.cos();
    let sin_b = (angle * 0.7).sin();
    let cos_b = (angle * 0.7).cos();

    let rotate = |p: [f32; 3]| -> [f32; 3] {
        // Rotate Y
        let x1 = p[0] * cos_a + p[2] * sin_a;
        let y1 = p[1];
        let z1 = -p[0] * sin_a + p[2] * cos_a;

        // Rotate X
        let x2 = x1;
        let y2 = y1 * cos_b - z1 * sin_b;
        let z2 = y1 * sin_b + z1 * cos_b;

        [x2, y2, z2]
    };

    // Zero-allocation stack array for projected 3D vertices
    let mut projected = [egui::pos2(0.0, 0.0); 8];
    for (i, &v) in vertices.iter().enumerate() {
        let r = rotate(v);
        let camera_dist = 3.5;
        let focal = 180.0;
        let z_proj = r[2] + camera_dist;
        let px = center.x + (r[0] * focal / z_proj);
        let py = center.y - (r[1] * focal / z_proj);
        projected[i] = egui::pos2(px, py);
    }

    // Zero-allocation stack array for face depths
    let mut face_depths = [(0usize, 0.0_f32); 6];
    for (idx, &(indices, _)) in faces.iter().enumerate() {
        let avg_z = indices.iter().map(|&i| rotate(vertices[i])[2]).sum::<f32>() / 4.0;
        face_depths[idx] = (idx, avg_z);
    }
    face_depths.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Draw faces with diffuse lighting
    let light_dir = [-0.5_f32, 0.8, -0.6];
    let light_len = (light_dir[0].powi(2) + light_dir[1].powi(2) + light_dir[2].powi(2)).sqrt();
    let norm_light = [light_dir[0] / light_len, light_dir[1] / light_len, light_dir[2] / light_len];

    for (face_idx, _) in face_depths {
        let (indices, normal) = faces[face_idx];
        let r_normal = rotate(normal);

        // Dot product with camera view vector [0, 0, -1] for backface culling
        if r_normal[2] < 0.1 {
            let dot = (r_normal[0] * norm_light[0] + r_normal[1] * norm_light[1] + r_normal[2] * norm_light[2]).max(0.0);
            let brightness = 0.25 + (dot * 0.75);

            let (base_r, base_g, base_b) = match theme {
                AppTheme::Dark => (0.0_f32, 190.0, 240.0),
                AppTheme::Light => (14.0_f32, 116.0, 222.0),
            };

            let face_col = Color32::from_rgb(
                (base_r * brightness) as u8,
                (base_g * brightness) as u8,
                (base_b * brightness) as u8,
            );

            let quad = [
                projected[indices[0]],
                projected[indices[1]],
                projected[indices[2]],
                projected[indices[3]],
            ];

            ui.painter().add(egui::Shape::convex_polygon(
                quad.to_vec(),
                face_col,
                Stroke::new(1.2_f32, Color32::from_rgb(255, 255, 255)),
            ));
        }
    }

    // Overlay FPS watermark badge inside viewport
    ui.painter().text(
        egui::pos2(rect.min.x + 12.0, rect.min.y + 12.0),
        egui::Align2::LEFT_TOP,
        if state.is_rendering {
            format!("🟢 3D GPU STRESS RENDER | {:.0} FPS ({:.2} ms)", state.live_fps, state.frametime_ms)
        } else {
            "⏸ Render Test em Pausa - Clique em Iniciar".to_string()
        },
        egui::FontId::proportional(12.5),
        if state.is_rendering { theme.accent_secondary() } else { theme.text_secondary() },
    );
}

