//! Typed information read from a connected interface.

use serde::{Deserialize, Serialize};

use crate::DeviceSettings;

/// Firmware identity returned by the device protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirmwareInfo {
    /// Firmware version encoded as the vendor's integer (100 means 1.00).
    pub version: u16,
    /// Firmware build number.
    pub build: u16,
    /// Product name embedded in the firmware.
    pub product_name: String,
    /// Product signature embedded in the firmware.
    pub product_signature: String,
    /// Firmware build date.
    pub build_date: String,
    /// Firmware build time.
    pub build_time: String,
}

/// Complete state exposed by the first implementation milestone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceStatus {
    /// Firmware and product identity.
    pub firmware: FirmwareInfo,
    /// Active hardware sample rate.
    pub sample_rate_hz: u32,
    /// User-facing routing and power settings.
    pub settings: DeviceSettings,
}
