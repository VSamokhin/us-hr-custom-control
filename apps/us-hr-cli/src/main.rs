use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use tracing::info;
use tracing_subscriber::EnvFilter;
use us_hr_core::{ChannelMode, SettingChange};
use us_hr_usb::{AccessMode, UsbTransport, discover};

#[derive(Debug, Parser)]
#[command(
    name = "us-hr",
    version,
    about = "Control and inspect TASCAM US-HR interfaces"
)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List supported connected devices without sending control messages.
    List,
    /// Open and print the first supported device's USB identity.
    Inspect,
    /// Read firmware, sample rate, routing, mixer, and power settings.
    Status,
    /// Change one ordinary control-panel setting.
    Set {
        /// Setting to change.
        #[arg(value_enum)]
        setting: SettingName,
        /// New value: on/off, mono/stereo, or an integer from 0 to 127.
        value: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SettingName {
    DirectMonitor,
    Input1,
    Input2,
    MonitorBalance,
    Loopback,
    LoopbackInput,
    LoopbackOutput,
    BroadcastVolume,
    AutoPowerSave,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    match Arguments::parse().command {
        Command::List => list_devices(),
        Command::Inspect => inspect_device(),
        Command::Status => read_status(),
        Command::Set { setting, value } => set_value(setting, &value),
    }
}

fn list_devices() -> Result<()> {
    let devices = discover().context("USB discovery failed")?;
    if devices.is_empty() {
        println!("No supported TASCAM US-HR devices found.");
        return Ok(());
    }

    for device in devices {
        println!(
            "{} {:04x}:{:04x} bus={} address={} USB={}.{}.{} product={:?} manufacturer={:?} serial={:?}",
            device.model.display_name(),
            us_hr_core::TASCAM_VENDOR_ID,
            device.model.product_id(),
            device.bus_number,
            device.address,
            device.usb_device_version.0,
            device.usb_device_version.1,
            device.usb_device_version.2,
            device.product_string,
            device.manufacturer_string,
            device.serial_number,
        );
    }
    Ok(())
}

fn inspect_device() -> Result<()> {
    let transport = UsbTransport::open_first_read_only().context("failed to open device")?;
    info!(
        model = transport.info().model.display_name(),
        "opened device read-only"
    );
    println!("{:#?}", transport.info());
    Ok(())
}

fn read_status() -> Result<()> {
    let mut transport = UsbTransport::open_first_read_only().context("failed to open device")?;
    println!("{}", transport.info().model.display_name());
    let status = transport.read_status().context("status query failed")?;
    println!("{status:#?}");
    Ok(())
}

fn set_value(setting: SettingName, value: &str) -> Result<()> {
    let change = match setting {
        SettingName::DirectMonitor => SettingChange::DirectMonitor(parse_mode(value)?),
        SettingName::Input1 => SettingChange::Input1Enabled(parse_toggle(value)?),
        SettingName::Input2 => SettingChange::Input2Enabled(parse_toggle(value)?),
        SettingName::MonitorBalance => SettingChange::MonitorBalance(parse_level(value)?),
        SettingName::Loopback => SettingChange::LoopbackEnabled(parse_toggle(value)?),
        SettingName::LoopbackInput => SettingChange::LoopbackInput(parse_mode(value)?),
        SettingName::LoopbackOutput => SettingChange::LoopbackOutput(parse_mode(value)?),
        SettingName::BroadcastVolume => SettingChange::BroadcastVolume(parse_level(value)?),
        SettingName::AutoPowerSave => SettingChange::AutoPowerSave(parse_toggle(value)?),
    };
    let mut transport =
        UsbTransport::open_first(AccessMode::ReadWrite).context("failed to open device")?;
    let status = transport
        .apply_setting_verified(change)
        .context("failed to apply and verify setting")?;
    println!(
        "Applied and verified {setting:?} = {value}; device state: {:?}",
        status.settings
    );
    Ok(())
}

fn parse_toggle(value: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "on" | "true" | "enabled" => Ok(true),
        "off" | "false" | "disabled" => Ok(false),
        _ => bail!("expected on or off, got {value:?}"),
    }
}

fn parse_mode(value: &str) -> Result<ChannelMode> {
    match value.to_ascii_lowercase().as_str() {
        "mono" => Ok(ChannelMode::Mono),
        "stereo" => Ok(ChannelMode::Stereo),
        _ => bail!("expected mono or stereo, got {value:?}"),
    }
}

fn parse_level(value: &str) -> Result<u8> {
    let level = value
        .parse::<u8>()
        .with_context(|| format!("expected an integer from 0 to 127, got {value:?}"))?;
    if level > 127 {
        bail!("level must be from 0 to 127, got {level}");
    }
    Ok(level)
}

#[cfg(test)]
mod tests {
    use super::{
        Arguments, ChannelMode, Command, Parser, SettingName, parse_level, parse_mode,
        parse_toggle, set_value,
    };

    #[test]
    fn parses_human_setting_values() -> anyhow::Result<()> {
        assert!(parse_toggle("on")?);
        assert!(!parse_toggle("DISABLED")?);
        assert_eq!(parse_mode("stereo")?, ChannelMode::Stereo);
        assert_eq!(parse_level("127")?, 127);
        assert!(parse_level("128").is_err());
        Ok(())
    }

    #[test]
    fn accepts_documented_toggle_aliases_case_insensitively() -> anyhow::Result<()> {
        for value in ["on", "TRUE", "Enabled"] {
            assert!(parse_toggle(value)?);
        }
        for value in ["off", "FALSE", "Disabled"] {
            assert!(!parse_toggle(value)?);
        }
        Ok(())
    }

    #[test]
    fn rejects_invalid_values_before_opening_hardware() {
        for value in ["", "yes", "1"] {
            assert!(parse_toggle(value).is_err());
        }
        for value in ["", "dual", "1"] {
            assert!(parse_mode(value).is_err());
        }
        for value in ["-1", "128", "256", "12.5", " 12"] {
            assert!(parse_level(value).is_err());
        }

        assert!(set_value(SettingName::BroadcastVolume, "128").is_err());
    }

    #[test]
    fn parses_kebab_case_setting_names() -> anyhow::Result<()> {
        let arguments = Arguments::try_parse_from(["us-hr", "set", "auto-power-save", "enabled"])?;

        assert!(matches!(
            arguments.command,
            Command::Set {
                setting: SettingName::AutoPowerSave,
                value
            } if value == "enabled"
        ));
        Ok(())
    }
}
