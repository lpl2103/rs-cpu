//! Motherboard and BIOS introspection using SMBIOS tables and platform DMI.

use serde::{Deserialize, Serialize};
use smbioslib::{
    SMBiosBaseboardInformation, SMBiosInformation, SMBiosSystemInformation,
};

/// Motherboard and BIOS hardware information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardInfo {
    /// Motherboard manufacturer (e.g. "`ASUSTeK` COMPUTER INC.").
    pub manufacturer: String,
    /// Motherboard product model (e.g. "ROG STRIX B650E-F GAMING WIFI").
    pub model: String,
    /// Motherboard hardware version/revision.
    pub version: String,
    /// Motherboard serial number (masked or raw).
    pub serial_number: String,
    /// System manufacturer (OEM name, e.g. "Dell", "Lenovo", "Custom PC").
    pub system_manufacturer: String,
    /// System family / SKU.
    pub system_sku: String,
    /// Chipset / Southbridge estimate.
    pub chipset: String,
    /// Bus specifications.
    pub bus_specs: String,
    /// Graphic interface support (e.g. "PCI-Express 4.0 / 5.0 (16x)").
    pub graphic_interface: String,
    /// BIOS Vendor (e.g. "American Megatrends Inc.").
    pub bios_vendor: String,
    /// BIOS Version (e.g. "2613").
    pub bios_version: String,
    /// BIOS Release Date (e.g. "08/14/2024").
    pub bios_date: String,
    /// SMBIOS table version (e.g. "3.5").
    pub smbios_version: String,
}

impl Default for MotherboardInfo {
    fn default() -> Self {
        Self {
            manufacturer: "Unknown Manufacturer".to_string(),
            model: "Base Board Model".to_string(),
            version: "Rev 1.0".to_string(),
            serial_number: "Default string".to_string(),
            system_manufacturer: "System Manufacturer".to_string(),
            system_sku: "Default SKU".to_string(),
            chipset: "AMD / Intel Platform Controller Hub".to_string(),
            bus_specs: "PCI-Express 4.0 / 5.0".to_string(),
            graphic_interface: "PCI-Express 5.0 x16".to_string(),
            bios_vendor: "American Megatrends Inc.".to_string(),
            bios_version: "1.00".to_string(),
            bios_date: "01/01/2024".to_string(),
            smbios_version: "3.5".to_string(),
        }
    }
}

impl MotherboardInfo {
    /// Detects motherboard and BIOS specifications via SMBIOS tables.
    #[must_use]
    pub fn detect() -> Self {
        let mut info = Self::default();

        if let Ok(data) = smbioslib::table_load_from_device() {
            if let Some(version) = data.version {
                info.smbios_version = format!("{}.{}.{}", version.major, version.minor, version.revision);
            }

            // Parse Baseboard
            for baseboard in data.collect::<SMBiosBaseboardInformation<'_>>() {
                let s = baseboard.manufacturer().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.manufacturer = s;
                }
                let s = baseboard.product().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.model = s;
                }
                let s = baseboard.version().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.version = s;
                }
                let s = baseboard.serial_number().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.serial_number = s;
                }
            }

            // Parse System Information
            for sys in data.collect::<SMBiosSystemInformation<'_>>() {
                let s = sys.manufacturer().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.system_manufacturer = s;
                }
                let s = sys.sku_number().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.system_sku = s;
                }
                if info.model == "Base Board Model" {
                    let s = sys.product_name().to_string();
                    if !s.trim().is_empty() && !s.contains("Undefined") {
                        info.model = s;
                    }
                }
            }

            // Parse BIOS Information
            for bios in data.collect::<SMBiosInformation<'_>>() {
                let s = bios.vendor().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.bios_vendor = s;
                }
                let s = bios.version().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.bios_version = s;
                }
                let s = bios.release_date().to_string();
                if !s.trim().is_empty() && !s.contains("Undefined") {
                    info.bios_date = s;
                }
            }
        }

        // Refine chipset name based on model string
        info.chipset = guess_chipset(&info.model, &info.manufacturer);

        info
    }
}

/// Heuristically deduces chipset family from motherboard product model.
fn guess_chipset(model: &str, manufacturer: &str) -> String {
    let m = model.to_uppercase();
    if m.contains("X870E") { "AMD X870E Promontory 21".to_string() }
    else if m.contains("X870") { "AMD X870 Promontory 21".to_string() }
    else if m.contains("X670E") { "AMD X670E Dual Promontory 21".to_string() }
    else if m.contains("X670") { "AMD X670 Dual Promontory 21".to_string() }
    else if m.contains("B650E") { "AMD B650E Extreme".to_string() }
    else if m.contains("B650") { "AMD B650 Promontory 21".to_string() }
    else if m.contains("A620") { "AMD A620".to_string() }
    else if m.contains("X570") { "AMD X570".to_string() }
    else if m.contains("B550") { "AMD B550".to_string() }
    else if m.contains("B450") { "AMD B450 Promontory".to_string() }
    else if m.contains("A520") { "AMD A520".to_string() }
    else if m.contains("Z890") { "Intel Z890 Chipset (Arrow Lake)".to_string() }
    else if m.contains("Z790") { "Intel Z790 Express Chipset".to_string() }
    else if m.contains("B760") { "Intel B760 Express Chipset".to_string() }
    else if m.contains("H770") { "Intel H770 Express Chipset".to_string() }
    else if m.contains("Z690") { "Intel Z690 Express Chipset".to_string() }
    else if m.contains("B660") { "Intel B660 Express Chipset".to_string() }
    else if m.contains("Z590") { "Intel Z590 Express Chipset".to_string() }
    else if m.contains("B560") { "Intel B560 Express Chipset".to_string() }
    else if m.contains("Z490") { "Intel Z490 Express Chipset".to_string() }
    else if m.contains("B460") { "Intel B460 Express Chipset".to_string() }
    else {
        format!("{manufacturer} Host Bridge / PCH")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_chipset_amd() {
        let chipset = guess_chipset("ROG STRIX B650E-F GAMING WIFI", "ASUS");
        assert!(chipset.contains("B650E"));
    }

    #[test]
    fn test_guess_chipset_intel() {
        let chipset = guess_chipset("MPG Z790 CARBON WIFI", "MSI");
        assert!(chipset.contains("Z790"));
    }

    #[test]
    fn test_motherboard_default() {
        let mb = MotherboardInfo::default();
        assert_eq!(mb.manufacturer, "Unknown Manufacturer");
    }
}
