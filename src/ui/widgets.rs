//! Reusable modern UI widgets, brand badges, and presentation helpers.

use super::theme::AppTheme;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Ui};

/// Returns the description for an instruction set flag.
fn get_instruction_description(name: &str) -> &'static str {
    match name {
        "MMX" => "MultiMedia eXtensions: Instruções SIMD para processamento de áudio/gráficos em números inteiros.",
        "SSE" => "Streaming SIMD Extensions: Operações vetoriais de ponto flutuante de 128 bits.",
        "SSE2" => "Streaming SIMD Extensions 2: Ponto flutuante de dupla precisão e inteiros em 128 bits.",
        "SSE3" => "Streaming SIMD Extensions 3: Instruções de DSP e operações horizontais SIMD.",
        "SSSE3" => "Supplemental SSE3: Manipulação avançada de bytes e permutações alinhadas.",
        "SSE4.1" => "SSE 4.1: Operações de multiplicação de vetores e ponto flutuante de alta performance.",
        "SSE4.2" => "SSE 4.2: Instruções aceleradas para processamento de strings, texto e cálculo CRC32.",
        "AVX" => "Advanced Vector Extensions: Registradores YMM de 256 bits com formato de 3 operandos.",
        "AVX2" => "Advanced Vector Extensions 2: Expansão de operações inteiras para 256 bits e suporte a FMA.",
        "AVX-512F" => "AVX-512 Foundation: Vetores ultralargos de 512 bits com registradores ZMM.",
        "AVX-512BW" => "AVX-512 Byte & Word: Operações de 512 bits para inteiros de 8 e 16 bits.",
        "AVX-512CD" => "AVX-512 Conflict Detection: Detecção de conflitos de endereçamento em laços.",
        "AVX-512DQ" => "AVX-512 Doubleword & Quadword: Operações aritméticas e conversões em 32 e 64 bits.",
        "AVX-512VL" => "AVX-512 Vector Length: Extensões AVX-512 aplicadas em vetores menores de 128 e 256 bits.",
        "AVX-512ER" => "AVX-512 Exponential & Reciprocal: Funções matemáticas de aproximação rápida.",
        "AVX-512PF" => "AVX-512 Prefetch: Instruções de pré-carregamento de cache para computação científica.",
        "FMA3" => "Fused Multiply-Add: Cálculo a*b + c com precisão máxima em um único ciclo de clock.",
        "AES" => "Advanced Encryption Standard: Aceleração direta por hardware para criptografia AES.",
        "SHA" => "Secure Hash Algorithm: Aceleração em hardware para funções criptográficas SHA-1 e SHA-256.",
        "VT-x" => "Intel Virtualization Technology: Suporte a virtualização assistida por hardware.",
        "AMD-V" => "AMD Virtualization: Extensões SVM de alta eficiência para máquinas virtuais.",
        "x86-64" => "Arquitetura x86-64 (AMD64/EM64T): Registradores RAX..R15 e endereçamento de 64 bits.",
        "ABM" => "Advanced Bit Manipulation: Instruções POPCNT e LZCNT para contagem de bits.",
        "BMI1" => "Bit Manipulation Set 1: Extração e manipulação rápida de campos de bits sem branches.",
        "BMI2" => "Bit Manipulation Set 2: Instruções PDEP, PEXT e BZHI de alto throughput.",
        "POPCNT" => "Population Count: Contagem instantânea do número de bits '1' em uma palavra.",
        "CLFLUSHOPT" => "Cache Line Flush Optimized: Limpeza otimizada de linhas de cache com alto throughput.",
        "SGX" => "Software Guard Extensions: Enclaves de segurança e memória protegida em hardware.",
        "SSE4A" => "SSE4a (AMD): Instruções específicas da AMD para inserção/extração de bits.",
        "3DNow!" => "3DNow! (AMD): Conjunto SIMD pioneiro para cálculos de ponto flutuante 3D.",
        _ => "Conjunto de instruções e extensões SIMD aceleradas por hardware.",
    }
}

/// Returns the description for a GPU technology flag.
fn get_gpu_tech_description(name: &str) -> &'static str {
    match name {
        "DirectX 12 Ultimate" => "Microsoft DirectX 12 Ultimate: Suporte a DXR 1.1, Mesh Shaders, Sampler Feedback e VRS.",
        "Vulkan 1.3" => "Khronos Vulkan 1.3: API gráfica de baixo nível com renderização dinâmica e sincronização avançada.",
        "Ray Tracing (RTX/DXR)" => "Ray Tracing por Hardware: Aceleração em tempo real para iluminação global, reflexos e sombras físicas.",
        "DLSS / FSR Upscaling" => "Super-resolução e Reconstrução por IA (Deep Learning Super Sampling / FidelityFX Super Resolution).",
        "CUDA 12.6" | "CUDA / Compute" => "NVIDIA CUDA / Compute: Arquitetura de computação massivamente paralela para IA e renderização.",
        "OpenCL 3.0" => "OpenCL 3.0: Padrão aberto para computação heterogênea em GPUs e aceleradores.",
        "DirectCompute" => "Microsoft DirectCompute: Shader de computação para tarefas GPGPU no DirectX.",
        "AV1 Dual Encode" | "AV1 Decode/Encode" => "Codec AV1: Codificação e decodificação em hardware com altíssima eficiência de compressão.",
        "Resizable BAR" => "Resizable BAR / Smart Access Memory: Acesso direto da CPU a toda a memória VRAM da GPU via PCIe.",
        _ => "Recurso gráfico avançado acelerado por hardware.",
    }
}

/// Renders a section header with icon, title, and optional brand badge.
pub fn section_header(ui: &mut Ui, theme: AppTheme, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(icon)
                .size(21.0)
                .color(theme.accent_primary())
                .strong(),
        );
        ui.label(
            RichText::new(title)
                .size(17.5)
                .color(theme.text_primary())
                .strong(),
        );
    });
    ui.add_space(8.0);
}

/// Renders a stylized brand logo badge (e.g. Intel Core, AMD Ryzen, NVIDIA `GeForce`, etc.).
pub fn brand_logo_badge(ui: &mut Ui, vendor: &str, name: &str) {
    let lower = format!("{vendor} {name}").to_lowercase();
    let (bg_color, border_color, brand_text, sub_text, text_color) = if lower.contains("amd") || lower.contains("ryzen") {
        (
            Color32::from_rgb(45, 12, 10),
            Color32::from_rgb(235, 50, 35),
            "AMD",
            if lower.contains("threadripper") { "THREADRIPPER" } else { "RYZEN" },
            Color32::from_rgb(255, 235, 230),
        )
    } else if lower.contains("intel") || lower.contains("core") {
        (
            Color32::from_rgb(8, 28, 55),
            Color32::from_rgb(0, 115, 230),
            "intel",
            if lower.contains("xeon") { "XEON" } else if lower.contains("ultra") { "CORE ULTRA" } else { "CORE" },
            Color32::from_rgb(225, 242, 255),
        )
    } else if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("rtx") {
        (
            Color32::from_rgb(18, 38, 12),
            Color32::from_rgb(118, 185, 0),
            "NVIDIA",
            "GEFORCE RTX",
            Color32::from_rgb(230, 255, 220),
        )
    } else if lower.contains("radeon") {
        (
            Color32::from_rgb(45, 12, 10),
            Color32::from_rgb(235, 50, 35),
            "AMD",
            "RADEON",
            Color32::from_rgb(255, 235, 230),
        )
    } else {
        (
            Color32::from_rgb(25, 30, 42),
            Color32::from_rgb(70, 85, 110),
            "M-CPU",
            "HARDWARE",
            Color32::from_rgb(240, 245, 255),
        )
    };

    egui::Frame::new()
        .fill(bg_color)
        .stroke(Stroke::new(1.5_f32, border_color))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(brand_text)
                        .size(15.0)
                        .color(text_color)
                        .strong(),
                );
                ui.label(
                    RichText::new(sub_text)
                        .size(12.0)
                        .color(border_color)
                        .strong(),
                );
            });
        });
}

/// Renders a key-value specification row with crisp formatting and larger text.
pub fn spec_row(ui: &mut Ui, theme: AppTheme, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_min_height(23.0);
        ui.label(
            RichText::new(label)
                .size(14.0)
                .color(theme.text_secondary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(value)
                    .size(14.0)
                    .color(theme.text_primary())
                    .strong(),
            );
        });
    });
}

/// Renders a compact instruction set or feature badge chip with rich tooltip on hover.
pub fn feature_badge(ui: &mut Ui, theme: AppTheme, text: &str, active: bool) {
    let tooltip = get_instruction_description(text);
    let (bg, border, text_col) = if active {
        (
            match theme {
                AppTheme::Dark => Color32::from_rgb(14, 44, 62),
                AppTheme::Light => Color32::from_rgb(218, 238, 255),
            },
            theme.accent_primary(),
            theme.accent_primary(),
        )
    } else {
        (
            match theme {
                AppTheme::Dark => Color32::from_rgb(26, 30, 38),
                AppTheme::Light => Color32::from_rgb(238, 241, 246),
            },
            match theme {
                AppTheme::Dark => Color32::from_rgb(45, 50, 60),
                AppTheme::Light => Color32::from_rgb(210, 215, 222),
            },
            theme.text_secondary(),
        )
    };

    let response = egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, border))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(12.5)
                    .color(text_col)
                    .strong(),
            );
        });

    response.response.on_hover_ui(|ui| {
        ui.label(RichText::new(text).strong().size(13.5).color(theme.accent_primary()));
        ui.add_space(2.0);
        ui.label(RichText::new(tooltip).size(12.5).color(theme.text_primary()));
    });
}

/// Renders a GPU technology badge with tooltip on hover.
pub fn gpu_tech_badge(ui: &mut Ui, theme: AppTheme, text: &str) {
    let tooltip = get_gpu_tech_description(text);
    let (bg, border, text_col) = (
        match theme {
            AppTheme::Dark => Color32::from_rgb(16, 42, 28),
            AppTheme::Light => Color32::from_rgb(225, 248, 235),
        },
        theme.accent_secondary(),
        theme.accent_secondary(),
    );

    let response = egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, border))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(12.5)
                    .color(text_col)
                    .strong(),
            );
        });

    response.response.on_hover_ui(|ui| {
        ui.label(RichText::new(text).strong().size(13.5).color(theme.accent_secondary()));
        ui.add_space(2.0);
        ui.label(RichText::new(tooltip).size(12.5).color(theme.text_primary()));
    });
}

/// Renders instructions in a multi-column grid rather than wrapping inconsistently.
pub fn instructions_grid(ui: &mut Ui, theme: AppTheme, instructions: &[String]) {
    const COLS: usize = 6;
    let rows = instructions.len().div_ceil(COLS);

    egui::Grid::new("instructions_structured_grid")
        .spacing(egui::vec2(6.0, 6.0))
        .show(ui, |ui| {
            for r in 0..rows {
                for c in 0..COLS {
                    let idx = r * COLS + c;
                    if let Some(inst) = instructions.get(idx) {
                        feature_badge(ui, theme, inst, true);
                    } else {
                        ui.label("");
                    }
                }
                ui.end_row();
            }
        });
}

/// Renders a modern progress/load gauge.
pub fn load_gauge(ui: &mut Ui, theme: AppTheme, label: &str, pct: f32, subtext: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(13.5)
                .color(theme.text_secondary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(subtext)
                    .size(13.5)
                    .color(theme.text_primary())
                    .strong(),
            );
        });
    });

    let fraction = (pct / 100.0).clamp(0.0, 1.0);
    let bar_color = if fraction > 0.85 {
        Color32::from_rgb(239, 68, 68)
    } else if fraction > 0.60 {
        Color32::from_rgb(245, 158, 11)
    } else {
        theme.accent_secondary()
    };

    let desired_width = ui.available_width();
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(desired_width, 10.0), egui::Sense::hover());
    
    // Background track
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(5),
        match theme {
            AppTheme::Dark => Color32::from_rgb(32, 38, 50),
            AppTheme::Light => Color32::from_rgb(220, 226, 236),
        },
    );

    // Filled progress
    if fraction > 0.001 {
        let mut filled_rect = rect;
        filled_rect.set_width(rect.width() * fraction);
        ui.painter().rect_filled(filled_rect, CornerRadius::same(5), bar_color);
    }
}

/// Renders a high-impact metric stat box.
pub fn stat_metric_box(ui: &mut Ui, theme: AppTheme, title: &str, value: &str, sub: &str) {
    egui::Frame::new()
        .fill(match theme {
            AppTheme::Dark => Color32::from_rgb(28, 33, 46),
            AppTheme::Light => Color32::from_rgb(242, 246, 252),
        })
        .stroke(Stroke::new(1.0_f32, theme.card_border()))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(12.0).color(theme.text_secondary()));
                ui.add_space(2.0);
                ui.label(RichText::new(value).size(17.0).color(theme.text_primary()).strong());
                ui.add_space(1.0);
                ui.label(RichText::new(sub).size(11.5).color(theme.accent_primary()));
            });
        });
}
