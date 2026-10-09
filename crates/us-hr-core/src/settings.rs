//! User-facing settings represented independently of the GUI.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum value accepted by mixer level controls.
pub const MAX_MIXER_LEVEL: u8 = 127;

/// Mono or stereo routing for a paired signal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelMode {
    /// Sum or duplicate the pair as mono.
    #[default]
    Mono,
    /// Preserve the left and right channels.
    Stereo,
}

/// Settings exposed by the US-1x2HR control panel.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceSettings {
    /// Direct-monitor interpretation of inputs 1 and 2.
    pub direct_monitor: ChannelMode,
    /// Whether input channel 1 is delivered to the host.
    pub input_1_enabled: bool,
    /// Whether input channel 2 is delivered to the host.
    pub input_2_enabled: bool,
    /// Monitor balance, from input (0) to computer (127).
    pub monitor_balance: u8,
    /// Whether loopback routing is enabled.
    pub loopback_enabled: bool,
    /// Mono/stereo treatment of the loopback input signal.
    pub loopback_input: ChannelMode,
    /// Mono/stereo treatment of computer playback in the loopback signal.
    pub loopback_output: ChannelMode,
    /// Broadcast output level, from 0 to 127.
    pub broadcast_volume: u8,
    /// Whether standalone automatic power saving is enabled.
    pub auto_power_save: bool,
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            direct_monitor: ChannelMode::Mono,
            input_1_enabled: true,
            input_2_enabled: true,
            monitor_balance: 64,
            loopback_enabled: false,
            loopback_input: ChannelMode::Stereo,
            loopback_output: ChannelMode::Stereo,
            broadcast_volume: 127,
            auto_power_save: true,
        }
    }
}

/// One user-requested setting change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingChange {
    /// Set direct-monitor input pairing.
    DirectMonitor(ChannelMode),
    /// Enable or mute input 1.
    Input1Enabled(bool),
    /// Enable or mute input 2.
    Input2Enabled(bool),
    /// Set monitor balance from input (0) to computer (127).
    MonitorBalance(u8),
    /// Enable or disable loopback.
    LoopbackEnabled(bool),
    /// Set loopback input pairing.
    LoopbackInput(ChannelMode),
    /// Set computer-output pairing in the loopback mix.
    LoopbackOutput(ChannelMode),
    /// Set broadcast level from 0 to 127.
    BroadcastVolume(u8),
    /// Enable or disable automatic power saving.
    AutoPowerSave(bool),
}

impl SettingChange {
    /// Returns whether the setting represented by this change matches a full snapshot.
    #[must_use]
    pub fn matches(self, settings: &DeviceSettings) -> bool {
        match self {
            Self::DirectMonitor(value) => settings.direct_monitor == value,
            Self::Input1Enabled(value) => settings.input_1_enabled == value,
            Self::Input2Enabled(value) => settings.input_2_enabled == value,
            Self::MonitorBalance(value) => settings.monitor_balance == value,
            Self::LoopbackEnabled(value) => settings.loopback_enabled == value,
            Self::LoopbackInput(value) => settings.loopback_input == value,
            Self::LoopbackOutput(value) => settings.loopback_output == value,
            Self::BroadcastVolume(value) => settings.broadcast_volume == value,
            Self::AutoPowerSave(value) => settings.auto_power_save == value,
        }
    }

    /// Validates values that have a device-defined numeric range.
    ///
    /// # Errors
    ///
    /// Returns an error when a mixer level is outside `0..=127`.
    pub fn validate(self) -> Result<(), SettingsError> {
        match self {
            Self::MonitorBalance(value) if value > MAX_MIXER_LEVEL => {
                Err(SettingsError::MonitorBalanceOutOfRange(value))
            }
            Self::BroadcastVolume(value) if value > MAX_MIXER_LEVEL => {
                Err(SettingsError::BroadcastVolumeOutOfRange(value))
            }
            _ => Ok(()),
        }
    }
}

impl DeviceSettings {
    /// Validates all device-defined numeric ranges before a snapshot is used.
    ///
    /// # Errors
    ///
    /// Returns an error when either mixer level is outside `0..=127`.
    pub fn validate(&self) -> Result<(), SettingsError> {
        SettingChange::MonitorBalance(self.monitor_balance).validate()?;
        SettingChange::BroadcastVolume(self.broadcast_volume).validate()
    }

    /// Returns the complete set of write commands for this snapshot.
    #[must_use]
    pub const fn changes(self) -> [SettingChange; 9] {
        [
            SettingChange::DirectMonitor(self.direct_monitor),
            SettingChange::Input1Enabled(self.input_1_enabled),
            SettingChange::Input2Enabled(self.input_2_enabled),
            SettingChange::MonitorBalance(self.monitor_balance),
            SettingChange::LoopbackEnabled(self.loopback_enabled),
            SettingChange::LoopbackInput(self.loopback_input),
            SettingChange::LoopbackOutput(self.loopback_output),
            SettingChange::BroadcastVolume(self.broadcast_volume),
            SettingChange::AutoPowerSave(self.auto_power_save),
        ]
    }
}

/// Invalid user-facing settings that cannot be represented safely on a device.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum SettingsError {
    /// Monitor balance exceeded the device's seven-bit range.
    #[error("monitor balance {0} is outside 0..={MAX_MIXER_LEVEL}")]
    MonitorBalanceOutOfRange(u8),
    /// Broadcast volume exceeded the device's seven-bit range.
    #[error("broadcast volume {0} is outside 0..={MAX_MIXER_LEVEL}")]
    BroadcastVolumeOutOfRange(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changes_match_their_source_snapshot() {
        let settings = DeviceSettings::default();
        assert!(
            settings
                .changes()
                .into_iter()
                .all(|change| change.matches(&settings))
        );
    }

    #[test]
    fn changed_value_does_not_match_snapshot() {
        let settings = DeviceSettings::default();
        assert!(!SettingChange::BroadcastVolume(12).matches(&settings));
    }

    #[test]
    fn accepts_boundary_mixer_levels() {
        let settings = DeviceSettings {
            monitor_balance: 0,
            broadcast_volume: MAX_MIXER_LEVEL,
            ..DeviceSettings::default()
        };

        assert_eq!(settings.validate(), Ok(()));
        assert_eq!(
            SettingChange::MonitorBalance(MAX_MIXER_LEVEL).validate(),
            Ok(())
        );
    }

    #[test]
    fn rejects_out_of_range_mixer_levels() {
        let invalid_monitor = DeviceSettings {
            monitor_balance: 128,
            ..DeviceSettings::default()
        };
        let invalid_broadcast = DeviceSettings {
            broadcast_volume: u8::MAX,
            ..DeviceSettings::default()
        };

        assert_eq!(
            invalid_monitor.validate(),
            Err(SettingsError::MonitorBalanceOutOfRange(128))
        );
        assert_eq!(
            invalid_broadcast.validate(),
            Err(SettingsError::BroadcastVolumeOutOfRange(u8::MAX))
        );
    }
}
