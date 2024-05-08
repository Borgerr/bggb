use std::ops::Index;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CartridgeType {
    ROM_ONLY = 0x00,

    MBC1 = 0x01,
    MBC1_RAM = 0x02,
    MBC1_RAM_BATTERY = 0x03,

    MBC2 = 0x05,
    MBC2_BATTERY = 0x06,

    ROM_RAM = 0x08,
    ROM_RAM_BATTERY = 0x09,

    MMM01 = 0x0b,
    MMM01_RAM = 0x0c,
    MMM01_RAM_BATTERY = 0x0d,

    MBC3_TIMER_BATTERY = 0x0f,
    MBC3_TIMER_RAM_BATTERY = 0x10,
    MBC3 = 0x11,
    MBC3_RAM = 0x12,
    MBC3_RAM_BATTERY = 0x13,

    MBC5 = 0x19,
    MBC5_RAM = 0x1a,
    MBC5_RAM_BATTERY = 0x1b,
    MBC5_RUMBLE = 0x1c,
    MBC5_RUMBLE_RAM = 0x1d,
    MBC5_RUMBLE_RAM_BATTERY = 0x1e,

    MBC6 = 0x20,

    MBC7_SENSOR_RUMBLE_RAM_BATTERY = 0x22,

    POCKET_CAMERA = 0xfc,
    BANDAI_TAMA5 = 0xfd,

    HuC3 = 0xfe,
    HuC1_RAM_BATTERY = 0xff,
}

impl CartridgeType {
    pub fn from_num(num: u8) -> CartridgeType {
        match num {
            0x01 => CartridgeType::MBC1,
            0x02 => CartridgeType::MBC1_RAM,
            0x03 => CartridgeType::MBC1_RAM_BATTERY,

            0x05 => CartridgeType::MBC2,
            0x06 => CartridgeType::MBC2_BATTERY,

            0x08 => CartridgeType::ROM_RAM,
            0x09 => CartridgeType::ROM_RAM_BATTERY,

            0x0b => CartridgeType::MMM01,
            0x0c => CartridgeType::MMM01_RAM,
            0x0d => CartridgeType::MMM01_RAM_BATTERY,

            0x0f => CartridgeType::MBC3_TIMER_BATTERY,
            0x10 => CartridgeType::MBC3_TIMER_RAM_BATTERY,
            0x11 => CartridgeType::MBC3,
            0x12 => CartridgeType::MBC3_RAM,
            0x13 => CartridgeType::MBC3_RAM_BATTERY,

            0x19 => CartridgeType::MBC5,
            0x1a => CartridgeType::MBC5_RAM,
            0x1b => CartridgeType::MBC5_RAM_BATTERY,
            0x1c => CartridgeType::MBC5_RUMBLE,
            0x1d => CartridgeType::MBC5_RUMBLE_RAM,
            0x1e => CartridgeType::MBC5_RUMBLE_RAM_BATTERY,

            0x20 => CartridgeType::MBC6,

            0x22 => CartridgeType::MBC7_SENSOR_RUMBLE_RAM_BATTERY,
            0xfc => CartridgeType::POCKET_CAMERA,
            0xfd => CartridgeType::BANDAI_TAMA5,

            0xfe => CartridgeType::HuC3,
            0xff => CartridgeType::HuC1_RAM_BATTERY,

            0 | _ => CartridgeType::ROM_ONLY,
        }
    }

    pub fn to_num(ty: CartridgeType) -> u8 {
        match ty {
            CartridgeType::ROM_ONLY => 0x00,

            CartridgeType::MBC1 => 0x01,
            CartridgeType::MBC1_RAM => 0x02,
            CartridgeType::MBC1_RAM_BATTERY => 0x03,

            CartridgeType::MBC2 => 0x05,
            CartridgeType::MBC2_BATTERY => 0x06,

            CartridgeType::ROM_RAM => 0x08,
            CartridgeType::ROM_RAM_BATTERY => 0x09,

            CartridgeType::MMM01 => 0x0b,
            CartridgeType::MMM01_RAM => 0x0c,
            CartridgeType::MMM01_RAM_BATTERY => 0x0d,

            CartridgeType::MBC3_TIMER_BATTERY => 0x0f,
            CartridgeType::MBC3_TIMER_RAM_BATTERY => 0x10,
            CartridgeType::MBC3 => 0x11,
            CartridgeType::MBC3_RAM => 0x12,
            CartridgeType::MBC3_RAM_BATTERY => 0x13,

            CartridgeType::MBC5 => 0x19,
            CartridgeType::MBC5_RAM => 0x1a,
            CartridgeType::MBC5_RAM_BATTERY => 0x1b,
            CartridgeType::MBC5_RUMBLE => 0x1c,
            CartridgeType::MBC5_RUMBLE_RAM => 0x1d,
            CartridgeType::MBC5_RUMBLE_RAM_BATTERY => 0x1e,

            CartridgeType::MBC6 => 0x20,

            CartridgeType::MBC7_SENSOR_RUMBLE_RAM_BATTERY => 0x22,
            CartridgeType::POCKET_CAMERA => 0xfc,
            CartridgeType::BANDAI_TAMA5 => 0xfd,

            CartridgeType::HuC3 => 0xfe,
            CartridgeType::HuC1_RAM_BATTERY => 0xff,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct CartridgeHeader {
    // https://gbdev.io/pandocs/The_Cartridge_Header.html
    entry_point: [u8; 4],    // 0100-0103, 4 bytes
    nintendo_logo: [u8; 48], // 0104-0133, 48 bytes
    title: [u8; 16],         // 0134-0143, 16 bytes
    // manufacturer code is addresses 0134-0143
    cgb_only: bool,                // single byte entry at end of title
    new_licensee: [u8; 2],         // 0144-0145, 2 bytes, can change to enum
    sgb_included: bool, // ignore any command packets if byte is set to val other than 0x03
    cartridge_type: CartridgeType, // 0147, single byte

    rom_shift_count: u8,      // 0148, single byte
    ram_size: u8,             // 0149, single byte
    destination_code: u8,     // 014a, single byte
    old_licensee: u8,         // 014b, single byte, can change to enum
    version_number: u8,       // 014c, single byte
    pub header_checksum: u8,  // 014d, single byte
    global_checksum: [u8; 2], // 014e-014f, two bytes
}

impl CartridgeHeader {
    pub fn new() -> CartridgeHeader {
        CartridgeHeader {
            entry_point: [0; 4],
            nintendo_logo: [0; 48],
            title: [0; 16],
            cgb_only: false,
            new_licensee: [0; 2],
            sgb_included: false,
            cartridge_type: CartridgeType::ROM_ONLY,

            rom_shift_count: 0,
            ram_size: 0,
            destination_code: 0,
            old_licensee: 0,
            version_number: 0,
            header_checksum: 0,
            global_checksum: [0; 2],
        }
    }

    pub fn read(&mut self, data: &[u8]) {
        for i in 0..4 {
            self.entry_point[i] = data[i];
        }

        for i in 0..48 {
            self.nintendo_logo[i] = data[i + 4];
        }

        for i in 0..16 {
            self.title[i] = data[i + 52];
        }

        self.cgb_only = self.title[15] == 0xc0;
        self.new_licensee[0] = data[68];
        self.new_licensee[1] = data[69];

        self.sgb_included = data[70] == 0x03;

        self.cartridge_type = CartridgeType::from_num(data[71]);

        self.rom_shift_count = data[72];
        self.ram_size = match data[73] {
            0x02 => 8,
            0x03 => 32,
            0x04 => 128,
            0x05 => 64,
            0 | _ => 0,
        };

        self.destination_code = data[74];
        self.old_licensee = data[75];
        self.version_number = data[76];
        self.header_checksum = data[77];

        self.global_checksum[0] = data[78];
        self.global_checksum[1] = data[79];
    }

    pub fn from(data: &[u8]) -> CartridgeHeader {
        let mut ch = CartridgeHeader::new();
        ch.read(data);
        ch
    }

    pub fn cartridge_type(&self) -> CartridgeType {
        self.cartridge_type.clone()
    }
    pub fn rom_shift_count(&self) -> u8 {
        self.rom_shift_count
    }
    pub fn ram_size(&self) -> u8 {
        self.ram_size
    }
    pub fn sgb_included(&self) -> bool {
        self.sgb_included
    }
    pub fn cgb_only(&self) -> bool {
        self.cgb_only
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cartridgeheader::{CartridgeHeader, CartridgeType};

    #[test]
    fn create_blank_header() {
        assert_eq!(
            CartridgeHeader {
                entry_point: [0; 4],
                nintendo_logo: [0; 48],
                title: [0; 16],
                cgb_only: false,
                new_licensee: [0; 2],
                sgb_included: false,
                cartridge_type: CartridgeType::ROM_ONLY,

                rom_shift_count: 0,
                ram_size: 0,
                destination_code: 0,
                old_licensee: 0,
                version_number: 0,
                header_checksum: 0,
                global_checksum: [0; 2],
            },
            CartridgeHeader::new()
        );
    }

    macro_rules! num_to_carttype {
        ($name:tt, $x:expr, $ct:expr) => {
            #[test]
            fn $name() {
                assert_eq!(CartridgeType::from_num($x), $ct);
            }
        };
    }
    num_to_carttype!(romonly_from_num, 0x00, CartridgeType::ROM_ONLY);

    num_to_carttype!(mbc1_from_num, 0x01, CartridgeType::MBC1);
    num_to_carttype!(mbc1ram_from_num, 0x02, CartridgeType::MBC1_RAM);
    num_to_carttype!(mbc1rb_from_num, 0x03, CartridgeType::MBC1_RAM_BATTERY);

    num_to_carttype!(mbc2_from_num, 0x05, CartridgeType::MBC2);
    num_to_carttype!(mbc2bat_from_num, 0x06, CartridgeType::MBC2_BATTERY);

    num_to_carttype!(romram_from_num, 0x08, CartridgeType::ROM_RAM);
    num_to_carttype!(romrb_from_num, 0x09, CartridgeType::ROM_RAM_BATTERY);

    num_to_carttype!(m01_from_num, 0x0b, CartridgeType::MMM01);
    num_to_carttype!(m01ram_from_num, 0x0c, CartridgeType::MMM01_RAM);
    num_to_carttype!(m01rb_from_num, 0x0d, CartridgeType::MMM01_RAM_BATTERY);

    num_to_carttype!(tb_from_num, 0x0f, CartridgeType::MBC3_TIMER_BATTERY);
    num_to_carttype!(trb_from_num, 0x10, CartridgeType::MBC3_TIMER_RAM_BATTERY);
    num_to_carttype!(mbc3_from_num, 0x11, CartridgeType::MBC3);
    num_to_carttype!(mbc3ram_from_num, 0x12, CartridgeType::MBC3_RAM);
    num_to_carttype!(mbc3rb_from_num, 0x13, CartridgeType::MBC3_RAM_BATTERY);

    num_to_carttype!(mbc5_from_num, 0x19, CartridgeType::MBC5);
    num_to_carttype!(mbc5ram_from_num, 0x1a, CartridgeType::MBC5_RAM);
    num_to_carttype!(mbc5rb_from_num, 0x1b, CartridgeType::MBC5_RAM_BATTERY);
    num_to_carttype!(mbc5rum_from_num, 0x1c, CartridgeType::MBC5_RUMBLE);
    num_to_carttype!(mbc5rr_from_num, 0x1d, CartridgeType::MBC5_RUMBLE_RAM);
    num_to_carttype!(
        mbc5rrb_from_num,
        0x1e,
        CartridgeType::MBC5_RUMBLE_RAM_BATTERY
    );

    num_to_carttype!(mbc6_from_num, 0x20, CartridgeType::MBC6);
    num_to_carttype!(
        mbc7_from_num,
        0x22,
        CartridgeType::MBC7_SENSOR_RUMBLE_RAM_BATTERY
    );

    num_to_carttype!(poccam_from_num, 0xfc, CartridgeType::POCKET_CAMERA);
    num_to_carttype!(bandai_from_num, 0xfd, CartridgeType::BANDAI_TAMA5);
    num_to_carttype!(huc3_from_num, 0xfe, CartridgeType::HuC3);
    num_to_carttype!(huc1_from_num, 0xff, CartridgeType::HuC1_RAM_BATTERY);

    // other way...
    macro_rules! carttype_to_num {
        ($name:tt, $x:expr, $ct:expr) => {
            #[test]
            fn $name() {
                assert_eq!($x, CartridgeType::to_num($ct));
            }
        };
    }
    carttype_to_num!(romonly_to_num, 0x00, CartridgeType::ROM_ONLY);

    carttype_to_num!(mbc1_to_num, 0x01, CartridgeType::MBC1);
    carttype_to_num!(mbc1ram_to_num, 0x02, CartridgeType::MBC1_RAM);
    carttype_to_num!(mbc1rb_to_num, 0x03, CartridgeType::MBC1_RAM_BATTERY);

    carttype_to_num!(mbc2_to_num, 0x05, CartridgeType::MBC2);
    carttype_to_num!(mbc2bat_to_num, 0x06, CartridgeType::MBC2_BATTERY);

    carttype_to_num!(romram_to_num, 0x08, CartridgeType::ROM_RAM);
    carttype_to_num!(romrb_to_num, 0x09, CartridgeType::ROM_RAM_BATTERY);

    carttype_to_num!(m01_to_num, 0x0b, CartridgeType::MMM01);
    carttype_to_num!(m01ram_to_num, 0x0c, CartridgeType::MMM01_RAM);
    carttype_to_num!(m01rb_to_num, 0x0d, CartridgeType::MMM01_RAM_BATTERY);

    carttype_to_num!(tb_to_num, 0x0f, CartridgeType::MBC3_TIMER_BATTERY);
    carttype_to_num!(trb_to_num, 0x10, CartridgeType::MBC3_TIMER_RAM_BATTERY);
    carttype_to_num!(mbc3_to_num, 0x11, CartridgeType::MBC3);
    carttype_to_num!(mbc3ram_to_num, 0x12, CartridgeType::MBC3_RAM);
    carttype_to_num!(mbc3rb_to_num, 0x13, CartridgeType::MBC3_RAM_BATTERY);

    carttype_to_num!(mbc5_to_num, 0x19, CartridgeType::MBC5);
    carttype_to_num!(mbc5ram_to_num, 0x1a, CartridgeType::MBC5_RAM);
    carttype_to_num!(mbc5rb_to_num, 0x1b, CartridgeType::MBC5_RAM_BATTERY);
    carttype_to_num!(mbc5rum_to_num, 0x1c, CartridgeType::MBC5_RUMBLE);
    carttype_to_num!(mbc5rr_to_num, 0x1d, CartridgeType::MBC5_RUMBLE_RAM);
    carttype_to_num!(mbc5rrb_to_num, 0x1e, CartridgeType::MBC5_RUMBLE_RAM_BATTERY);

    carttype_to_num!(mbc6_to_num, 0x20, CartridgeType::MBC6);
    carttype_to_num!(
        mbc7_to_num,
        0x22,
        CartridgeType::MBC7_SENSOR_RUMBLE_RAM_BATTERY
    );

    carttype_to_num!(poccam_to_num, 0xfc, CartridgeType::POCKET_CAMERA);
    carttype_to_num!(bandai_to_num, 0xfd, CartridgeType::BANDAI_TAMA5);
    carttype_to_num!(huc3_to_num, 0xfe, CartridgeType::HuC3);
    carttype_to_num!(huc1_to_num, 0xff, CartridgeType::HuC1_RAM_BATTERY);

    #[test]
    fn reads_valid_full_zeroes() {
        // test should complete without panicking
        let mut header = CartridgeHeader::new();
        header.read(&[0; 80]);
    }
    #[test]
    #[should_panic]
    fn reads_invalid_empty() {
        let mut header = CartridgeHeader::new();
        header.read(&[]);
    }

    macro_rules! ram_size_test {
        ($name:tt, $tag:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut read_arr = [0; 80];
                read_arr[73] = $tag;
                let header = CartridgeHeader::from(&read_arr);

                assert_eq!(header.ram_size, $expected);
            }
        };
    }
    ram_size_test!(ram_size_0kib, 0x00, 0);
    ram_size_test!(ram_size_8kib, 0x02, 8);
    ram_size_test!(ram_size_32kib, 0x03, 32);
    ram_size_test!(ram_size_128kib, 0x04, 128);
    ram_size_test!(ram_size_64kib, 0x05, 64);

    // last few of these are just for coverage,
    // likely no issues to arise but we can come back to it later
    #[test]
    fn blank_header_cartridge_fetch() {
        assert_eq!(
            CartridgeType::ROM_ONLY,
            (CartridgeHeader::new()).cartridge_type()
        );
    }
    #[test]
    fn blank_header_rsc_fetch() {
        assert_eq!(0, (CartridgeHeader::new()).rom_shift_count());
    }
    #[test]
    fn blank_header_ram_size_fetch() {
        assert_eq!(0, (CartridgeHeader::new()).ram_size());
    }
    #[test]
    fn blank_header_sgb_fetch() {
        assert_eq!(false, (CartridgeHeader::new()).sgb_included());
    }
    #[test]
    fn blank_header_cgb_fetch() {
        assert_eq!(false, (CartridgeHeader::new()).cgb_only());
    }
}
