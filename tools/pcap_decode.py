#!/usr/bin/env python3
"""
RK855 USB Packet Decoder (PCAP / PCAPNG)
Extracts USB HID SET_REPORT / GET_REPORT control and interrupt packets,
decodes Royal Kludge RK855 opcodes and payloads, and writes annotated traces.

Zero external dependencies - uses Python standard library struct and pathlib.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
import struct
import sys
from typing import BinaryIO, Iterator, Optional

# Link types
LINKTYPE_USB_LINUX = 189
LINKTYPE_USB_LINUX_MMAPPED = 220
LINKTYPE_USBPCAP = 249

# Opcode dictionaries
LIGHT_MODES = {
    0x00: "Static",
    0x01: "Single On",
    0x02: "Single Off",
    0x03: "Glittering",
    0x04: "Falling",
    0x05: "Color Cycling",
    0x06: "Breathing",
    0x07: "Spectrum",
    0x08: "Outward",
    0x09: "Scrolling",
    0x0A: "Wave",
    0x0B: "Accumulate",
    0x0C: "Windmill",
    0x0D: "Colorful Waterfall",
    0x0E: "Blossoming / Ripples",
    0x0F: "Flowing",
    0x10: "Constant Rainbow",
    0x11: "Sine Wave",
    0x12: "Shadow",
    0x13: "Backlight Off",
}

COLOR_TYPES = {
    0x00: "Fixed / Single Color",
    0x01: "RGB 24-bit Custom",
    0x07: "Spectrum 7-Color Cycle",
    0x08: "Random / Dynamic Rainbow",
}

DIRECTIONS = {
    0x00: "Left-to-Right / Clockwise",
    0x01: "Right-to-Left / Counter-Clockwise",
}

ISP_TRIGGER_MAGIC = bytes([0xAA, 0x42, 0x89, 0x5A, 0xFF, 0x71, 0x62, 0xCC])


@dataclass
class UsbPacket:
    timestamp_sec: float
    data: bytes
    direction_in: bool  # True: IN (Device -> Host), False: OUT (Host -> Device)
    is_control: bool
    bmRequestType: Optional[int] = None
    bRequest: Optional[int] = None
    wValue: Optional[int] = None
    wIndex: Optional[int] = None
    wLength: Optional[int] = None


def parse_pcap(f: BinaryIO) -> Iterator[UsbPacket]:
    """Parse classic PCAP file."""
    header = f.read(24)
    if len(header) < 24:
        return
    magic, maj, min_ver, thiszone, sigfigs, snaplen, linktype = struct.unpack("<IHHiIII", header)
    endian = "<"
    if magic == 0xD4C3B2A1 or magic == 0x4D3CB2A1:
        endian = ">"
    elif magic != 0xA1B2C3D4 and magic != 0xA1B23C4D:
        # Not a valid PCAP magic
        return

    while True:
        pkt_hdr = f.read(16)
        if len(pkt_hdr) < 16:
            break
        ts_sec, ts_usec, incl_len, orig_len = struct.unpack(f"{endian}IIII", pkt_hdr)
        data = f.read(incl_len)
        ts = ts_sec + (ts_usec / 1_000_000.0)

        pkt = parse_linktype_packet(linktype, data, ts)
        if pkt:
            yield pkt


def parse_pcapng(f: BinaryIO) -> Iterator[UsbPacket]:
    """Parse PCAPNG file."""
    endian = "<"
    interfaces = []  # linktypes

    while True:
        hdr = f.read(8)
        if len(hdr) < 8:
            break
        block_type, block_len = struct.unpack(f"{endian}II", hdr)
        if block_type == 0x0A0D0D0A:  # Section Header Block
            body = f.read(block_len - 8)
            byte_order_magic = struct.unpack("<I", body[:4])[0]
            if byte_order_magic == 0x1A2B3C4D:
                endian = "<"
            elif byte_order_magic == 0x4D3C2B1A:
                endian = ">"
            continue

        body = f.read(block_len - 8)
        if block_type == 0x00000001:  # Interface Description Block
            link_type = struct.unpack(f"{endian}H", body[:2])[0]
            interfaces.append(link_type)
        elif block_type == 0x00000006:  # Enhanced Packet Block
            if len(body) < 20:
                continue
            iface_id, ts_high, ts_low, cap_len, orig_len = struct.unpack(f"{endian}IIIII", body[:20])
            packet_data = body[20 : 20 + cap_len]
            linktype = interfaces[iface_id] if iface_id < len(interfaces) else LINKTYPE_USBPCAP
            ts = ((ts_high << 32) | ts_low) / 1_000_000.0  # default 1us
            pkt = parse_linktype_packet(linktype, packet_data, ts)
            if pkt:
                yield pkt


def parse_linktype_packet(linktype: int, raw: bytes, ts: float) -> Optional[UsbPacket]:
    if linktype == LINKTYPE_USBPCAP:
        # USBPcap packet header:
        # headerLen (uint16)
        # irpId (uint64)
        # usbd_status (uint32)
        # function (uint16)
        # info (uint8) -> bit 0: PDO to FDO (direction in = 1)
        # bus (uint16), device (uint16), endpoint (uint8)
        # transfer_type (uint8) -> 0: isoc, 1: int, 2: ctrl, 3: bulk
        # dataLength (uint32)
        if len(raw) < 27:
            return None
        header_len = struct.unpack("<H", raw[:2])[0]
        if len(raw) < header_len:
            return None
        info = raw[16]
        direction_in = bool(info & 0x01)
        transfer_type = raw[23]
        is_control = transfer_type == 2
        payload = raw[header_len:]

        bmRequestType, bRequest, wValue, wIndex, wLength = None, None, None, None, None
        if is_control and header_len >= 35:
            # Control transfer setup packet is located within header: offset 27..35
            setup = raw[27:35]
            if len(setup) == 8:
                bmRequestType, bRequest, wValue, wIndex, wLength = struct.unpack("<BBHHH", setup)

        return UsbPacket(
            timestamp_sec=ts,
            data=payload,
            direction_in=direction_in,
            is_control=is_control,
            bmRequestType=bmRequestType,
            bRequest=bRequest,
            wValue=wValue,
            wIndex=wIndex,
            wLength=wLength,
        )

    elif linktype in (LINKTYPE_USB_LINUX, LINKTYPE_USB_LINUX_MMAPPED):
        # Linux usbmon header: 48 or 64 bytes
        if len(raw) < 48:
            return None
        hdr_len = 64 if linktype == LINKTYPE_USB_LINUX_MMAPPED else 48
        direction_char = chr(raw[8])  # 'C' callback/in or 'S' submit
        transfer_type = raw[9]
        is_control = transfer_type == 2
        payload = raw[hdr_len:]
        direction_in = direction_char == "C"

        bmRequestType, bRequest, wValue, wIndex, wLength = None, None, None, None, None
        if is_control and len(raw) >= 48:
            setup = raw[40:48]
            bmRequestType, bRequest, wValue, wIndex, wLength = struct.unpack("<BBHHH", setup)

        return UsbPacket(
            timestamp_sec=ts,
            data=payload,
            direction_in=direction_in,
            is_control=is_control,
            bmRequestType=bmRequestType,
            bRequest=bRequest,
            wValue=wValue,
            wIndex=wIndex,
            wLength=wLength,
        )

    return None


def annotate_payload(data: bytes, is_control: bool, bRequest: Optional[int], wValue: Optional[int]) -> str:
    """Decodes RK855 payload into human-readable description."""
    if not data:
        if is_control and bRequest == 0x09:
            return "SET_REPORT Control Setup (empty payload or split stage)"
        return "Empty payload"

    # Normalize wire format:
    # On Windows/USBPcap, Feature Reports are often prefixed with Report ID (0x00) -> 65 bytes
    payload = data
    report_id = None
    if len(payload) == 65 and payload[0] == 0x00:
        report_id = payload[0]
        payload = payload[1:]

    hex_preview = payload[:16].hex(" ")
    if len(payload) > 16:
        hex_preview += f" ... ({len(payload)} bytes total)"

    lines = [f"Raw [{len(data)}B]: {hex_preview}"]

    # Check ISP trigger pattern
    if payload.startswith(ISP_TRIGGER_MAGIC):
        lines.append("  [!] ISP BOOTLOADER TRIGGER (AA 42 89 5A FF 71 62 CC)")
        return "\n".join(lines)

    # Check standard RK855 configurator commands: payload[0] == 0x04
    if len(payload) >= 2 and payload[0] == 0x04:
        cmd = payload[1]
        if cmd == 0x18:
            lines.append("  -> CMD 0x18: Begin Lighting Configuration")
        elif cmd == 0x11:
            mode_id = payload[2] if len(payload) > 2 else 0
            ctype = payload[3] if len(payload) > 3 else 0
            r = payload[4] if len(payload) > 4 else 0
            g = payload[5] if len(payload) > 5 else 0
            b = payload[6] if len(payload) > 6 else 0
            brightness = payload[7] if len(payload) > 7 else 0
            speed = payload[8] if len(payload) > 8 else 0
            direction = payload[9] if len(payload) > 9 else 0
            mode_name = LIGHT_MODES.get(mode_id, f"Unknown (0x{mode_id:02X})")
            ctype_name = COLOR_TYPES.get(ctype, f"0x{ctype:02X}")
            dir_name = DIRECTIONS.get(direction, f"0x{direction:02X}")
            lines.append(f"  -> CMD 0x11: Set Light Mode")
            lines.append(f"     Mode:       {mode_id} ({mode_name})")
            lines.append(f"     Color Type: {ctype_name}")
            lines.append(f"     RGB Color:  #{r:02X}{g:02X}{b:02X} (R:{r}, G:{g}, B:{b})")
            lines.append(f"     Brightness: {brightness}/4")
            lines.append(f"     Speed:      {speed}/4 (or raw 0x{speed:02X})")
            lines.append(f"     Direction:  {dir_name}")
        elif cmd == 0x13:
            block_idx = payload[2] if len(payload) > 2 else 0
            total_blocks = payload[8] if len(payload) > 8 else 0
            lines.append(f"  -> CMD 0x13: Custom Per-Key RGB Header (total_blocks_param={total_blocks})")
        elif cmd == 0x20:
            block_idx = payload[2] if len(payload) > 2 else 0
            block_count = payload[8] if len(payload) > 8 else 0
            lines.append(f"  -> CMD 0x20: Begin Key Remapping (block_count={block_count})")
        elif cmd == 0x19:
            lines.append("  -> CMD 0x19: Begin Macro Configuration")
        elif cmd == 0x15:
            lines.append("  -> CMD 0x15: Macro Data Block")
        elif cmd == 0x02:
            lines.append("  -> CMD 0x02: Commit Configuration to Flash Memory")
        elif cmd == 0xF0:
            lines.append("  -> CMD 0xF0: PWM Reload (Apply Lighting changes immediately)")
        elif cmd in (0xF1, 0xF2, 0xF3):
            profile_num = cmd - 0xF0
            lines.append(f"  -> CMD 0x{cmd:02X}: Switch Profile to Slot {profile_num}")
        else:
            lines.append(f"  -> Unknown Subcommand 0x{cmd:02X}")

    elif len(payload) >= 1 and payload[0] in (0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A):
        lines.append(f"  -> ISP Bootloader Cortex-M0 Opcode: 0x{payload[0]:02X}")
    elif len(payload) >= 1 and payload[0] in (0x01, 0x02, 0x03, 0x06, 0x07, 0x08):
        lines.append(f"  -> ISP Legacy / Bootloader Opcode: 0x{payload[0]:02X}")

    return "\n".join(lines)


def decode_file(pcap_path: Path, output_path: Optional[Path] = None) -> list[str]:
    """Decodes a single pcap/pcapng file and returns the annotated lines."""
    with open(pcap_path, "rb") as f:
        magic = f.read(4)
        f.seek(0)
        if magic == b"\n\r\r\n":
            pkts = list(parse_pcapng(f))
        else:
            pkts = list(parse_pcap(f))

    out_lines = []
    out_lines.append(f"=== RK855 Capture Decode: {pcap_path.name} ===")
    out_lines.append(f"Total USB Packets Extracted: {len(pkts)}")
    out_lines.append("-" * 60)

    count_hid = 0
    for idx, pkt in enumerate(pkts):
        # We care about HID Feature / SET_REPORT or payloads with RK855 signatures
        is_set_report = pkt.is_control and pkt.bRequest == 0x09
        is_get_report = pkt.is_control and pkt.bRequest == 0x01
        has_rk_payload = len(pkt.data) >= 2 and (
            pkt.data[0] == 0x04
            or (len(pkt.data) == 65 and pkt.data[1] == 0x04)
            or pkt.data.startswith(ISP_TRIGGER_MAGIC)
            or (len(pkt.data) == 65 and pkt.data[1:].startswith(ISP_TRIGGER_MAGIC))
        )

        if is_set_report or is_get_report or has_rk_payload or (pkt.is_control and pkt.data):
            count_hid += 1
            direction = "IN  (Dev->Host)" if pkt.direction_in else "OUT (Host->Dev)"
            ts_str = f"{pkt.timestamp_sec:.6f}s"
            req_str = ""
            if pkt.bRequest is not None:
                req_str = f" bRequest=0x{pkt.bRequest:02X} wValue=0x{pkt.wValue or 0:04X} wIndex=0x{pkt.wIndex or 0:04X}"

            out_lines.append(f"[{count_hid:04d}] TS: {ts_str} | {direction}{req_str}")
            annotated = annotate_payload(pkt.data, pkt.is_control, pkt.bRequest, pkt.wValue)
            for line in annotated.splitlines():
                out_lines.append(f"      {line}")
            out_lines.append("")

    if count_hid == 0:
        out_lines.append("No RK855 HID or ISP packets identified in trace.")

    result_text = "\n".join(out_lines) + "\n"

    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(result_text, encoding="utf-8")
        print(f"[+] Decoded trace written to: {output_path}")

    return out_lines


def main():
    parser = argparse.ArgumentParser(description="Decode USB HID captures of Royal Kludge RK855")
    parser.add_argument("input", type=Path, help="Input .pcap or .pcapng file or directory of captures")
    parser.add_argument("-o", "--output", type=Path, help="Output file or directory for decoded txt (default: captures/decoded/)")
    args = parser.parse_args()

    input_path = args.input
    if not input_path.exists():
        print(f"Error: input path '{input_path}' does not exist.", file=sys.stderr)
        sys.exit(1)

    default_out_dir = Path("captures/decoded")

    if input_path.is_dir():
        pcap_files = list(input_path.glob("*.pcap")) + list(input_path.glob("*.pcapng"))
        if not pcap_files:
            print(f"No .pcap or .pcapng files found in '{input_path}'.")
            return
        for pf in pcap_files:
            out_file = (args.output or default_out_dir) / f"{pf.stem}.txt"
            decode_file(pf, out_file)
    else:
        out_file = args.output
        if out_file is None:
            out_file = default_out_dir / f"{input_path.stem}.txt"
        decode_file(input_path, out_file)


if __name__ == "__main__":
    main()
