//! Device-independent types and protocol primitives for TASCAM US-HR devices.

mod message;
mod model;
mod settings;
mod status;

pub use message::{Field, FieldValue, MAX_MESSAGE_LEN, Message, MessageError};
pub use model::{DeviceModel, TASCAM_VENDOR_ID};
pub use settings::{ChannelMode, DeviceSettings, MAX_MIXER_LEVEL, SettingChange, SettingsError};
pub use status::{DeviceStatus, FirmwareInfo};

/// USB vendor request used to send a message to a US-HR device.
pub const REQUEST_SEND: u8 = 0x1d;

/// USB vendor request used to receive a message from a US-HR device.
pub const REQUEST_RECEIVE: u8 = 0x1e;

/// Host-to-device, vendor, device USB request type.
pub const REQUEST_TYPE_OUT: u8 = 0x40;

/// Device-to-host, vendor, device USB request type.
pub const REQUEST_TYPE_IN: u8 = 0xc0;
