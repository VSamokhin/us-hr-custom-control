//! Supported USB product identities.

use serde::{Deserialize, Serialize};

/// TEAC Corporation's USB vendor identifier.
pub const TASCAM_VENDOR_ID: u16 = 0x0644;

/// A supported TASCAM US-HR model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceModel {
    /// TASCAM US-1x2HR.
    Us1x2Hr,
    /// TASCAM US-2x2HR.
    Us2x2Hr,
    /// TASCAM US-4x4HR.
    Us4x4Hr,
}

impl DeviceModel {
    /// Returns the model for a known USB product identifier.
    #[must_use]
    pub const fn from_product_id(product_id: u16) -> Option<Self> {
        match product_id {
            0x806f => Some(Self::Us1x2Hr),
            0x8070 => Some(Self::Us2x2Hr),
            0x8071 => Some(Self::Us4x4Hr),
            _ => None,
        }
    }

    /// Returns the USB product identifier.
    #[must_use]
    pub const fn product_id(self) -> u16 {
        match self {
            Self::Us1x2Hr => 0x806f,
            Self::Us2x2Hr => 0x8070,
            Self::Us4x4Hr => 0x8071,
        }
    }

    /// Returns the display name used by TASCAM.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Us1x2Hr => "US-1x2HR",
            Self::Us2x2Hr => "US-2x2HR",
            Self::Us4x4Hr => "US-4x4HR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_all_supported_product_ids() {
        for model in [
            DeviceModel::Us1x2Hr,
            DeviceModel::Us2x2Hr,
            DeviceModel::Us4x4Hr,
        ] {
            assert_eq!(
                DeviceModel::from_product_id(model.product_id()),
                Some(model)
            );
        }
    }

    #[test]
    fn rejects_unknown_product_id() {
        assert_eq!(DeviceModel::from_product_id(0xffff), None);
    }
}
