use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rk855_config::ConfigManager;
use rk855_hid::{probe_devices, DeviceInfo, HidApiTransport, MockTransport};
use rk855_isp::analyze_firmware_bin;
use rk855_proto::{ColorType, Direction, LightMode, LightParams};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rk855",
    author = "Syafiq",
    version,
    about = "Royal Kludge RK855 (RK68) CLI configurator & firmware tool"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Enumerate USB HID devices and probe for RK855 keyboard or bootloader interfaces
    Probe {
        /// Show all connected HID devices, not just RK855 candidates
        #[arg(short, long)]
        all: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Control keyboard LED lighting effects, colors, brightness, and animations
    Light {
        #[command(subcommand)]
        command: LightCommands,
    },

    /// Firmware analysis and pre-flight validation tools
    Firmware {
        #[command(subcommand)]
        command: FirmwareCommands,
    },
}

#[derive(Subcommand, Debug)]
enum LightCommands {
    /// Configure keyboard backlight lighting effect, color, brightness, speed, and direction
    Set {
        /// Lighting effect mode (e.g. 'breath', 'static', 'wave', 'rainbow', or 0..19)
        #[arg(short, long)]
        mode: String,

        /// Color hex (#RRGGBB, RRGGBB) or preset ('rainbow', 'spectrum', 'red', 'cyan', etc.)
        #[arg(short, long, default_value = "ff00ff")]
        color: String,

        /// Brightness level (0..4 user steps or 0..15 raw value)
        #[arg(short, long, default_value = "3")]
        brightness: u8,

        /// Speed level (0..4 user steps or 0..10 raw value)
        #[arg(short, long, default_value = "2")]
        speed: u8,

        /// Animation direction ('ltr' or 'rtl')
        #[arg(short, long, default_value = "ltr")]
        direction: String,

        /// Live test mode: updates lighting immediately in RAM without committing to flash (prevents flash wear)
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to onboard flash memory across power cycles (default)
        #[arg(long)]
        persist: bool,

        /// Use direct 4-packet sequence (04 18, 04 11, [04 02], 04 F0) instead of 22-packet transaction
        #[arg(long)]
        direct: bool,

        /// Specific HID device path to open (defaults to auto-detected Interface 0)
        #[arg(long)]
        path: Option<String>,

        /// Run against an in-memory mock transport (for testing without touching hardware)
        #[arg(long)]
        mock: bool,

        /// Output applied parameters in JSON format
        #[arg(long)]
        json: bool,
    },

    /// List all 20 built-in lighting effect modes with supported parameters
    ListModes {
        /// Output modes in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum FirmwareCommands {
    /// Inspect and validate a firmware image (.bin) offline without hardware
    Check {
        /// Path to firmware binary (.bin)
        path: PathBuf,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Probe { all, json } => run_probe(all, json)?,
        Commands::Light { command } => match command {
            LightCommands::Set {
                mode,
                color,
                brightness,
                speed,
                direction,
                live,
                persist,
                direct,
                path,
                mock,
                json,
            } => run_light_set(
                mode, color, brightness, speed, direction, live, persist, direct, path, mock, json,
            )?,
            LightCommands::ListModes { json } => run_light_list_modes(json)?,
        },
        Commands::Firmware { command } => match command {
            FirmwareCommands::Check { path, json } => run_firmware_check(path, json)?,
        },
    }

    Ok(())
}

fn run_probe(show_all: bool, output_json: bool) -> Result<()> {
    let devices = probe_devices().context("Failed to enumerate HID devices via hidapi")?;

    let candidates: Vec<&DeviceInfo> = devices
        .iter()
        .filter(|d| d.is_candidate_rk855() || d.is_bootloader())
        .collect();

    if output_json {
        let display_list: Vec<&DeviceInfo> = if show_all {
            devices.iter().collect()
        } else {
            candidates.clone()
        };
        let s = serde_json::to_string_pretty(&display_list)?;
        println!("{s}");
        return Ok(());
    }

    println!("============================================================");
    println!(" Royal Kludge RK855 USB HID Probe");
    println!("============================================================");
    println!("Total HID interfaces enumerated: {}", devices.len());
    println!("Candidate RK855 interfaces detected: {}", candidates.len());
    println!();

    if !candidates.is_empty() {
        println!("--- Detected RK855 / Bootloader Candidates ---");
        for (i, d) in candidates.iter().enumerate() {
            print_device_info(i + 1, d);
        }
    } else {
        println!("No active RK855 keyboard or bootloader interfaces detected.");
        if !show_all {
            println!("Tip: Run 'rk855 probe --all' to inspect all connected HID devices.");
        }
    }

    if show_all {
        println!();
        println!("--- All Enumerated HID Devices ---");
        for (i, d) in devices.iter().enumerate() {
            print_device_info(i + 1, d);
        }
    }

    Ok(())
}

fn print_device_info(idx: usize, d: &DeviceInfo) {
    let tag = if d.is_bootloader() {
        " [BOOTLOADER MODE (0C45:82AA)]"
    } else if d.is_config_interface() {
        " [PRIMARY CONFIG CHANNEL (Interface 0)]"
    } else if d.is_candidate_rk855() {
        " [CANDIDATE RK855 INTERFACE]"
    } else {
        ""
    };

    println!(
        "[{:02}] {:04X}:{:04X}{} | Iface: {} | UsagePage: 0x{:04X} | Usage: 0x{:04X}",
        idx, d.vendor_id, d.product_id, tag, d.interface_number, d.usage_page, d.usage
    );
    if let Some(ref prod) = d.product_string {
        println!("     Product:      {}", prod);
    }
    if let Some(ref mfg) = d.manufacturer_string {
        println!("     Manufacturer: {}", mfg);
    }
    println!("     Path:         {}", d.path);
    println!();
}

fn run_firmware_check(path: PathBuf, output_json: bool) -> Result<()> {
    let analysis = analyze_firmware_bin(&path)
        .with_context(|| format!("Failed to analyze firmware file: {}", path.display()))?;

    if output_json {
        let json_val = serde_json::json!({
            "path": path.display().to_string(),
            "file_size": analysis.file_size,
            "sha256": analysis.sha256_hex,
            "initial_sp": format!("0x{:08X}", analysis.initial_sp),
            "reset_vector": format!("0x{:08X}", analysis.reset_vector),
            "has_rom_marker": analysis.has_rom_marker,
            "rom_marker_value": format!("0x{:08X}", analysis.rom_marker_value),
            "contains_apple_vid_pid": analysis.contains_apple_vid_pid,
            "contains_isp_trigger_magic": analysis.contains_isp_trigger_magic,
            "populated_bytes": analysis.populated_bytes,
        });
        println!("{}", serde_json::to_string_pretty(&json_val)?);
        return Ok(());
    }

    println!("============================================================");
    println!(" RK855 Offline Firmware Analysis");
    println!(" File: {}", path.display());
    println!("============================================================");
    println!(
        "File Size:          {} bytes (0x{:X})",
        analysis.file_size, analysis.file_size
    );
    println!(
        "Populated Bytes:    {} bytes (non-0xFF)",
        analysis.populated_bytes
    );
    println!("SHA-256 Checksum:   {}", analysis.sha256_hex);
    println!(
        "Initial SP:         0x{:08X} (Top of SRAM)",
        analysis.initial_sp
    );
    println!(
        "Reset Vector:       0x{:08X} (Thumb vector)",
        analysis.reset_vector
    );
    println!(
        "ROM Marker (0xFFFC): 0x{:08X} [{}]",
        analysis.rom_marker_value,
        if analysis.has_rom_marker {
            "VALID (0xAAAA5555)"
        } else {
            "INVALID / MISSING"
        }
    );
    println!(
        "Apple ROM VID:PID:  {}",
        if analysis.contains_apple_vid_pid {
            "PRESENT (05AC:024F found at literal/ROM descriptor table)"
        } else {
            "NOT FOUND"
        }
    );
    println!(
        "ISP Trigger Magic:  {}",
        if analysis.contains_isp_trigger_magic {
            "PRESENT (AA 42 89 5A FF 71 62 CC found in code path)"
        } else {
            "NOT FOUND"
        }
    );

    println!();
    println!("Pre-Flight Verification Summary:");
    if analysis.initial_sp == 0x200015B0
        && analysis.reset_vector == 0x000066D1
        && analysis.has_rom_marker
    {
        println!("  [OK] Image matches verified RK855 SN32F240B target firmware specifications.");
    } else {
        println!(
            "  [WARNING] Vector table or ROM validity marker deviates from verified target image."
        );
    }

    Ok(())
}

fn run_light_list_modes(output_json: bool) -> Result<()> {
    if output_json {
        let modes: Vec<serde_json::Value> = LightMode::ALL
            .iter()
            .map(|m| {
                serde_json::json!({
                    "index": *m as u8,
                    "hex": format!("0x{:02X}", *m as u8),
                    "name": m.name(),
                    "slug": m.slug(),
                    "description": m.description(),
                    "supports_color": m.supports_color(),
                    "supports_speed": m.supports_speed(),
                    "supports_direction": m.supports_direction(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&modes)?);
        return Ok(());
    }

    println!("===============================================================================");
    println!(" Royal Kludge RK855 (RK68) Built-in Lighting Modes");
    println!("===============================================================================");
    println!(
        "{:<3} {:<6} {:<20} {:<22} {:<6} {:<6} {:<6}",
        "Idx", "Hex", "Slug", "Name", "Color", "Speed", "Dir"
    );
    println!(
        "{:-<3} {:-<6} {:-<20} {:-<22} {:-<6} {:-<6} {:-<6}",
        "", "", "", "", "", "", ""
    );

    for m in LightMode::ALL {
        println!(
            "{:<3} 0x{:02X}  {:<20} {:<22} {:<6} {:<6} {:<6}",
            m as u8,
            m as u8,
            m.slug(),
            m.name(),
            if m.supports_color() { "yes" } else { "no" },
            if m.supports_speed() { "yes" } else { "no" },
            if m.supports_direction() { "yes" } else { "no" },
        );
    }
    println!();
    println!("Usage:");
    println!("  rk855 light set --mode <slug> [options]");
    println!("Example:");
    println!(
        "  rk855 light set --mode breath --color ff00ff --brightness 3 --speed 2 --direction ltr"
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_light_set(
    mode_str: String,
    color_str: String,
    brightness_arg: u8,
    speed_arg: u8,
    direction_str: String,
    live: bool,
    persist: bool,
    direct: bool,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let mode = LightMode::from_name_or_slug(&mode_str).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown lighting mode '{}'. Run 'rk855 light list-modes' to see supported modes.",
            mode_str
        )
    })?;

    let (color_type, r, g, b) =
        parse_color(&color_str).with_context(|| format!("Invalid color format '{}'", color_str))?;

    let brightness = parse_brightness(brightness_arg);
    let speed = parse_speed(speed_arg);
    let direction = Direction::from_str_name(&direction_str).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid direction '{}'. Expected 'ltr' or 'rtl'.",
            direction_str
        )
    })?;

    // Lowest-wear default: If --live is explicitly set, do not commit.
    // If --persist is set, commit to flash.
    // Default: persist = true so setting is stored unless live test requested.
    let should_persist = if live { false } else { persist || !live };

    let params = LightParams {
        mode,
        color_type,
        r,
        g,
        b,
        brightness,
        speed,
        direction,
    };

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        if direct {
            mgr.apply_lighting_direct(&params, should_persist)?;
        } else {
            mgr.apply_lighting(&params, should_persist)?;
        }
        let sent_count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "packets_sent": sent_count,
                    "params": params,
                    "persisted": should_persist,
                    "direct": direct,
                })
            );
        } else {
            println!(
                "Mock test successful: {} packets staged via MockTransport.",
                sent_count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB and you have appropriate permissions.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    if direct {
        mgr.apply_lighting_direct(&params, should_persist)?;
    } else {
        mgr.apply_lighting(&params, should_persist)?;
    }

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "mode": mode.slug(),
                "mode_name": mode.name(),
                "color": format!("#{:02X}{:02X}{:02X}", r, g, b),
                "color_type": color_type.slug(),
                "brightness_level": brightness_arg,
                "brightness_raw": brightness,
                "speed_level": speed_arg,
                "speed_raw": speed,
                "direction": direction.slug(),
                "persisted": should_persist,
                "direct": direct,
            })
        );
    } else {
        println!("============================================================");
        println!(" Royal Kludge RK855 Lighting Updated Successfully");
        println!("============================================================");
        println!("  Mode:       {} ({})", mode.name(), mode.slug());
        println!(
            "  Color:      #{:02X}{:02X}{:02X} ({})",
            r,
            g,
            b,
            color_type.slug()
        );
        println!(
            "  Brightness: {} (raw hardware level: {})",
            brightness_arg, brightness
        );
        println!(
            "  Speed:      {} (raw hardware level: {})",
            speed_arg, speed
        );
        println!("  Direction:  {}", direction.slug());
        println!(
            "  Storage:    {}",
            if should_persist {
                "Committed to Flash (persists across reconnects)"
            } else {
                "RAM only (live preview, zero flash wear)"
            }
        );
        println!(
            "  Protocol:   {}",
            if direct {
                "Direct sequence (04 18 / 04 11)"
            } else {
                "Atomic transaction (22 packets)"
            }
        );
    }

    Ok(())
}

fn parse_color(s: &str) -> Result<(ColorType, u8, u8, u8)> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("rainbow") || s.eq_ignore_ascii_case("dynamic") {
        return Ok((ColorType::DynamicRainbow, 0, 0, 0));
    }
    if s.eq_ignore_ascii_case("spectrum") || s.eq_ignore_ascii_case("cycle") {
        return Ok((ColorType::SpectrumCycle, 0, 0, 0));
    }
    if s.eq_ignore_ascii_case("red") {
        return Ok((ColorType::RgbCustom, 255, 0, 0));
    }
    if s.eq_ignore_ascii_case("green") {
        return Ok((ColorType::RgbCustom, 0, 255, 0));
    }
    if s.eq_ignore_ascii_case("blue") {
        return Ok((ColorType::RgbCustom, 0, 0, 255));
    }
    if s.eq_ignore_ascii_case("white") {
        return Ok((ColorType::RgbCustom, 255, 255, 255));
    }
    if s.eq_ignore_ascii_case("yellow") {
        return Ok((ColorType::RgbCustom, 255, 255, 0));
    }
    if s.eq_ignore_ascii_case("cyan") {
        return Ok((ColorType::RgbCustom, 0, 255, 255));
    }
    if s.eq_ignore_ascii_case("magenta") || s.eq_ignore_ascii_case("purple") {
        return Ok((ColorType::RgbCustom, 255, 0, 255));
    }
    if s.eq_ignore_ascii_case("orange") {
        return Ok((ColorType::RgbCustom, 255, 128, 0));
    }

    let hex = s
        .strip_prefix('#')
        .or_else(|| s.strip_prefix("0x"))
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);

    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| anyhow::anyhow!("Invalid red hex byte: '{}'", &hex[0..2]))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| anyhow::anyhow!("Invalid green hex byte: '{}'", &hex[2..4]))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| anyhow::anyhow!("Invalid blue hex byte: '{}'", &hex[4..6]))?;
        Ok((ColorType::RgbCustom, r, g, b))
    } else if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16)? * 17;
        let g = u8::from_str_radix(&hex[1..2], 16)? * 17;
        let b = u8::from_str_radix(&hex[2..3], 16)? * 17;
        Ok((ColorType::RgbCustom, r, g, b))
    } else {
        anyhow::bail!(
            "Expected 6-character hex color (e.g. 'ff00ff' or '#ff00ff') or named color ('red', 'cyan', 'rainbow')"
        );
    }
}

fn parse_brightness(val: u8) -> u8 {
    match val {
        0 => 0,
        1 => 3,
        2 => 7,
        3 => 11,
        4 => 15,
        other => other.min(15),
    }
}

fn parse_speed(val: u8) -> u8 {
    match val {
        0 => 1,
        1 => 4,
        2 => 7,
        3 => 9,
        4 => 10,
        other => other.min(10),
    }
}
