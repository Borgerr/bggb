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
                if self.header.rom_shift_count() > 4 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type().clone(),
                        reason: String::from("given ROM size is too large"),
                    });
                }
                // no RAM specified
                if self.header.ram_size() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type().clone(),
                        reason: String::from("header says RAM included with wrong cartridge type"),
                    });
                }

                self.rom = vec![0; 32768 * (1 << self.header.rom_shift_count())];
            }
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

            _ => {
                return Err(MemoryError::UnsupportedCartType {
                    ct: self.header.cartridge_type().clone(),
                })
            }
        }

        Ok(())
    }
}
