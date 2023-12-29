use std::ops::{Index, IndexMut};

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
    // Other goals:
    // add rest of MBC1 and MBC2/3/5

    pub fn new() -> CartridgeData {
        CartridgeData {
            header: CartridgeHeader::new(), // always addresses $0100 - $014F
            rom: Vec::new(),
            switchable_banks: Vec::new(),
            ram: Vec::new(),
        }
    }

    pub fn read(&mut self, data: Vec<u8>) {}

    pub fn from(data: Vec<u8>) -> Result<CartridgeData, MemoryError> {
        let mut cd = Self::new();
        cd.header.read(&data[0x0100..0x014f]);

        // big meaty part of code put in a different function for readability
        cd.organize_memory()?;

        for i in 0..cd.rom.len() {
            cd.rom[i] = data[i];
        }

        let mut i = cd.rom.len();
        let mut bank_number = 1;
        for bank in &mut cd.switchable_banks {
            for _ in 0..cd.rom.len() {
                bank[i - (bank_number * cd.rom.len())] = data[i];
                i += 1;
            }
            bank_number += 1;
        }

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

impl Index<usize> for CartridgeData {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        match self.header.cartridge_type() {
            CartridgeType::ROM_ONLY | CartridgeType::ROM_RAM | CartridgeType::ROM_RAM_BATTERY => {
                if (index >= 0x0000) && (index <= 0x7fff) {
                    // rom; includes header
                    return &self.rom[index];
                } else {
                    // ram
                    return &self.ram[index - 0x0a000];
                }
            }

            CartridgeType::MBC1 => {
                if index <= 0x3fff {
                    // rom bank 1
                    return &self.rom[index];
                } else {
                    // switchable rom bank
                    return &self.switchable_banks[0][index - 0x4000];
                }
            }
            _ => &0,
        }
    }
}

impl IndexMut<usize> for CartridgeData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.header.cartridge_type() {
            CartridgeType::ROM_ONLY | CartridgeType::ROM_RAM | CartridgeType::ROM_RAM_BATTERY => {
                if (index <= 0x014f) && (index >= 0x0100) {
                    // header
                    todo!();
                } else if (index >= 0x0000) && (index <= 0x7fff) {
                    // rom
                    return &mut self.rom[index];
                } else {
                    // ram
                    return &mut self.ram[index - 0x0a000];
                }
            }

            CartridgeType::MBC1 => {
                if (index <= 0x01f) && (index >= 0x0100) {
                    // header
                    todo!();
                } else if index <= 0x3fff {
                    // rom bank 1
                    return &mut self.rom[index];
                } else {
                    // switchable rom bank
                    return &mut self.switchable_banks[0][index - 0x4000];
                }
            }
            _ => &mut self.rom[0], // should this be allowed?
        }
    }
}

#[cfg(test)]
mod tests {}
