use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rk855_hid::{probe_devices, DeviceInfo};
use rk855_isp::analyze_firmware_bin;
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

    /// Firmware analysis and pre-flight validation tools
    Firmware {
        #[command(subcommand)]
        command: FirmwareCommands,
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
