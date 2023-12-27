use self::cartridgeheader::{CartridgeHeader, CartridgeType};

mod cartridgeheader;

#[derive(Debug, Clone)]
pub enum MemoryError {
    CartTypeMismatch { ct: CartridgeType, reason: String },
    UnsupportedCartType { ct: CartridgeType },
}

pub struct CartridgeData {
    header: CartridgeHeader,
    rom: Vec<u8>,
    switchable_banks: Vec<Vec<u8>>,
    ram: Vec<u8>,
}

impl CartridgeData {
    pub fn new() -> CartridgeData {
        CartridgeData {
            header: CartridgeHeader::new(),
            rom: vec![],
            switchable_banks: vec![],
            ram: vec![],
        }
    }

    pub fn read(&mut self, data: Vec<u8>) {}

    pub fn from(data: Vec<u8>) -> Result<CartridgeData, MemoryError> {
        let mut cd = Self::new();
        cd.header.read(&data[0x0100..0x014f]);

        // big meaty part of code put in a different function
        // for readability
        cd.organize_memory()?;

        Ok(cd)
    }

    fn organize_memory(&mut self) -> Result<(), MemoryError> {
        match self.header.cartridge_type() {
            CartridgeType::ROM_ONLY | CartridgeType::ROM_RAM | CartridgeType::ROM_RAM_BATTERY => {
                if self.header.rom_shift_count() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type().clone(),
                        reason: String::from("given ROM size is too large"),
                    });
                }

                // no MBC, no memory beyond 32 KiB. RAM addressed the same as ROM
                self.rom = vec![0; 32768];
            }

            CartridgeType::MBC1 => {
                // no RAM specified
                if self.header.ram_size() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type().clone(),
                        reason: String::from("header says RAM included with wrong cartridge type"),
                    });
                }

                match self.header.rom_shift_count() {
                    0x00 => {
                        // 32 KiB of ROM, two banks
                        self.rom = vec![0; 0x3fff]; // 0000-3fff, bank X0
                        self.switchable_banks.push(vec![0; 0x3fff]); // 4000-7fff, bank 01
                    }
                    0x01 => {
                        // 64 KiB of ROM, 4 banks
                        self.add_banks_for_mbc1(4);
                    }
                    0x02 => {
                        // 128 KiB of ROM, 8 banks
                        self.add_banks_for_mbc1(8);
                    }
                    0x03 => {
                        // 256 KiB of ROM, 16 banks
                        self.add_banks_for_mbc1(16);
                    }
                    0x04 => {
                        // 512 KiB of ROM, 32 banks
                        self.add_banks_for_mbc1(32);
                    }
                    // for these last two cases, i.e. 1MiB+,
                    // banks are different.
                    // TODO: look into how they are different,
                    // and organize accordingly
                    0x05 => {
                        // 1 MiB of ROM, 64 banks
                        todo!()
                    }
                    0x06 => {
                        // 2 MiB of ROM, 128 banks
                        todo!()
                    }
                    _ => {
                        return Err(MemoryError::CartTypeMismatch {
                            ct: self.header.cartridge_type().clone(),
                            reason: String::from("given ROM size is too large or incorrect"),
                        })
                    }
                }
            }
            /*
            Implement all of these once we've confirmed the above cases work.
            CartridgeType::MBC1_RAM => {}
            CartridgeType::MBC1_RAM_BATTERY => {}

            CartridgeType::MBC2 => {}
            CartridgeType::MBC2_BATTERY => {}

            CartridgeType::MBC3_TIMER_BATTERY => {}
            CartridgeType::MBC3_TIMER_RAM_BATTERY => {}
            CartridgeType::MBC3 => {}
            CartridgeType::MBC3_RAM => {}
            CartridgeType::MBC3_RAM_BATTERY => {}

            CartridgeType::MBC5 => {}
            CartridgeType::MBC5_RAM => {}
            CartridgeType::MBC5_RAM_BATTERY => {}
            CartridgeType::MBC5_RUMBLE => {}
            CartridgeType::MBC5_RUMBLE_RAM => {}
            CartridgeType::MBC5_RUMBLE_RAM_BATTERY => {}
            */
            _ => {
                return Err(MemoryError::UnsupportedCartType {
                    ct: self.header.cartridge_type().clone(),
                })
            }
        }

        Ok(())
    }

    fn add_banks_for_mbc1(&mut self, bank_count: u16) {
        self.rom = vec![0; 0x3fff]; // covers first bank
        for _ in 0..bank_count - 1 {
            self.switchable_banks.push(vec![0; 0x3fff]);
        }
    }
}
