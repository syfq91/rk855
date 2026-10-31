//! Pure, no-I/O protocol primitives, constants, packet builders, parsers,
//! layout tables, and transaction generators for Royal Kludge RK855 (SONiX SN32F240B).

#![no_std]

extern crate alloc;

pub mod constants;
pub mod enums;
pub mod keys;
pub mod lighting;
pub mod packet;
pub mod remap;

// Re-exports for convenient top-level access
pub use constants::*;
pub use enums::*;
pub use keys::{
    find_key_by_index, find_key_by_name, find_key_by_scancode, PhysicalKey, PHYSICAL_KEYS,
};
pub use lighting::{
    build_active_mode_block, build_begin_light, build_custom_rgb_header,
    build_lighting_transaction, build_set_light, LightParams,
};
pub use packet::{
    build_begin_macro, build_commit, build_isp_trigger, build_macro_data, build_pwm_reload,
    build_read_back_request, build_switch_profile, from_wire_report, parse_packet, to_wire_report,
    Packet, ParsedPacket, ProtoError, WireReport,
};
pub use remap::{
    build_begin_remap, decode_remap_table, encode_remap_table, split_remap_table_blocks, RemapEntry,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_vector_test1_transaction() {
        let params = LightParams {
            mode: LightMode::ConstantRainbow, // 0x10 (Explode in vendor software)
            color_type: ColorType::RgbCustom, // 0x01
            r: 255,
            g: 0,
            b: 255,
            brightness: 15, // 0x0F
            speed: 10,      // 0x0A
            direction: Direction::LeftToRight,
        };

        let tx = build_lighting_transaction(&params);
        assert_eq!(tx.len(), 22, "Transaction must contain exactly 22 packets");

        // Packet 1: 04 18 (Begin Lighting Configuration)
        assert_eq!(tx[0][0], 0x04);
        assert_eq!(tx[0][1], 0x18);

        // Packet 2: 04 13 ... 0x12 (Header with 18 data blocks)
        assert_eq!(tx[1][0], 0x04);
        assert_eq!(tx[1][1], 0x13);
        assert_eq!(tx[1][8], 0x12);

        // Packet 3: Block 1 (Preset Slot 1)
        assert_eq!(tx[2][0], 0x01);
        assert_eq!(tx[2][1], 0xFF);
        assert_eq!(tx[2][2], 0xE1);
        assert_eq!(&tx[2][14..16], &[0xAA, 0x55]);

        // Packets 8..10: Zero-padding blocks
        assert_eq!(&tx[7], &[0u8; 64]);
        assert_eq!(&tx[8], &[0u8; 64]);
        assert_eq!(&tx[9], &[0u8; 64]);

        // Packets 11..19: Key records with 0x80 status
        assert_eq!(tx[10][0], 0x80);
        assert_eq!(tx[18][0], 0x80);

        // Packet 20: Block 18 (Active Mode: 0x10, Brightness 15=0x0F, Speed 10=0x0A)
        assert_eq!(tx[19][0], 0x10);
        assert_eq!(tx[19][1], 0xFF);
        assert_eq!(tx[19][2], 0xE1);
        assert_eq!(tx[19][8], 0x01);
        assert_eq!(tx[19][9], 0x0F); // Brightness 15
        assert_eq!(tx[19][10], 0x0A); // Speed 10
        assert_eq!(&tx[19][14..16], &[0xAA, 0x55]);

        // Packet 21: 04 02 (Commit)
        assert_eq!(tx[20][0], 0x04);
        assert_eq!(tx[20][1], 0x02);

        // Packet 22: 04 F0 (PWM Reload)
        assert_eq!(tx[21][0], 0x04);
        assert_eq!(tx[21][1], 0xF0);
    }

    #[test]
    fn test_golden_commit_ack_parsing() {
        // Response captured in test_1.pcap packet [0083]: 04 02 00 01 d8 85 00 ...
        let mut ack_packet = [0u8; 64];
        ack_packet[0] = 0x04;
        ack_packet[1] = 0x02;
        ack_packet[2] = 0x00;
        ack_packet[3] = 0x01; // status OK
        ack_packet[4] = 0xD8; // flash address low byte
        ack_packet[5] = 0x85; // flash address high byte (0x85D8)

        let parsed = parse_packet(&ack_packet).expect("Failed to parse commit ACK");
        match parsed {
            ParsedPacket::CommitAck {
                status,
                flash_address,
            } => {
                assert_eq!(status, 1);
                assert_eq!(flash_address, 0x85D8);
            }
            other => panic!("Unexpected parsed packet: {:?}", other),
        }
    }

    #[test]
    fn test_key_lookup() {
        let esc = find_key_by_name("esc").expect("esc key not found");
        assert_eq!(esc.scancode, 0x29);
        assert_eq!(esc.index, 0);

        let caps = find_key_by_name("caps").expect("caps key not found");
        assert_eq!(caps.scancode, 0x39);
        assert_eq!(caps.index, 30);

        let by_code = find_key_by_scancode(0x14).expect("q key not found by scancode");
        assert_eq!(by_code.name, "q");
    }

    #[test]
    fn test_wire_conversion() {
        let pkt = build_commit();
        let wire = to_wire_report(&pkt);
        assert_eq!(wire[0], 0x00);
        assert_eq!(wire[1], 0x04);
        assert_eq!(wire[2], 0x02);

        let extracted = from_wire_report(&wire).expect("Failed to convert from wire");
        assert_eq!(extracted, pkt);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use alloc::vec;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_wire_report_roundtrip(bytes in proptest::collection::vec(any::<u8>(), 64)) {
            let mut pkt = [0u8; 64];
            pkt.copy_from_slice(&bytes);
            let wire = to_wire_report(&pkt);
            let restored = from_wire_report(&wire).unwrap();
            prop_assert_eq!(pkt, restored);
        }

        #[test]
        fn prop_remap_table_roundtrip(
            entries in proptest::collection::vec(
                (1u8..=255, 1u8..=255).prop_map(|(p, t)| RemapEntry::new(p, t)),
                0..=68
            )
        ) {
            let table = encode_remap_table(&entries);
            let decoded = decode_remap_table(&table);
            prop_assert_eq!(entries, decoded);
        }

        #[test]
        fn prop_set_light_packet_roundtrip(
            mode_raw in 0u8..=19,
            color_type_raw in proptest::sample::select(vec![0x00u8, 0x01, 0x07, 0x08]),
            r in any::<u8>(),
            g in any::<u8>(),
            b in any::<u8>(),
            brightness in 0u8..=15,
            speed in any::<u8>(),
            direction_raw in 0u8..=1,
        ) {
            let mode = LightMode::from_u8(mode_raw).unwrap();
            let color_type = ColorType::from_u8(color_type_raw).unwrap();
            let direction = Direction::from_u8(direction_raw).unwrap();

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

            let pkt = build_set_light(&params);
            let parsed = parse_packet(&pkt).unwrap();

            match parsed {
                ParsedPacket::SetLight(parsed_params) => {
                    prop_assert_eq!(params, parsed_params);
                }
                other => prop_assert!(false, "Expected SetLight, got {:?}", other),
            }
        }
    }
}
