use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use rk855_config::{parse_rgb_hex, parse_rgb_json, ConfigManager, ProfileFile};
use rk855_hid::{probe_devices, DeviceInfo, HidApiTransport, MockTransport};
use rk855_isp::analyze_firmware_bin;
use rk855_proto::{
    action_code_to_name, find_key_by_name, resolve_action_code, ColorType, CustomRgbMap, Direction,
    LightMode, LightParams, ProfileSlot, RemapEntry, PHYSICAL_KEYS,
};
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

    /// Key remapping and physical matrix customizations
    Remap {
        #[command(subcommand)]
        command: RemapCommands,
    },

    /// Onboard profile switching, export, and import
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Firmware analysis and pre-flight validation tools
    Firmware {
        #[command(subcommand)]
        command: FirmwareCommands,
    },

    /// Custom per-key RGB backlighting illumination
    Rgb {
        #[command(subcommand)]
        command: RgbCommands,
    },

    /// Keystroke macro configuration and protocol status
    Macro {
        #[command(subcommand)]
        command: MacroCommands,
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
enum RemapCommands {
    /// Remap a physical key to a target scancode or action
    Set {
        /// Physical key on RK855 (e.g. 'CAPS', 'esc', 'lctrl', 'space')
        #[arg(short, long)]
        key: String,

        /// Target key, action, or scancode (e.g. 'ESC', 'caps', 'play', 'mouse_left', 'none')
        #[arg(short, long)]
        to: String,

        /// Live test mode: updates in RAM without committing to flash
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to flash across power cycles (default)
        #[arg(long)]
        persist: bool,

        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Dump active key remapping table
    Dump {
        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Query keyboard hardware matrix status via 04 F5
        #[arg(long)]
        query: bool,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Reset all key remappings to factory default matrix
    Reset {
        /// Live test mode: updates in RAM without committing to flash
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to flash across power cycles (default)
        #[arg(long)]
        persist: bool,

        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ProfileCommands {
    /// Export active configuration and remaps to a TOML profile file
    Export {
        /// Destination file path (defaults to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Profile name
        #[arg(long, default_value = "Royal Kludge RK855 Profile")]
        name: String,

        /// Profile slot (1, 2, or 3)
        #[arg(short, long)]
        slot: Option<u8>,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Import and apply configuration from a TOML profile file
    Import {
        /// Path to profile TOML file
        #[arg(short, long)]
        path: PathBuf,

        /// Live test mode: updates in RAM without committing to flash
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to flash across power cycles (default)
        #[arg(long)]
        persist: bool,

        /// Specific HID device path
        #[arg(long)]
        device_path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Switch active onboard hardware profile slot (1, 2, or 3)
    Switch {
        /// Profile slot number (1, 2, or 3)
        #[arg(short, long)]
        slot: u8,

        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output in JSON format
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

#[derive(Subcommand, Debug)]
enum RgbCommands {
    /// Set custom RGB color on an individual key (e.g. `rk855 rgb set-key ESC ff0000`)
    SetKey {
        /// Physical key identifier (e.g. 'esc', 'caps', 'space', 'w', etc.)
        #[arg(short, long)]
        key: Option<String>,

        /// Color hex (e.g. 'ff0000', '#00ff00', '0000ff')
        #[arg(short, long)]
        color: Option<String>,

        /// Positional key argument (shortcut: `rk855 rgb set-key ESC ff0000`)
        #[arg(value_name = "KEY_POS")]
        key_pos: Option<String>,

        /// Positional color argument (shortcut: `rk855 rgb set-key ESC ff0000`)
        #[arg(value_name = "COLOR_POS")]
        color_pos: Option<String>,

        /// Live preview: updates lighting in RAM without flash write
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to flash memory across power cycles
        #[arg(long)]
        persist: bool,

        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Load custom per-key RGB map from a JSON file (e.g. `{"esc": "#ff0000", "w": "#00ff00"}`)
    Load {
        /// Path to JSON RGB mapping file
        #[arg(short, long)]
        path_file: Option<PathBuf>,

        /// Positional file path (shortcut: `rk855 rgb load map.json`)
        #[arg(value_name = "FILE")]
        pos_file: Option<PathBuf>,

        /// Live preview: updates in RAM without flash write
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to flash memory across power cycles
        #[arg(long)]
        persist: bool,

        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Set all 68 keys to a uniform custom color
    SetAll {
        /// Color hex (e.g. 'ff0000', '#00ff00')
        color: String,

        /// Live preview: updates in RAM without flash write
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to flash memory across power cycles
        #[arg(long)]
        persist: bool,

        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Clear all custom per-key RGB illumination (turn off all LEDs)
    Clear {
        /// Live preview: updates in RAM without flash write
        #[arg(long, conflicts_with = "persist")]
        live: bool,

        /// Persist to flash memory across power cycles
        #[arg(long)]
        persist: bool,

        /// Specific HID device path
        #[arg(long)]
        path: Option<String>,

        /// Run against mock transport
        #[arg(long)]
        mock: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Dump active per-key RGB color map
    Dump {
        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum MacroCommands {
    /// Inspect keyboard macro capabilities and protocol verification status
    Status {
        /// Output in JSON format
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
        Commands::Remap { command } => match command {
            RemapCommands::Set {
                key,
                to,
                live,
                persist,
                path,
                mock,
                json,
            } => run_remap_set(key, to, live, persist, path, mock, json)?,
            RemapCommands::Dump {
                path,
                query,
                mock,
                json,
            } => run_remap_dump(path, query, mock, json)?,
            RemapCommands::Reset {
                live,
                persist,
                path,
                mock,
                json,
            } => run_remap_reset(live, persist, path, mock, json)?,
        },
        Commands::Profile { command } => match command {
            ProfileCommands::Export {
                output,
                name,
                slot,
                json,
            } => run_profile_export(output, name, slot, json)?,
            ProfileCommands::Import {
                path,
                live,
                persist,
                device_path,
                mock,
                json,
            } => run_profile_import(path, live, persist, device_path, mock, json)?,
            ProfileCommands::Switch {
                slot,
                path,
                mock,
                json,
            } => run_profile_switch(slot, path, mock, json)?,
        },
        Commands::Firmware { command } => match command {
            FirmwareCommands::Check { path, json } => run_firmware_check(path, json)?,
        },
        Commands::Rgb { command } => match command {
            RgbCommands::SetKey {
                key,
                color,
                key_pos,
                color_pos,
                live,
                persist,
                path,
                mock,
                json,
            } => run_rgb_set_key(
                key, color, key_pos, color_pos, live, persist, path, mock, json,
            )?,
            RgbCommands::Load {
                path_file,
                pos_file,
                live,
                persist,
                path,
                mock,
                json,
            } => run_rgb_load(path_file, pos_file, live, persist, path, mock, json)?,
            RgbCommands::SetAll {
                color,
                live,
                persist,
                path,
                mock,
                json,
            } => run_rgb_set_all(color, live, persist, path, mock, json)?,
            RgbCommands::Clear {
                live,
                persist,
                path,
                mock,
                json,
            } => run_rgb_clear(live, persist, path, mock, json)?,
            RgbCommands::Dump { json } => run_rgb_dump(json)?,
        },
        Commands::Macro { command } => match command {
            MacroCommands::Status { json } => run_macro_status(json)?,
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

fn get_profile_store_path() -> PathBuf {
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile)
            .join(".rk855")
            .join("active_profile.toml")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".rk855")
            .join("active_profile.toml")
    } else {
        PathBuf::from("active_profile.toml")
    }
}

fn load_or_create_active_profile() -> ProfileFile {
    let path = get_profile_store_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(profile) = toml::from_str::<ProfileFile>(&content) {
                return profile;
            }
        }
    }
    ProfileFile::default()
}

fn save_active_profile(profile: &ProfileFile) -> Result<()> {
    let path = get_profile_store_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = toml::to_string_pretty(profile)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn run_remap_set(
    key_str: String,
    to_str: String,
    live: bool,
    persist: bool,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let phys_key = find_key_by_name(&key_str).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown physical key '{}'. Use standard names like 'CAPS', 'ESC', 'LCTRL', 'A', etc.",
            key_str
        )
    })?;

    let target_code = resolve_action_code(&to_str).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown target action or key '{}'. Use key names ('ESC', 'A'), media actions ('play', 'mute'), mouse actions ('mouse_left'), or 'none'.",
            to_str
        )
    })?;

    let should_persist = if live { false } else { persist || !live };

    let mut profile = load_or_create_active_profile();
    profile
        .remaps
        .insert(phys_key.name.to_string(), to_str.clone());
    let _ = save_active_profile(&profile);

    // Build remap entries from all accumulated active remaps
    let mut entries = Vec::new();
    for (k, v) in &profile.remaps {
        if let Some(pk) = find_key_by_name(k) {
            if let Some(tc) = resolve_action_code(v) {
                entries.push(RemapEntry::new(pk.scancode, tc));
            }
        }
    }

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        mgr.apply_remap(&entries, should_persist)?;
        let count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "packets_sent": count,
                    "key": phys_key.name,
                    "phys_scancode": format!("0x{:02X}", phys_key.scancode),
                    "to": to_str,
                    "target_code": format!("0x{:02X}", target_code),
                    "persisted": should_persist,
                })
            );
        } else {
            println!(
                "Mock test successful: {} packets staged via MockTransport.",
                count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    mgr.apply_remap(&entries, should_persist)?;

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "key": phys_key.name,
                "phys_scancode": format!("0x{:02X}", phys_key.scancode),
                "to": to_str,
                "target_code": format!("0x{:02X}", target_code),
                "target_name": action_code_to_name(target_code),
                "persisted": should_persist,
                "total_active_remaps": entries.len(),
            })
        );
    } else {
        println!("============================================================");
        println!(" Royal Kludge RK855 Key Remapped Successfully");
        println!("============================================================");
        println!(
            "  Physical Key:  {} (scancode: 0x{:02X}, row: {})",
            phys_key.label, phys_key.scancode, phys_key.row
        );
        println!(
            "  Target Action: {} (code: 0x{:02X} / {})",
            action_code_to_name(target_code),
            target_code,
            to_str
        );
        println!(
            "  Storage:       {}",
            if should_persist {
                "Committed to Flash (persists across replug)"
            } else {
                "RAM only (live preview)"
            }
        );
        println!("  Total Remaps:  {} active mapping(s)", entries.len());
    }

    Ok(())
}

fn run_remap_dump(
    path_opt: Option<String>,
    query: bool,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let profile = load_or_create_active_profile();

    let mut query_bytes: Option<Vec<u8>> = None;
    if query && !mock {
        let transport_res = match path_opt {
            Some(ref p) => HidApiTransport::open_path(p),
            None => HidApiTransport::open_first_candidate(),
        };
        if let Ok(transport) = transport_res {
            let mut mgr = ConfigManager::new(transport);
            if let Ok(data) = mgr.query_status_matrix() {
                query_bytes = Some(data);
            }
        }
    }

    if output_json {
        let mut list = Vec::new();
        for (k, v) in &profile.remaps {
            let phys = find_key_by_name(k);
            let target_code = resolve_action_code(v).unwrap_or(0);
            list.push(serde_json::json!({
                "key": k,
                "label": phys.map(|p| p.label).unwrap_or(""),
                "scancode": phys.map(|p| format!("0x{:02X}", p.scancode)).unwrap_or_default(),
                "to": v,
                "target_code": format!("0x{:02X}", target_code),
                "target_name": action_code_to_name(target_code),
            }));
        }
        println!(
            "{}",
            serde_json::json!({
                "profile_name": profile.name,
                "version": profile.version,
                "slot": profile.slot,
                "remaps": list,
                "hardware_readback": query_bytes.map(|b| b.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<_>>().join(" ")),
            })
        );
        return Ok(());
    }

    println!("===============================================================================");
    println!(" Royal Kludge RK855 Active Key Remappings");
    println!("===============================================================================");
    println!("Profile: {} (Schema v{})", profile.name, profile.version);
    if let Some(s) = profile.slot {
        println!("Hardware Slot: Profile {}", s);
    }
    println!();

    if profile.remaps.is_empty() {
        println!("No custom key remaps configured (keyboard using factory default matrix).");
        println!("Run 'rk855 remap set --key <KEY> --to <TARGET>' to add a remap.");
    } else {
        println!(
            "{:<15} {:<10} {:<18} {:<10}",
            "Physical Key", "Scancode", "Target Action", "Target Code"
        );
        println!("{:-<15} {:-<10} {:-<18} {:-<10}", "", "", "", "");
        for (k, v) in &profile.remaps {
            let phys = find_key_by_name(k);
            let phys_code = phys
                .map(|p| format!("0x{:02X}", p.scancode))
                .unwrap_or_else(|| "??".to_string());
            let target_code = resolve_action_code(v).unwrap_or(0);
            println!(
                "{:<15} {:<10} {:<18} 0x{:02X}",
                phys.map(|p| p.label).unwrap_or(k.as_str()),
                phys_code,
                action_code_to_name(target_code),
                target_code
            );
        }
    }

    if let Some(ref data) = query_bytes {
        println!();
        println!(
            "Hardware Matrix Status Read-Back (04 F5 response, {} bytes):",
            data.len()
        );
        for chunk in data.chunks(16) {
            let hex_str: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
            println!("  {}", hex_str.join(" "));
        }
    }

    Ok(())
}

fn run_remap_reset(
    live: bool,
    persist: bool,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let should_persist = if live { false } else { persist || !live };

    let mut profile = load_or_create_active_profile();
    let cleared_count = profile.remaps.len();
    profile.remaps.clear();
    let _ = save_active_profile(&profile);

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        mgr.reset_remap(should_persist)?;
        let count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "packets_sent": count,
                    "cleared_remaps": cleared_count,
                    "persisted": should_persist,
                })
            );
        } else {
            println!(
                "Mock test successful: reset table staged via MockTransport ({} packets).",
                count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    mgr.reset_remap(should_persist)?;

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "cleared_remaps": cleared_count,
                "persisted": should_persist,
            })
        );
    } else {
        println!("============================================================");
        println!(" Royal Kludge RK855 Remaps Reset Successfully");
        println!("============================================================");
        println!("  Cleared {} custom mapping(s).", cleared_count);
        println!("  Keyboard matrix restored to factory default layout.");
        println!(
            "  Storage: {}",
            if should_persist {
                "Committed to Flash (persists across replug)"
            } else {
                "RAM only"
            }
        );
    }

    Ok(())
}

fn run_profile_export(
    output_opt: Option<PathBuf>,
    name: String,
    slot_opt: Option<u8>,
    output_json: bool,
) -> Result<()> {
    let mut profile = load_or_create_active_profile();
    profile.name = name;
    if let Some(s) = slot_opt {
        profile.slot = Some(s);
    }

    let toml_str = toml::to_string_pretty(&profile)?;

    if let Some(dest) = output_opt {
        std::fs::write(&dest, &toml_str)
            .with_context(|| format!("Failed to write profile TOML to '{}'", dest.display()))?;
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "saved_to": dest.display().to_string(),
                    "profile": profile,
                })
            );
        } else {
            println!("Profile exported successfully to: {}", dest.display());
        }
    } else if output_json {
        println!("{}", serde_json::to_string_pretty(&profile)?);
    } else {
        println!("{toml_str}");
    }

    Ok(())
}

fn run_profile_import(
    path: PathBuf,
    live: bool,
    persist: bool,
    device_path: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read profile file '{}'", path.display()))?;

    let should_persist = if live { false } else { persist || !live };

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        let imported = mgr.import_profile_toml(&content, true, should_persist)?;
        let sent_count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "packets_sent": sent_count,
                    "profile": imported,
                    "persisted": should_persist,
                })
            );
        } else {
            println!(
                "Mock import successful: {} packets staged for profile '{}'.",
                sent_count, imported.name
            );
        }
        return Ok(());
    }

    let transport = match device_path {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    let imported = mgr.import_profile_toml(&content, true, should_persist)?;

    // If lighting configuration was included, also apply it
    if let Some(ref l) = imported.lighting {
        if let Some(mode) = LightMode::from_name_or_slug(&l.mode) {
            let (color_type, r, g, b) =
                parse_color(&l.color).unwrap_or((ColorType::RgbCustom, 255, 0, 255));
            let brightness = parse_brightness(l.brightness);
            let speed = parse_speed(l.speed);
            let direction =
                Direction::from_str_name(&l.direction).unwrap_or(Direction::LeftToRight);
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
            let _ = mgr.apply_lighting(&params, should_persist);
        }
    }

    let _ = save_active_profile(&imported);

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "profile": imported,
                "persisted": should_persist,
            })
        );
    } else {
        println!("============================================================");
        println!(" Royal Kludge RK855 Profile Imported Successfully");
        println!("============================================================");
        println!("  Name:        {}", imported.name);
        println!("  Version:     {}", imported.version);
        if let Some(s) = imported.slot {
            println!("  Slot:        Profile {}", s);
        }
        println!(
            "  Remaps:      {} mapping(s) applied",
            imported.remaps.len()
        );
        if let Some(ref l) = imported.lighting {
            println!("  Lighting:    mode={}, color={}", l.mode, l.color);
        }
        println!(
            "  Storage:     {}",
            if should_persist {
                "Committed to Flash"
            } else {
                "RAM only"
            }
        );
    }

    Ok(())
}

fn run_profile_switch(
    slot_num: u8,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let slot = ProfileSlot::from_u8(slot_num).ok_or_else(|| {
        anyhow::anyhow!("Invalid profile slot {}. Expected 1, 2, or 3.", slot_num)
    })?;

    let mut profile = load_or_create_active_profile();
    profile.slot = Some(slot_num);
    let _ = save_active_profile(&profile);

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        mgr.switch_profile(slot)?;
        let count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "packets_sent": count,
                    "slot": slot_num,
                })
            );
        } else {
            println!(
                "Mock test successful: switched to Profile {} via MockTransport ({} packets).",
                slot_num, count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    mgr.switch_profile(slot)?;

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "slot": slot_num,
                "profile_opcode": format!("0x{:02X}", 0xF0 + slot_num),
            })
        );
    } else {
        println!("============================================================");
        println!(" Active Onboard Profile Switched");
        println!("============================================================");
        println!(
            "  Active Slot: Profile {} (opcode: 0x{:02X})",
            slot_num,
            0xF0 + slot_num
        );
        println!("  Target MCU:  Address remapped to Flash slot offset");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_rgb_set_key(
    key_opt: Option<String>,
    color_opt: Option<String>,
    key_pos: Option<String>,
    color_pos: Option<String>,
    live: bool,
    persist: bool,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let key_str = key_opt
        .or(key_pos)
        .ok_or_else(|| anyhow!("Missing key identifier. Usage: rk855 rgb set-key <KEY> <COLOR>"))?;
    let color_str = color_opt
        .or(color_pos)
        .ok_or_else(|| anyhow!("Missing color. Usage: rk855 rgb set-key <KEY> <COLOR>"))?;

    let phys_key = find_key_by_name(&key_str).ok_or_else(|| {
        anyhow!(
            "Unknown key identifier '{}'. Run 'rk855 remap dump' to inspect supported keys.",
            key_str
        )
    })?;

    let (r, g, b) = parse_rgb_hex(&color_str)?;
    let is_persist = persist || !live;

    // Load active profile and update custom_rgb
    let mut profile = load_or_create_active_profile();
    let hex_val = format!("#{:02X}{:02X}{:02X}", r, g, b);
    let all_keys: std::collections::BTreeMap<String, String> = {
        let rgb_map_ref = profile
            .custom_rgb
            .get_or_insert_with(std::collections::BTreeMap::new);
        rgb_map_ref.insert(phys_key.name.to_string(), hex_val.clone());
        rgb_map_ref.clone()
    };
    let _ = save_active_profile(&profile);

    // Build CustomRgbMap from all stored keys
    let mut custom_map = CustomRgbMap::new();
    for (k_name, hex) in all_keys.iter() {
        if let Ok((kr, kg, kb)) = parse_rgb_hex(hex) {
            let _ = custom_map.set_key_by_name(k_name, kr, kg, kb);
        }
    }

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        mgr.apply_custom_rgb(&custom_map, is_persist)?;
        let count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "key": phys_key.name,
                    "color": hex_val,
                    "matrix_index": phys_key.matrix_index,
                    "buffer_offset": phys_key.rgb_buffer_offset(),
                    "packets_sent": count,
                    "persist": is_persist,
                })
            );
        } else {
            println!(
                "Mock test successful: set key '{}' to {} via MockTransport ({} packets sent).",
                phys_key.name, hex_val, count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    mgr.apply_custom_rgb(&custom_map, is_persist)?;

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "key": phys_key.name,
                "label": phys_key.label,
                "color": hex_val,
                "matrix_index": phys_key.matrix_index,
                "buffer_offset": phys_key.rgb_buffer_offset(),
                "persist": is_persist,
            })
        );
    } else {
        println!("============================================================");
        println!(" Custom Per-Key RGB Applied");
        println!("============================================================");
        println!("  Key:           {} ({})", phys_key.label, phys_key.name);
        println!("  Scan Code:     0x{:02X}", phys_key.scancode);
        println!("  Matrix Index:  {}", phys_key.matrix_index);
        println!(
            "  Payload Offset:{} (quad: [0x80, {}, {}, {}])",
            phys_key.rgb_buffer_offset(),
            r,
            g,
            b
        );
        println!(
            "  Color:         {} (R: {}, G: {}, B: {})",
            hex_val, r, g, b
        );
        println!(
            "  Persistence:   {}",
            if is_persist {
                "Persisted to onboard flash (04 02)"
            } else {
                "RAM preview only (omitted 04 02)"
            }
        );
        println!(
            "  Active Keys:   {} total keys lit in active custom profile",
            all_keys.len()
        );
    }

    Ok(())
}

fn run_rgb_load(
    path_file: Option<PathBuf>,
    pos_file: Option<PathBuf>,
    live: bool,
    persist: bool,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let file_path = path_file
        .or(pos_file)
        .ok_or_else(|| anyhow!("Missing JSON file path. Usage: rk855 rgb load <map.json>"))?;
    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read RGB JSON file: {}", file_path.display()))?;

    let rgb_map = parse_rgb_json(&content)?;
    let is_persist = persist || !live;

    // Merge into active profile
    let mut profile = load_or_create_active_profile();
    let stored_map = profile
        .custom_rgb
        .get_or_insert_with(std::collections::BTreeMap::new);
    let parsed_json: std::collections::BTreeMap<String, String> = serde_json::from_str(&content)?;
    for (k, v) in parsed_json {
        if let Ok((r, g, b)) = parse_rgb_hex(&v) {
            stored_map.insert(k, format!("#{:02X}{:02X}{:02X}", r, g, b));
        }
    }
    let _ = save_active_profile(&profile);

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        mgr.apply_custom_rgb(&rgb_map, is_persist)?;
        let count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "file": file_path.display().to_string(),
                    "packets_sent": count,
                    "persist": is_persist,
                })
            );
        } else {
            println!(
                "Mock test successful: loaded RGB map from {} ({} packets sent).",
                file_path.display(),
                count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    mgr.apply_custom_rgb(&rgb_map, is_persist)?;

    let lit_count = rgb_map.colors.iter().filter(|c| c.is_some()).count();

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "file": file_path.display().to_string(),
                "keys_configured": lit_count,
                "persist": is_persist,
            })
        );
    } else {
        println!("============================================================");
        println!(" Custom Per-Key RGB Map Loaded");
        println!("============================================================");
        println!("  Source File:   {}", file_path.display());
        println!("  Keys Lit:      {} / 68 physical keys", lit_count);
        println!(
            "  Persistence:   {}",
            if is_persist {
                "Persisted to onboard flash (04 02)"
            } else {
                "RAM preview only (omitted 04 02)"
            }
        );
    }

    Ok(())
}

fn run_rgb_set_all(
    color_str: String,
    live: bool,
    persist: bool,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let (r, g, b) = parse_rgb_hex(&color_str)?;
    let is_persist = persist || !live;
    let hex_val = format!("#{:02X}{:02X}{:02X}", r, g, b);

    let mut custom_map = CustomRgbMap::new();
    custom_map.set_all(r, g, b);

    // Update active profile
    let mut profile = load_or_create_active_profile();
    let stored_map = profile
        .custom_rgb
        .get_or_insert_with(std::collections::BTreeMap::new);
    stored_map.clear();
    for key in PHYSICAL_KEYS.iter() {
        stored_map.insert(key.name.to_string(), hex_val.clone());
    }
    let _ = save_active_profile(&profile);

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        mgr.apply_custom_rgb(&custom_map, is_persist)?;
        let count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "success",
                    "mock": true,
                    "color": hex_val,
                    "packets_sent": count,
                })
            );
        } else {
            println!(
                "Mock test successful: set all 68 keys to {} ({} packets sent).",
                hex_val, count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    mgr.apply_custom_rgb(&custom_map, is_persist)?;

    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "mock": false,
                "color": hex_val,
                "total_keys": 68,
                "persist": is_persist,
            })
        );
    } else {
        println!("============================================================");
        println!(" Uniform RGB Applied to All Keys");
        println!("============================================================");
        println!("  Color:       {} (R: {}, G: {}, B: {})", hex_val, r, g, b);
        println!("  Keys:        All 68 physical keys configured");
        println!(
            "  Persistence: {}",
            if is_persist {
                "Persisted to onboard flash (04 02)"
            } else {
                "RAM preview only (omitted 04 02)"
            }
        );
    }

    Ok(())
}

fn run_rgb_clear(
    live: bool,
    persist: bool,
    path_opt: Option<String>,
    mock: bool,
    output_json: bool,
) -> Result<()> {
    let is_persist = persist || !live;

    let mut profile = load_or_create_active_profile();
    if let Some(ref mut m) = profile.custom_rgb {
        m.clear();
    }
    let _ = save_active_profile(&profile);

    let empty_map = CustomRgbMap::new();

    if mock {
        let mut mgr = ConfigManager::new(MockTransport::new());
        mgr.apply_custom_rgb(&empty_map, is_persist)?;
        let count = mgr.transport_mut().sent_packets.len();
        if output_json {
            println!(
                "{}",
                serde_json::json!({ "status": "success", "mock": true, "packets_sent": count })
            );
        } else {
            println!(
                "Mock test successful: cleared all custom RGB ({} packets sent).",
                count
            );
        }
        return Ok(());
    }

    let transport = match path_opt {
        Some(ref p) => HidApiTransport::open_path(p)
            .with_context(|| format!("Failed to open HID device at path '{}'", p))?,
        None => HidApiTransport::open_first_candidate()
            .context("Failed to find or open RK855 primary config interface (Interface 0). Ensure keyboard is connected via USB.")?,
    };

    let mut mgr = ConfigManager::new(transport);
    mgr.apply_custom_rgb(&empty_map, is_persist)?;

    if output_json {
        println!(
            "{}",
            serde_json::json!({ "status": "success", "mock": false, "persist": is_persist })
        );
    } else {
        println!("============================================================");
        println!(" Custom Per-Key RGB Cleared");
        println!("============================================================");
        println!("  All 68 physical LEDs turned off");
        println!(
            "  Persistence: {}",
            if is_persist {
                "Persisted to onboard flash (04 02)"
            } else {
                "RAM preview only (omitted 04 02)"
            }
        );
    }

    Ok(())
}

fn run_rgb_dump(output_json: bool) -> Result<()> {
    let profile = load_or_create_active_profile();
    let custom_rgb = profile.custom_rgb.unwrap_or_default();

    if output_json {
        println!("{}", serde_json::to_string_pretty(&custom_rgb)?);
        return Ok(());
    }

    println!("============================================================");
    println!(" Active Custom Per-Key RGB Illumination");
    println!("============================================================");
    if custom_rgb.is_empty() {
        println!("  No custom per-key RGB colors configured in active profile.");
        println!("  Tip: Use 'rk855 rgb set-key <KEY> <COLOR>' to illuminate individual keys.");
        println!("       Use 'rk855 rgb load <map.json>' to load an RGB key map.");
        return Ok(());
    }

    println!("  Total configured keys: {}", custom_rgb.len());
    println!();
    println!(
        "  {:<16} {:<10} {:<12} {:<14} {:<10}",
        "Key", "ScanCode", "MatrixIndex", "PayloadOffset", "Color"
    );
    println!(
        "  {:-<16} {:-<10} {:-<12} {:-<14} {:-<10}",
        "", "", "", "", ""
    );

    for (k_name, hex_color) in &custom_rgb {
        if let Some(key) = find_key_by_name(k_name) {
            println!(
                "  {:<16} 0x{:02X}       {:<12} {:<14} {}",
                format!("{} ({})", key.label, key.name),
                key.scancode,
                key.matrix_index,
                format!("{}+4", key.rgb_buffer_offset()),
                hex_color
            );
        } else {
            println!(
                "  {:<16} {:<10} {:<12} {:<14} {}",
                k_name, "-", "-", "-", hex_color
            );
        }
    }

    Ok(())
}

fn run_macro_status(output_json: bool) -> Result<()> {
    if output_json {
        println!(
            "{}",
            serde_json::json!({
                "subcommand_begin_macro": "0x04 0x19",
                "subcommand_macro_data": "0x04 0x15",
                "flash_storage_offset": "0x00009340",
                "verification_status": "CFG-11 unverified (requires USB packet capture of recorded macro)",
                "desktop_actions_delegation": "Categories 126-129 (Launch App, Open URL, Text Injection) are host software hooks, not executable autonomously on MCU hardware",
            })
        );
        return Ok(());
    }

    println!("============================================================");
    println!(" Keyboard Macro Capabilities & Verification Status");
    println!("============================================================");
    println!("  Opcode 04 19:  Begin Macro Configuration (Verified by static disassembly)");
    println!("  Opcode 04 15:  Macro Streaming Data (Pending verified packet capture: CFG-11)");
    println!("  Flash Target:  Sector 0x00009340 (Profile 3 / Macro user storage block)");
    println!();
    println!("  Desktop Actions Policy (Phase 5 Task 4):");
    println!("    - Categories 126-129 (Launch App, Open URL, Text Injection) are host-side");
    println!("      Windows hooks executed by vendor software (DeviceDriver.exe).");
    println!("    - They cannot execute autonomously on keyboard MCU hardware without");
    println!("      a running host daemon.");
    println!("    - Per rewrite plan ground rules, unverified formats remain safely unsupported.");

    Ok(())
}
