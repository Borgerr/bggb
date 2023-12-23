enum CartridgeType {
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
}

struct CartridgeHeader {
    // https://gbdev.io/pandocs/The_Cartridge_Header.html
    entry_point: [u8; 4],    // 0100-0103, 4 bytes
    nintendo_logo: [u8; 48], // 0104-0133, 48 bytes
    title: [u8; 16],         // 0134-0143, 16 bytes
    // manufacturer code is addresses 0134-0143
    cgb_only: bool,                // single byte entry at end of title
    new_licensee: u16,             // 0144-0145, 2 bytes, can change to enum
    sgb_included: bool, // ignore any command packets if byte is set to val other than 0x03
    cartridge_type: CartridgeType, // 0147, single byte

    rom_shift_count: u8,  // 0148, single byte
    ram_size: u8,         // 0149, single byte
    destination_code: u8, // 014a, single byte
    old_licensee: u8,     // 014b, single byte, can change to enum
    version_number: u8,   // 014c, single byte
    header_checksum: u8,  // 014d, single byte
    global_checksum: u16, // 014e-014f, two bytes
}

impl CartridgeHeader {
    pub fn new() -> CartridgeHeader {
        CartridgeHeader {
            entry_point: [0; 4],
            nintendo_logo: [0; 48],
            title: [0; 16],
            cgb_only: false,
            new_licensee: 0,
            sgb_included: false,
            cartridge_type: CartridgeType::ROM_ONLY,
            rom_shift_count: 0,
            ram_size: 0,
            destination_code: 0,
            old_licensee: 0,
            version_number: 0,
            header_checksum: 0,
            global_checksum: 0,
        }
    }

    pub fn read(&mut self, data: &[u8; 0x014f - 0x0100]) {
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
        self.new_licensee = ((data[68] as u16) << 8) | (data[69] as u16);

        self.sgb_included = data[70] == 0x03;

        self.cartridge_type = CartridgeType::from_num(data[71]);

        self.rom_shift_count = data[72];
        self.ram_size = match data[73] {
            0x02 => 8,
            0x03 => 32,
            0x04 => 128,
            0x05 => 64,
            0 | _ => 0,
        }

        self.destination_code = data[74];
        self.old_licensee = data[75];
        self.version_number = data[76];
        self.header_checksum = data[77];

        self.global_checksum = ((data[78] as u16) << 8) | (data[79] as u16);
    }

    pub fn from(data: &[u8; 0x014f - 0x0100]) -> CartridgeHeader {
        let mut ch = CartridgeHeader::new();
        ch.read(data);
        ch
    }
}
