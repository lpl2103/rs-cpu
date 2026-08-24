# ⚡ Modern CPU-Z (Rust High-Performance Edition)

Uma ferramenta moderna, ultra-rápida e precisa de diagnóstico e telemetria de hardware desenvolvida 100% em **Rust**, com suporte nativo multiplataforma (Windows e Linux), renderização acelerada por GPU a 60 FPS e zero dependência de drivers inseguros em nível de kernel.

---

## ✨ Destaques & Funcionalidades

- 🖥️ **Painel Unificado ("All-in-One Dashboard")**: Exibe simultaneamente em uma única tela moderna as informações completas do **Processador (CPU)**, da **Placa-Mãe (Motherboard)** e da **Memória RAM & SPD**, além de medidores de telemetria em tempo real.
- 🔲 **Introspecção de CPU em Baixo Nível**:
  - Leitura direta de registradores x86 via instruções `CPUID` (`raw-cpuid`).
  - Identificação de família, modelo, stepping, revisão, codinome da arquitetura e soquete.
  - Tabela completa de instruções com badges dinâmicos (AVX, AVX2, AVX-512, SSE4.2, FMA3, AES, SHA, VT-x/AMD-V).
  - Hierarquia de cache multinível (L1 Data, L1 Instruction, L2, L3 com tamanhos e associatividades).
  - Telemetria de clocks em tempo real (Core Speed, Multiplicador, BCLK) e monitoramento de carga por núcleo.
- 🖧 **Placa-Mãe & BIOS (SMBIOS/DMI)**:
  - Leitura de tabelas SMBIOS Type 0, 1, 2 para detecção de fabricante, modelo, versão, chipset, barramentos e interface gráfica.
  - Dados completos de BIOS (Vendor, Versão, Data de Lançamento e suporte UEFI).
- 💾 **Memória RAM & Tabela SPD por Slot**:
  - Identificação de geração (DDR5, DDR4, etc.), tamanho total, modo de canais (Dual/Quad/Single) e frequências DRAM e Uncore.
  - Timings primários completos (CL, tRCD, tRP, tRAS, tRC, Command Rate 1T/2T).
  - Seletor de slots físicos com detalhes do fabricante do módulo, chips DRAM, Part Number e perfis JEDEC/XMP/EXPO.
- 🎮 **Placa de Vídeo (GPU)**:
  - Adaptadores gráficos detectados, VRAM dedicada, tipo de memória e largura do barramento.
- 📊 **Benchmark de CPU Integrado**:
  - Testes determinísticos Single-Thread e Multi-Thread com algoritmos matemáticos intensivos.
  - Stress Test em tempo real.
  - Comparação instantânea com processadores de referência (Core i9-14900K, Ryzen 9 7950X, Ryzen 7 7800X3D, Core i7-14700K, etc.).
- 🎨 **Temas Claro e Escuro**:
  - **Dark Mode**: Visual moderno em tons grafite/slate com realces ciano e esmeralda.
  - **Light Mode**: Interface limpa em titânio e safira.
  - Alternância instantânea com um clique.
- 📄 **Exportação de Diagnósticos**:
  - Exportação de relatórios completos em formato `.TXT` (padrão legível) e `.JSON` estruturado.

---

## 🚀 Como Compilar e Executar

### Pré-requisitos
- Rust 1.80+ (com Cargo)
- No Windows: MSVC C++ Build Tools (instalado via Visual Studio Build Tools)

### Execução em Modo de Desenvolvimento
```bash
cargo run
```

### Build Otimizado de Produção (Release)
O projeto está configurado com otimização máxima (`opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` e CRT estático para Windows):

```bash
cargo build --release
```

O binário final autossuficiente será gerado em `target/release/modern-cpu-z.exe` (~4.7 MB).

### Execução dos Testes Automatizados
```bash
cargo test
```

### Verificação Rigorosa com Clippy
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 🛠️ Arquitetura do Código

```text
src/
├── main.rs                   # Entrypoint com inicialização do tracing e viewport da janela
├── lib.rs                    # Exportação de módulos e diretivas rigorosas do compilador
├── app.rs                    # Loop de atualização eframe, gerenciamento de abas e estado
├── error.rs                  # Hierarquia tipada de erros com thiserror
├── bench/
│   └── mod.rs                # Motor de benchmark determinístico single/multi-thread com Rayon
├── hardware/
│   ├── mod.rs                # Agregador de telemetria e ciclo de refresh
│   ├── cpu.rs                # CPUID, topologia de cache e instruções
│   ├── motherboard.rs        # Parser SMBIOS/DMI para Placa-mãe e BIOS
│   ├── memory.rs             # Diagnóstico de RAM, slots e perfis SPD
│   └── gpu.rs                # Detecção de placas gráficas
└── ui/
    ├── mod.rs                # Roteamento dos componentes da interface
    ├── theme.rs              # Definição e alternância de temas Claro e Escuro
    ├── widgets.rs            # Widgets reutilizáveis (cards, gauges, badges, métricas)
    ├── tab_unified.rs        # Painel All-in-One (CPU, Placa-Mãe e Memória juntos)
    ├── tab_cpu.rs            # Aba detalhada de CPU
    ├── tab_mainboard.rs      # Aba detalhada de Placa-Mãe
    ├── tab_memory.rs         # Aba detalhada de Memória & SPD
    ├── tab_graphics.rs       # Aba detalhada de GPU
    ├── tab_bench.rs          # Aba de Benchmark e Stress Test
    └── tab_about.rs          # Aba de Informações e Exportação de Relatórios
```

---

## 📜 Licença
Distribuído sob licença MIT ou Apache-2.0.
