//! UI modules, tabs, and presentation layer.

pub mod icon;
pub mod tab_about;
pub mod tab_bench;
pub mod tab_cpu;
pub mod tab_graphics;
pub mod tab_mainboard;
pub mod tab_memory;
pub mod tab_power;
pub mod tab_storage;
pub mod tab_unified;
pub mod theme;
pub mod widgets;

pub use icon::create_app_icon;
pub use tab_about::AboutTabState;
pub use tab_graphics::GraphicsTabState;
pub use tab_memory::MemoryTabState;
pub use tab_storage::StorageTabState;
pub use theme::AppTheme;
