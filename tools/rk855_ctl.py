#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "hidapi>=0.14.0",
# ]
# ///
"""
Royal Kludge RK855 (RK68) RGB Backlight & Device Control Tool
Reverse-engineered from the official Windows configurator software.

Protocol: USB HID Feature Reports (65 bytes: [ReportID(0x00), Opcode(0x04), ...])
Device: VID 0x0C45, PID 0x8018
"""

import argparse
import sys
import time

try:
    import hid
except ImportError:
    hid = None

VID = 0x0C45
PID = 0x8018

LIGHT_MODES = {
    0: "Static",
    1: "Single On",
    2: "Single Off",
    3: "Glittering",
    4: "Falling",
    5: "Color Cycling",
    6: "Breathing",
    7: "Spectrum",
    8: "Outward",
    9: "Scrolling",
    10: "Wave",
    11: "Accumulate",
    12: "Windmill",
    13: "Colorful Waterfall",
    14: "Blossoming / Ripples",
    15: "Flowing",
    16: "Constant Rainbow",
    17: "Sine Wave",
    18: "Shadow",
    19: "Backlight Off",
}

NAMED_COLORS = {
    "red": (255, 0, 0),
    "green": (0, 255, 0),
    "blue": (0, 0, 255),
    "cyan": (0, 255, 255),
    "magenta": (255, 0, 255),
    "yellow": (255, 255, 0),
    "white": (255, 255, 255),
    "orange": (255, 128, 0),
    "purple": (128, 0, 255),
    "pink": (255, 105, 180),
}


def parse_color(color_str: str) -> tuple[int, int, int]:
    """Parse named color, hex string (#RRGGBB or RRGGBB), or comma-separated R,G,B."""
    c_lower = color_str.strip().lower()
    if c_lower in NAMED_COLORS:
        return NAMED_COLORS[c_lower]

    if c_lower.startswith("#"):
        c_lower = c_lower[1:]

    if len(c_lower) == 6:
        try:
            r = int(c_lower[0:2], 16)
            g = int(c_lower[2:4], 16)
            b = int(c_lower[4:6], 16)
            return (r, g, b)
        except ValueError:
            pass

    if "," in c_lower:
        parts = c_lower.split(",")
        if len(parts) == 3:
            try:
                r = max(0, min(255, int(parts[0].strip())))
                g = max(0, min(255, int(parts[1].strip())))
                b = max(0, min(255, int(parts[2].strip())))
                return (r, g, b)
            except ValueError:
                pass

    raise argparse.ArgumentTypeError(f"Invalid color specification: '{color_str}'. Use hex (#FF00FF), named color, or R,G,B.")


def open_device(vid: int = VID, pid: int = PID) -> hid.device:
    """Finds and opens the configuration HID interface for the RK855."""
    if hid is None:
        print("Error: 'hidapi' module not found. Install it with: pip install hidapi", file=sys.stderr)
        sys.exit(1)
    dev = hid.device()
    try:
        dev.open(vid, pid)
        return dev
    except Exception as e:
        # Check permissions / availability
        matched = [d for d in hid.enumerate() if d["vendor_id"] == vid and d["product_id"] == pid]
        if not matched:
            raise RuntimeError(f"RK855 keyboard not found (VID: 0x{vid:04X}, PID: 0x{pid:04X}). Check USB connection.")
        raise PermissionError(
            f"Found RK855, but failed to open device ({e}). Ensure you have write permissions (e.g. udev rules) or run with sudo."
        )


def send_feature(dev: hid.device, payload: list[int] | bytes) -> None:
    """Sends a 65-byte Feature Report with Report ID 0x00 and trailing zero-padding."""
    buf = bytes([0x00]) + bytes(payload).ljust(64, b"\x00")
    dev.send_feature_report(buf)
    time.sleep(0.005)


def apply_lighting(
    mode_id: int,
    brightness: int = 3,
    speed: int = 2,
    r: int = 255,
    g: int = 0,
    b: int = 255,
    direction: int = 0,
    color_type: int = 1,
) -> None:
    """Configures the lighting mode and commits it to onboard memory."""
    dev = open_device()
    try:
        print(f"[+] Connected to Royal Kludge RK855")
        print(
            f"[+] Setting lighting: mode={mode_id} ({LIGHT_MODES.get(mode_id, 'Custom')}), "
            f"color=({r},{g},{b}), brightness={brightness}, speed={speed}, direction={direction}"
        )

        # 1. Begin Light Configuration
        send_feature(dev, [0x04, 0x18])

        # 2. Set Light Mode Parameters
        send_feature(dev, [0x04, 0x11, mode_id, color_type, r, g, b, brightness, speed, direction])

        # 3. Commit Changes to Flash
        send_feature(dev, [0x04, 0x02])

        # 4. Sync Live Lighting Engine
        send_feature(dev, [0x04, 0xF0])
        print("[+] Lighting settings applied successfully.")
    finally:
        dev.close()


def set_custom_rgb(colors_rgb: list[tuple[int, int, int]]) -> None:
    """Uploads a custom 68-key per-key RGB matrix (3 consecutive HID packets)."""
    if len(colors_rgb) > 68:
        colors_rgb = colors_rgb[:68]
    elif len(colors_rgb) < 68:
        colors_rgb = colors_rgb + [(0, 0, 0)] * (68 - len(colors_rgb))

    # Flatten into byte stream
    flat_bytes = bytearray()
    for r, g, b in colors_rgb:
        flat_bytes.extend([r, g, b])

    dev = open_device()
    try:
        print("[+] Uploading custom 68-key per-key RGB matrix...")
        send_feature(dev, [0x04, 0x18])

        # 3 packets: 24 keys (72B) / 24 keys (72B) / 20 keys (60B)
        # HID feature packets hold up to 60 payload bytes per packet
        offsets = [0, 60, 120, 180]
        for idx in range(len(offsets) - 1):
            start = offsets[idx]
            end = min(offsets[idx + 1], len(flat_bytes))
            chunk = flat_bytes[start:end]
            send_feature(dev, [0x04, 0x16, idx, len(chunk)] + list(chunk))

        send_feature(dev, [0x04, 0x02])
        send_feature(dev, [0x04, 0xF0])
        print("[+] Custom matrix applied successfully.")
    finally:
        dev.close()


def detect_devices() -> None:
    """Scans and lists attached RK855 devices and current operating modes."""
    if hid is None:
        print("Error: 'hidapi' module not found. Install it with: pip install hidapi", file=sys.stderr)
        sys.exit(1)
    devices = hid.enumerate()
    found = False
    for d in devices:
        vid = d["vendor_id"]
        pid = d["product_id"]
        if vid == VID and pid == PID:
            print(f"[+] Found RK855 Keyboard in Normal Operating Mode:")
            print(f"    Path:         {d['path']}")
            print(f"    Manufacturer: {d.get('manufacturer_string', 'Royal Kludge')}")
            print(f"    Product:      {d.get('product_string', 'RK855 Keyboard')}")
            print(f"    Release:      0x{d.get('release_number', 0):04X}")
            print(f"    Interface:    {d.get('interface_number', 0)}")
            found = True
        elif vid == VID and pid == 0x82AA:
            print(f"[+] Found RK855 Keyboard in Sonix Bootloader (ISP) Mode:")
            print(f"    Path:         {d['path']}")
            print(f"    Product:      {d.get('product_string', 'SN32F240B Bootloader')}")
            found = True

    if not found:
        print(f"[-] No Royal Kludge RK855 keyboard found.")


def main():
    parser = argparse.ArgumentParser(
        description="Royal Kludge RK855 (RK68) Backlight and Device Control Tool",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")

    # Command: info
    subparsers.add_parser("info", help="Scan and display connected RK855 devices")

    # Command: list-modes
    subparsers.add_parser("list-modes", help="List all built-in lighting effects and IDs")

    # Command: mode
    p_mode = subparsers.add_parser("mode", help="Set lighting effect, color, brightness, and speed")
    p_mode.add_argument("mode", type=int, help="Lighting mode ID (0-19)")
    p_mode.add_argument("--color", "-c", type=parse_color, default=(255, 0, 255), help="Color (e.g. red, #00FFCC, 255,0,128)")
    p_mode.add_argument("--brightness", "-b", type=int, choices=range(0, 5), default=3, help="Brightness (0-4)")
    p_mode.add_argument("--speed", "-s", type=int, choices=range(0, 5), default=2, help="Effect speed (0-4)")
    p_mode.add_argument("--direction", "-d", type=int, choices=[0, 1], default=0, help="0: Left-to-right, 1: Right-to-left")

    # Command: color
    p_color = subparsers.add_parser("color", help="Quick shortcut to set static solid backlight color")
    p_color.add_argument("color", type=parse_color, help="Color name or hex code (e.g. cyan, #FF5500)")
    p_color.add_argument("--brightness", "-b", type=int, choices=range(0, 5), default=3, help="Brightness (0-4)")

    # Command: off
    subparsers.add_parser("off", help="Turn keyboard backlighting completely off")

    args = parser.parse_args()

    if not args.command or args.command == "info":
        detect_devices()
        return

    if args.command == "list-modes":
        print("Available Built-in Lighting Modes:")
        for mode_id, name in LIGHT_MODES.items():
            print(f"  [{mode_id:2d}] {name}")
        return

    if args.command == "mode":
        r, g, b = args.color
        apply_lighting(
            mode_id=args.mode,
            brightness=args.brightness,
            speed=args.speed,
            r=r,
            g=g,
            b=b,
            direction=args.direction,
        )
        return

    if args.command == "color":
        r, g, b = args.color
        apply_lighting(mode_id=0, brightness=args.brightness, speed=2, r=r, g=g, b=b)
        return

    if args.command == "off":
        apply_lighting(mode_id=19, brightness=0, speed=0, r=0, g=0, b=0)
        return


if __name__ == "__main__":
    main()
