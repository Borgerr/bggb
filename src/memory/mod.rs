use std::ops::{Index, IndexMut};

use self::cartridgeheader::{CartridgeHeader, CartridgeType};

mod cartridgeheader;

#[derive(Debug, Clone)]
pub enum MemoryError {
    CartTypeMismatch { ct: CartridgeType, reason: String },
    UnsupportedCartType { ct: CartridgeType },
}

pub struct Memory {
    header: CartridgeHeader,
    rom: Vec<u8>,
    switchable_banks: Vec<Vec<u8>>,
    ram: Vec<u8>,
    vram: Vec<u8>,
    wram1: Vec<u8>,
    wram2: Vec<u8>,
    oam: Vec<u8>,
    io_registers: Vec<u8>,
    hram: Vec<u8>,
    interrupt_enable_reg: u8,
}

impl Memory {
    // Other goals:
    // add rest of MBC1 and MBC2/3/5

    pub fn new() -> Memory {
        Memory {
            header: CartridgeHeader::new(), // always addresses $0100 - $014F
            rom: Vec::new(),
            switchable_banks: Vec::new(),
            ram: Vec::new(),
            vram: Vec::new(),
            wram1: Vec::new(),
            wram2: Vec::new(),
            oam: Vec::new(),
            io_registers: Vec::new(),
            hram: Vec::new(),
            interrupt_enable_reg: 0,
        }
    }

    pub fn read(&mut self, data: Vec<u8>) -> Result<(), MemoryError> {
        self.header.read(&data[0x0100..0x014f + 1]);

        // big meaty part of code put in a different function for readability
        self.organize_memory()?;

        for i in 0..data.len() {
            self[i] = data[i];
        }

        Ok(())
    }

    pub fn from(data: Vec<u8>) -> Result<Memory, MemoryError> {
        let mut cd = Self::new();
        cd.read(data)?;

        Ok(cd)
    }

    fn organize_memory(&mut self) -> Result<(), MemoryError> {
        // organize memory that's always involved
        self.vram = vec![0; 0x2000]; // 8KiB of VRAM
        self.wram1 = vec![0; 0x1000]; // 4KiB of WRAM (bank 1)
                                      // echo RAM points here
        self.wram2 = vec![0; 0x1000]; // 4KiB of WRAM (bank 2)
        self.oam = vec![0; 0x00a0];
        self.io_registers = vec![0; 0x0080];
        self.hram = vec![0; 0xffff - 0xff80];

        match self.header.cartridge_type() {
            CartridgeType::ROM_ONLY | CartridgeType::ROM_RAM | CartridgeType::ROM_RAM_BATTERY => {
                if self.header.rom_shift_count() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type().clone(),
                        reason: String::from("given ROM size is too large"),
                    });
                }

                // no MBC, no memory beyond 32 KiB. RAM addressed the same as ROM
                self.rom = vec![0; 0x4000];
                self.switchable_banks.push(vec![0; 0x4000]);
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
                        self.rom = vec![0; 0x4000]; // 0000-3fff, bank X0
                        self.switchable_banks.push(vec![0; 0x4000]); // 4000-7fff, bank 01
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

impl Index<usize> for Memory {
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

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.header.cartridge_type() {
            CartridgeType::ROM_ONLY | CartridgeType::ROM_RAM | CartridgeType::ROM_RAM_BATTERY => {
                if index <= 0x3fff {
                    // rom, includes header
                    return &mut self.rom[index];
                } else if (index > 0x3fff) && (index <= 0x7fff) {
                    return &mut self.switchable_banks[0][index - 0x3fff];
                } else {
                    // ram
                    return &mut self.ram[index - 0x0a000];
                }
            }

            CartridgeType::MBC1 => {
                if index <= 0x3fff {
                    // rom bank 1, includes header
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
mod tests {
    use crate::memory::cartridgeheader::CartridgeHeader;

    use super::Memory;

    #[test]
    fn new_blank_data() {
        let cd = Memory::new();
        assert_eq!(cd.header, CartridgeHeader::new());
        assert_eq!(cd.rom, Vec::<u8>::new());
        assert_eq!(cd.switchable_banks, Vec::<Vec<u8>>::new());
        assert_eq!(cd.ram, Vec::<u8>::new());
    }

    #[test]
    #[should_panic]
    fn reading_blank_invalid() {
        let mut cd = Memory::new();
        let _ = cd.read(Vec::new());
    }
    #[test]
    fn reading_16kib_zerovec_valid() {
        let result = Memory::from(vec![0; 0x3fff]);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_32kib_zerovec_valid() {
        let result = Memory::from(vec![0; 0x7fff]);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_zerovec_romram_valid() {
        let mut rom = vec![0; 0xbfff];
        rom[0x0100 + 71] = 0x08; // cartridge type includes ram
        rom[0x0100 + 73] = 0x02; // ram size is 8KiB

        let result = Memory::from(rom);
        assert!(if let Ok(_) = result { true } else { false });
    }
}
