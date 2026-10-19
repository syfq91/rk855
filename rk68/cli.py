"""Command-line interface for controlling Royal Kludge RK68 on Linux."""

import argparse
import sys
from typing import Optional
from .config import ConfigManager, KeyboardProfile
from .constants import LightingMode, VENDOR_ID, PRODUCT_ID
from .device import RK68Device, MockRK68Device
from .layout import find_key, KEY_DEFINITIONS


def parse_color(color_str: str) -> tuple:
    """Parse color name or hex into (R, G, B)."""
    color_map = {
        "red": (255, 0, 0),
        "green": (0, 255, 0),
        "blue": (0, 0, 255),
        "white": (255, 255, 255),
        "yellow": (255, 255, 0),
        "cyan": (0, 255, 255),
        "magenta": (255, 0, 255),
        "orange": (255, 128, 0),
        "purple": (128, 0, 255),
    }
    cleaned = color_str.strip().lower()
    if cleaned in color_map:
        return color_map[cleaned]
    if cleaned.startswith("#"):
        cleaned = cleaned[1:]
    if len(cleaned) == 6:
        try:
            return (int(cleaned[0:2], 16), int(cleaned[2:4], 16), int(cleaned[4:6], 16))
        except ValueError:
            pass
    raise ValueError(f"Invalid color format: '{color_str}'. Use hex like '#FF5500' or names like 'red'.")


def cmd_info(args, dev: RK68Device):
    print(f"=== Royal Kludge RK68 Linux Driver ===")
    print(f"Target USB Hardware: VID 0x{VENDOR_ID:04X}, PID 0x{PRODUCT_ID:04X}")
    print(f"Supported Keys: {len(KEY_DEFINITIONS)} physical keys in 68-key matrix\n")

    if args.mock:
        print("[MOCK MODE] Simulating connected device.")
        return 0

    devices = RK68Device.enumerate_devices()
    if not devices:
        print("Status: No RK68 keyboards detected on USB.")
        print("\nTroubleshooting:")
        print(" 1. Make sure keyboard is connected via USB-C cable (not wireless).")
        print(" 2. Ensure udev rule is installed: /etc/udev/rules.d/99-rk68.rules")
        print(" 3. Check dmesg or lsusb for '0c45:8018'.")
        return 1

    print(f"Detected {len(devices)} device(s):")
    for i, d in enumerate(devices, 1):
        print(f" [{i}] Product:      {d.get('product_string', 'RK68')}")
        print(f"     Manufacturer: {d.get('manufacturer_string', 'Sonix')}")
        print(f"     Path:         {d.get('path')}")
        print(f"     Interface:    {d.get('interface_number')}")
    return 0


def cmd_light(args, dev: RK68Device, cfg: ConfigManager):
    mode = LightingMode.from_string(args.mode)
    color = parse_color(args.color) if args.color else (255, 255, 255)
    speed = args.speed
    brightness = args.brightness
    direction = args.direction

    print(f"Applying lighting: Mode={mode.name}, Speed={speed}, Brightness={brightness}, Color={color}")
    with dev:
        dev.set_lighting(
            mode=mode,
            speed=speed,
            brightness=brightness,
            color_rgb=color,
            direction=direction,
        )

    # Optionally persist in active profile
    if args.save:
        active_name = cfg.get_active_profile_name()
        profile = cfg.load_profile(active_name)
        profile.lighting_mode = mode.name.lower()
        profile.speed = speed
        profile.brightness = brightness
        profile.color = f"#{color[0]:02X}{color[1]:02X}{color[2]:02X}"
        profile.direction = direction
        cfg.save_profile(profile)
        print(f"Saved lighting settings to profile '{active_name}'.")

    print("Success: Lighting settings updated.")
    return 0


def cmd_remap(args, dev: RK68Device, cfg: ConfigManager):
    source_key = find_key(args.key)
    if not source_key:
        print(f"Error: Unknown physical key '{args.key}'.")
        print(f"Available keys include: esc, 1-9, Q-P, A-L, Z-M, space, enter, shift, fn, etc.")
        return 1

    target_key = find_key(args.target)
    target_code = target_key.code if target_key else None
    if target_code is None:
        try:
            target_code = int(args.target, 16)
        except ValueError:
            print(f"Error: Target '{args.target}' is neither a known key nor a valid hex scancode.")
            return 1

    print(f"Remapping key '{source_key.name}' (0x{source_key.code:02X}) -> 0x{target_code:02X}...")
    with dev:
        dev.remap_key(source_key.code, target_code)

    if args.save:
        active_name = cfg.get_active_profile_name()
        profile = cfg.load_profile(active_name)
        profile.key_remaps[source_key.name] = hex(target_code)
        cfg.save_profile(profile)
        print(f"Saved remapping to profile '{active_name}'.")

    print("Success: Key remapped.")
    return 0


def cmd_profile(args, dev: RK68Device, cfg: ConfigManager):
    action = args.profile_action

    if action == "list":
        profiles = cfg.list_profiles()
        active = cfg.get_active_profile_name()
        print("Saved Profiles:")
        for p in profiles:
            star = " (active)" if p == active else ""
            print(f"  - {p}{star}")
        return 0

    if action == "load":
        name = args.name
        profile = cfg.load_profile(name)
        cfg.set_active_profile_name(name)
        print(f"Applying profile '{name}': Mode={profile.lighting_mode}, Color={profile.color}...")
        mode = LightingMode.from_string(profile.lighting_mode)
        with dev:
            dev.set_lighting(
                mode=mode,
                speed=profile.speed,
                brightness=profile.brightness,
                color_rgb=profile.get_color_rgb(),
                direction=profile.direction,
            )
            for k_name, sc_val in profile.key_remaps.items():
                k_def = find_key(k_name)
                if k_def:
                    dev.remap_key(k_def.code, int(sc_val, 16))
        print(f"Success: Profile '{name}' loaded and applied.")
        return 0

    if action == "save":
        name = args.name
        profile = cfg.load_profile(name)
        profile.name = name
        cfg.save_profile(profile)
        print(f"Success: Profile '{name}' saved.")
        return 0

    if action == "delete":
        name = args.name
        if cfg.delete_profile(name):
            print(f"Profile '{name}' deleted.")
            return 0
        print(f"Error: Cannot delete profile '{name}'.")
        return 1

    return 0


def cmd_reset(args, dev: RK68Device):
    print("Resetting RK68 keyboard to factory defaults...")
    with dev:
        dev.reset_defaults()
    print("Success: Keyboard reset.")
    return 0


def cmd_test(args, dev: RK68Device):
    """Launch the interactive Key Tester interface."""
    from .gui.main import main as gui_main
    gui_args = ["--tester"]
    if args.mock:
        gui_args.append("--mock")
    return gui_main(gui_args)


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="rk68",
        description="Royal Kludge RK68 Mechanical Keyboard Linux Configuration Utility",
    )
    parser.add_argument(
        "--mock", action="store_true", help="Run with mock device for simulation and testing"
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    # info
    subparsers.add_parser("info", help="Show keyboard detection and USB connection status")

    # test
    test_parser = subparsers.add_parser("test", help="Launch interactive Key Switch Matrix Tester")
    test_parser.add_argument("--mock", action="store_true", help="Run with mock device")

    # light
    light_parser = subparsers.add_parser("light", help="Set RGB backlight mode and parameters")
    light_parser.add_argument(
        "--mode",
        default="wave",
        choices=[m.name.lower() for m in LightingMode],
        help="Lighting mode (default: wave)",
    )
    light_parser.add_argument(
        "--speed", type=int, default=3, choices=range(1, 6), help="Animation speed 1-5"
    )
    light_parser.add_argument(
        "--brightness", type=int, default=3, choices=range(0, 6), help="Brightness 0-5"
    )
    light_parser.add_argument(
        "--color", default="#FF5500", help="Color as hex (#RRGGBB) or name (red, blue, cyan, etc.)"
    )
    light_parser.add_argument(
        "--direction", type=int, default=0, choices=[0, 1], help="Direction (0=left, 1=right)"
    )
    light_parser.add_argument(
        "--save", action="store_true", help="Persist settings in active profile"
    )

    # remap
    remap_parser = subparsers.add_parser("remap", help="Remap physical key scancode")
    remap_parser.add_argument("key", help="Key name to remap (e.g. capslock, esc, a)")
    remap_parser.add_argument("target", help="Target scancode or key name (e.g. control, 0x14)")
    remap_parser.add_argument("--save", action="store_true", help="Persist in active profile")

    # profile
    profile_parser = subparsers.add_parser("profile", help="Manage configuration profiles")
    profile_sub = profile_parser.add_subparsers(dest="profile_action", required=True)
    profile_sub.add_parser("list", help="List all profiles")
    p_load = profile_sub.add_parser("load", help="Load and apply a profile")
    p_load.add_argument("name", help="Profile name")
    p_save = profile_sub.add_parser("save", help="Save current setup as named profile")
    p_save.add_argument("name", help="Profile name")
    p_del = profile_sub.add_parser("delete", help="Delete a profile")
    p_del.add_argument("name", help="Profile name")

    # reset
    subparsers.add_parser("reset", help="Reset keyboard to factory defaults")

    return parser


def main(argv=None):
    parser = create_parser()
    args = parser.parse_args(argv)

    dev = MockRK68Device() if args.mock else RK68Device()
    cfg = ConfigManager()

    try:
        if args.command == "info":
            return cmd_info(args, dev)
        elif args.command == "test":
            return cmd_test(args, dev)
        elif args.command == "light":
            return cmd_light(args, dev, cfg)
        elif args.command == "remap":
            return cmd_remap(args, dev, cfg)
        elif args.command == "profile":
            return cmd_profile(args, dev, cfg)
        elif args.command == "reset":
            return cmd_reset(args, dev)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
