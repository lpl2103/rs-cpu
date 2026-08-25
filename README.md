# ⚡ M-CPU (Modern CPU-Z & Hardware Suite)

<div align="center">

![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-brightgreen?style=for-the-badge)
![GUI](https://img.shields.io/badge/GUI-egui%20%2F%20eframe%200.31-blueviolet?style=for-the-badge)
![Performance](https://img.shields.io/badge/Startup-<15ms-red?style=for-the-badge)

**Uma suíte completa, moderna e ultra-rápida de diagnóstico, telemetria em tempo real e benchmarks de hardware 100% desenvolvida em Rust.**

</div>

---

## 🌟 Visão Geral

O **M-CPU** combina a precisão do clássico CPU-Z, a riqueza gráfica do GPU-Z, a inteligência de armazenamento do CrystalDiskInfo/CrystalDiskMark e a estabilidade térmica do OCCT em uma interface gráfica moderna, acelerada por GPU, com suporte a temas Claro/Escuro, abertura instantânea (< 15ms) e zero dependência de drivers invasivos em nível de kernel.

---

## ✨ Funcionalidades Principais

### 1. 🔲 Processador (CPU)
- **Instrução Nativa CPUID (`raw-cpuid`)**: Leitura direta dos registradores x86 sem camadas intermediárias.
- **Topologia Completa**: Núcleos físicos, threads lógicas, família, modelo, stepping, revisão e litografia.
- **Hierarquia Multinível de Cache**: Detecção precisa de caches L1I, L1D, L2 e L3 com suas associatividades e tamanhos.
- **Badges de Instruções**: Identificação dinâmica de conjuntos modernos (`AVX-512`, `AVX2`, `FMA3`, `SSE4.2`, `AES-NI`, `SHA`, `VT-x/AMD-V`).
- **Telemetria por Núcleo**: Monitoramento de frequências individuais (MHz) e percentual de carga em tempo real.

### 2. 🖧 Placa-Mãe & BIOS (Motherboard)
- **Parser SMBIOS/DMI Type 0, 1, 2**: Fabricante, modelo, número de série, versão e data da BIOS.
- **Chipset & Barramentos**: Identificação de chipsets AMD (X670, B650, B550, etc.) e Intel (Z890, Z790, B760, etc.).

### 3. 💾 Memória RAM & Teste de Estabilidade
- **Configuração de Canais**: Single, Dual, Quad ou Octa-Channel, frequência DRAM e proporção de clock.
- **Timings Primários Detalhados**: `tCL`, `tRCD`, `tRP`, `tRAS`, `tRC` e `Command Rate (1T/2T)`.
- **Tabela SPD por Slot**: Fabricante dos módulos e chips DRAM (Samsung, Micron, SK Hynix), Part Number e perfis JEDEC/XMP/EXPO.
- **🔥 Teste de Estabilidade de RAM**: Alocador determinístico com padrões de estresse (*Bit Flip*, *Walking Inversion* e *Random Bit Pattern*) em chunks paralelos via Rayon.

### 4. 🎮 Placa Gráfica (GPU-Z) & Viewport 3D
- **Base de Hardware Expandida**: Suporte a NVIDIA RTX 50/40/30/20, GTX 16/10, AMD Radeon RX 7000/6000/5000 e Intel Arc/Battlemage.
- **Métricas Gráficas**: Shaders/CUDA Cores, Texture Fillrate, Pixel Fillrate, largura do barramento, tipo de memória e VRAM.
- **Badges Oficiais Dinâmicos**: Classificação automática de marcas e linhas (NVIDIA GeForce RTX/GTX, AMD Radeon, Intel Arc Graphics).
- **🌀 Viewport 3D Render Test (Zero-Allocation)**: Teste 3D interativo com iluminação difusa, culling de faces e rotação matemática executando a 60-144 FPS com **0 alocações de heap** por frame.

### 5. 💽 Armazenamento (SSD / NVMe / HDD)
- **Win32 IOCTL Direto**: Leitura física rápida via `STORAGE_DEVICE_DESCRIPTOR` e `IOCTL_DISK_GET_LENGTH_INFO`.
- **Mapeamento de Partições**: Mapeamento nativo de volumes (C:, D:, etc.) vinculados individualmente a cada disco físico.
- **Diagnósticos de Saúde & Alertas S.M.A.R.T.**: Monitoramento de temperatura, setores realocados, desgaste de células NAND e quedas de energia inseguras.
- **🚀 Benchmark de Disco Integrado (Estilo CrystalDiskMark)**:
  - Leitura/Escrita Sequencial (1 MiB, Q8T1).
  - Leitura/Escrita Randômica (4 KiB, Q32T1).
  - Histórico de benchmarks anteriores salvo para comparações.

### 6. ⚡ Energia & Fonte de Alimentação (PSU / OCCT)
- **Teste de Estabilidade da Fonte (PSU Stability Test)**: Carga sintética contínua em todos os núcleos lógicos.
- **Detecção de Vdroop**: Cálculo da variação percentual na linha de +12V e detecção automática de anomalias elétricas.
- **Gráficos de Telemetria com Linhas de Grade e Eixo Y**: Curvas em tempo real de temperatura, tensão +12V e consumo total de energia (W).

### 7. 📊 Benchmark de CPU & Stress Test
- **Workload Matemático**: Aritmética Mandelbrot determinística e transformações bitwise.
- **Pontuações Single-Thread & Multi-Thread**: Comparação instantânea com CPUs de referência (Core i9-14900K, Ryzen 9 7950X, Ryzen 7 7800X3D, Core i7-14700K, etc.).
- **Cancelamento Granular Instantâneo**: Interrupção imediata (< 5ms) ao acionar o botão de parada.

### 8. 🛡️ Exportação de Relatórios & Clipboard
- **📋 1-Click Clipboard**: Cópia instantânea do relatório completo para a área de transferência do sistema operacional.
- **📊 Planilhas CSV**: Exportação estruturada para Excel e Google Sheets.
- **🌐 Relatório HTML Moderno**: Documento web autocontido com visual escuro elegante e tabelas responsivas.
- **📄 Relatórios TXT e JSON**: Formatos padrão para auditorias e integrações.

---

## ⚡ Performance & Arquitetura

- **Inicialização Sub-15ms**: Substituição de comandos lentos em subprocessos por chamadas nativas de baixo nível na API Win32 e Sysfs.
- **Zero-Allocation Hot Paths**: Caches em memória, reuso de vetores e arrays na stack `[Pos2; 8]` e `[(usize, f32); 6]` para renderização fluida sem sobrecarregar o garbage collector.
- **Thread Safety**: Multithreading nativo com Rayon ThreadPools, controle de cancelamento atômico (`AtomicBool`) e mutexes protegidos contra envenenamento (`unwrap_or_else`).

---

## 🚀 Como Compilar e Executar

### Pré-requisitos
- [Rust](https://www.rust-lang.org/) (versão estável 1.80+)
- No Windows: MSVC C++ Build Tools (instalado via Visual Studio Build Tools)

### Execução em Modo de Desenvolvimento
```bash
cargo run
```

### Build Otimizado de Release (Produção)
```bash
cargo build --release
```
O executável final autossuficiente e otimizado com LTO e strip será gerado em:
```text
target/release/modern-cpu-z.exe
```

### Executar Testes Automatizados
```bash
cargo test --all-targets
```

### Verificar Linter Rigoroso
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 📁 Estrutura do Código

```text
src/
├── main.rs                   # Ponto de entrada, configuração de tracing e viewport
├── lib.rs                    # Exportação dos módulos principais e regras de compilação
├── app.rs                    # Gerenciador de estado, roteamento de abas e ciclo de vida eframe
├── error.rs                  # Hierarquia tipada de erros com thiserror
├── bench/
│   └── mod.rs                # Motor de benchmark determinístico e testes de estresse
├── hardware/
│   ├── mod.rs                # Agregador central e motor de atualização de hardware
│   ├── cpu.rs                # Introspecção CPUID, topologia e frequências
│   ├── motherboard.rs        # Parser SMBIOS/DMI para placa-mãe e BIOS
│   ├── memory.rs             # Diagnóstico de RAM, SPD e teste de estresse de memória
│   ├── gpu.rs                # Detecção de GPUs, especificações e hardware database
│   ├── storage.rs            # Win32 IOCTL, SMART, mapeamento de volumes e benchmark de disco
│   └── power.rs              # Teste de PSU, medição de Vdroop e curvas de telemetria
└── ui/
    ├── mod.rs                # Módulos de interface gráfica
    ├── theme.rs              # Temas Claro e Escuro com cores semânticas padronizadas
    ├── widgets.rs            # Componentes reutilizáveis (cards, gauges, badges, gráficos)
    ├── tab_unified.rs        # Painel All-in-One Dashboard
    ├── tab_cpu.rs            # Aba de Processador
    ├── tab_mainboard.rs      # Aba de Placa-Mãe
    ├── tab_memory.rs         # Aba de Memória RAM & SPD
    ├── tab_graphics.rs       # Aba de GPU & Render Test 3D
    ├── tab_storage.rs        # Aba de Armazenamento & Benchmark
    ├── tab_power.rs          # Aba de Energia & PSU
    ├── tab_bench.rs          # Aba de Benchmark de CPU
    └── tab_about.rs          # Aba Sobre, Exportação (TXT, JSON, CSV, HTML) e Clipboard
```

---

## 📜 Licença

Distribuído sob a licença **MIT**. Consulte o arquivo [LICENSE](LICENSE) para obter mais informações.

Desenvolvido por **[Leandro (lpl2103)](https://github.com/lpl2103)**.
