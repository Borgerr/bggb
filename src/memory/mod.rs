use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use thiserror::Error;

use self::cartridgeheader::{CartridgeHeader, CartridgeType};

mod cartridgeheader;

#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum MemoryError {
    CartTypeMismatch { ct: CartridgeType, reason: String },
    UnsupportedCartType { ct: CartridgeType },
}

impl Display for MemoryError {
    // TODO: REMEMBER TO IMPLEMENT THIS IF WE END UP USING IT!!!
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CartTypeMismatch { ct, reason } => todo!(),
            Self::UnsupportedCartType { ct } => todo!(),
        }
    }
}

pub struct Memory {
    pub header: CartridgeHeader,
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

        self.put_into_banks(data);

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
            CartridgeType::ROM_ONLY => {
                if self.header.rom_shift_count() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type(),
                        reason: String::from("given ROM size is too large"),
                    });
                }

                // no RAM specified
                if self.header.ram_size() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type(),
                        reason: String::from("header says RAM included with wrong cartridge type"),
                    });
                }

                self.rom = vec![0; 0x4000];
                self.switchable_banks.push(vec![0; 0x4000]);
            }

            CartridgeType::ROM_RAM | CartridgeType::ROM_RAM_BATTERY => {
                if self.header.rom_shift_count() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type(),
                        reason: String::from("given ROM size is too large"),
                    });
                }
                self.rom = vec![0; 0x4000];
                self.switchable_banks.push(vec![0; 0x4000]);
                self.ram = vec![0; 0x2000];
            }

            CartridgeType::MBC1 => {
                // no RAM specified
                if self.header.ram_size() > 0 {
                    return Err(MemoryError::CartTypeMismatch {
                        ct: self.header.cartridge_type(),
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
                            ct: self.header.cartridge_type(),
                            reason: String::from("given ROM size is too large or incorrect"),
                        })
                    }
                }
            }
            _ => {
                return Err(MemoryError::UnsupportedCartType {
                    ct: self.header.cartridge_type(),
                })
            }
        }

        Ok(())
    }

    fn add_banks_for_mbc1(&mut self, bank_count: u16) {
        self.rom = vec![0; 0x4000]; // covers first bank
        for _ in 0..bank_count - 1 {
            self.switchable_banks.push(vec![0; 0x4000]);
        }
    }

    fn put_into_banks(&mut self, data: Vec<u8>) {
        // write in bank 00 first
        let mut i = 0;
        while (i < self.rom.len()) && (i < data.len()) {
            self.rom[i] = data[i];
            i += 1;
        }
        // write in rest of memory banks
        while i < data.len() {
            for bank in &mut self.switchable_banks {
                for j in 0..bank.len() {
                    bank[j] = data[i];
                    i += 1;
                }
            }
        }
    }
}

// might consider adding different functionality depending on MBC
// for indexing implementations

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index <= 0x3fff {
            // rom bank 00, includes header
            return &self.rom[index];
        } else if (index >= 0x4000) && (index <= 0x7fff) {
            // rom bank 01
            return &self.switchable_banks[0][index - 0x4000];
        } else if (index >= 0x8000) && (index <= 0x9fff) {
            // VRAM
            return &self.vram[index - 0x8000];
        } else if (index >= 0xa000) && (index <= 0xbfff) {
            // external ram if any
            return &self.ram[index - 0xa000];
        } else if (index >= 0xc000) && (index <= 0xcfff) {
            // WRAM bank 1
            return &self.wram1[index - 0xc000];
        } else if (index >= 0xd000) && (index <= 0xdfff) {
            // WRAM bank 2
            return &self.wram2[index - 0xd000];
        } else if (index >= 0xe000) && (index <= 0xfdff) {
            // mirror of C000~DDFF
            return &self[(index - 0xe000) + 0xc000];
        } else if (index >= 0xfe00) && (index <= 0xfe9f) {
            // OAM
            return &self.oam[index - 0xfe00];
        } else if (index >= 0xff00) && (index <= 0xff7f) {
            // I/O registers
            return &self.io_registers[index - 0xff00];
        } else if (index >= 0xff80) && (index <= 0xfffe) {
            // HRAM
            return &self.hram[index - 0xff80];
        } else if index == 0xffff {
            // interrupt enable register
            return &self.interrupt_enable_reg;
        } else {
            return &0;
        }
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index <= 0x3fff {
            // rom bank 00, includes header
            return &mut self.rom[index];
        } else if (index >= 0x4000) && (index <= 0x7fff) {
            // rom bank 01
            return &mut self.switchable_banks[0][index - 0x4000];
        } else if (index >= 0x8000) && (index <= 0x9fff) {
            // VRAM
            return &mut self.vram[index - 0x8000];
        } else if (index >= 0xa000) && (index <= 0xbfff) {
            // external ram if any
            return &mut self.ram[index - 0xa000];
        } else if (index >= 0xc000) && (index <= 0xcfff) {
            // WRAM bank 1
            return &mut self.wram1[index - 0xc000];
        } else if (index >= 0xd000) && (index <= 0xdfff) {
            // WRAM bank 2
            return &mut self.wram2[index - 0xd000];
        } else if (index >= 0xe000) && (index <= 0xfdff) {
            // mirror of C000~DDFF
            return &mut self[(index - 0xe000) + 0xc000];
        } else if (index >= 0xfe00) && (index <= 0xfe9f) {
            // OAM
            return &mut self.oam[index - 0xfe00];
        } else if (index >= 0xff00) && (index <= 0xff7f) {
            // I/O registers
            return &mut self.io_registers[index - 0xff00];
        } else if (index >= 0xff80) && (index <= 0xfffe) {
            // HRAM
            return &mut self.hram[index - 0xff80];
        } else {
            // interrupt enable register
            return &mut self.interrupt_enable_reg;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{
        cartridgeheader::{CartridgeHeader, CartridgeType},
        MemoryError,
    };

    use super::Memory;

    #[test]
    fn new_blank_data() {
        let cd = Memory::new();
        assert_eq!(cd.header, CartridgeHeader::new());
        assert_eq!(cd.rom, Vec::<u8>::new());
        assert_eq!(cd.switchable_banks, Vec::<Vec<u8>>::new());
        assert_eq!(cd.ram, Vec::<u8>::new());
    }

    /*
       reading tests
    */

    #[test]
    #[should_panic]
    fn reading_blank_invalid() {
        let mut cd = Memory::new();
        let _ = cd.read(Vec::new());
    }
    #[test]
    fn reading_16kib_zerovec_valid() {
        let result = Memory::from(vec![0; 0x4000]);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_32kib_zerovec_valid() {
        let result = Memory::from(vec![0; 0x8000]);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_zerovec_romram_valid() {
        let mut rom = vec![0; 0x8000];
        rom[0x0100 + 71] = 0x08; // cartridge type includes ram
        rom[0x0100 + 73] = 0x02; // ram size is 8KiB

        let result = Memory::from(rom);
        assert!(if let Ok(_) = result { true } else { false });
    }

    #[test]
    fn reading_16kib_zerovec_ram_romonly_invalid() {
        let mut rom = vec![0; 0x4000];
        rom[0x0100 + 71] = 0x00; // cartridge type does not include ram
        rom[0x0100 + 73] = 0x02; // ram size is 8KiB; too large for cartridge type!

        let result = Memory::from(rom);

        if let Err(e) = result {
            assert_eq!(
                e,
                MemoryError::CartTypeMismatch {
                    ct: CartridgeType::ROM_ONLY,
                    reason: String::from("header says RAM included with wrong cartridge type"),
                }
            )
        } else {
            // not an error, fail this test
            assert!(false);
        }
    }
    #[test]
    fn reading_32kib_zerovec_ram_romonly_invalid() {
        let mut rom = vec![0; 0x8000];
        rom[0x0100 + 71] = 0x00; // cartridge type does not include ram
        rom[0x0100 + 73] = 0x02; // ram size is 8KiB; too large for cartridge type!

        let result = Memory::from(rom);

        if let Err(e) = result {
            assert_eq!(
                e,
                MemoryError::CartTypeMismatch {
                    ct: CartridgeType::ROM_ONLY,
                    reason: String::from("header says RAM included with wrong cartridge type"),
                }
            )
        } else {
            // not an error, fail this test
            assert!(false);
        }
    }
    #[test]
    fn reading_zerovec_romonly_invalid_shiftcount() {
        for i in 2..6 {
            let mut rom = vec![0; 0x4000];
            rom[0x0100 + 71] = 0x00; // cartridge type is ROM_ONLY

            rom[0x0100 + 72] = i;
            let result = Memory::from(rom);

            if let Err(e) = result {
                assert_eq!(
                    e,
                    MemoryError::CartTypeMismatch {
                        ct: CartridgeType::ROM_ONLY,
                        reason: String::from("given ROM size is too large"),
                    }
                )
            } else {
                // not an error, fail this test
                assert!(false);
            }
        }
    }
    #[test]
    fn reading_zerovec_romram_invalid_shiftcount() {
        for i in 2..6 {
            let mut rom = vec![0; 0x4000];
            rom[0x0100 + 71] = 0x08; // cartridge type is ROM_RAM

            rom[0x0100 + 72] = i;
            let result = Memory::from(rom);

            if let Err(e) = result {
                assert_eq!(
                    e,
                    MemoryError::CartTypeMismatch {
                        ct: CartridgeType::ROM_RAM,
                        reason: String::from("given ROM size is too large"),
                    }
                )
            } else {
                // not an error, fail this test
                assert!(false);
            }
        }
    }

    #[test]
    fn reading_32kib_zerovec_mbc1_valid() {
        let mut rom = vec![0; 0x8000];
        rom[0x0100 + 71] = 0x01; // cartridge type is MBC1
        rom[0x0100 + 73] = 0x00; // no ram

        let result = Memory::from(rom);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_64kib_zerovec_mbc1_valid() {
        let mut rom = vec![0; 0x10000];
        rom[0x0147] = 0x01; // cartridge type is MBC1
        rom[0x0148] = 0x01; // ROM size is 64KiB
        rom[0x0149] = 0x00; // no ram

        let result = Memory::from(rom);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_128kib_zerovec_mbc1_valid() {
        let mut rom = vec![0; 0x20000];
        rom[0x0147] = 0x01; // cartridge type is MBC1
        rom[0x0148] = 0x02; // ROM size is 128KiB
        rom[0x0149] = 0x00; // no ram

        let result = Memory::from(rom);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_256kib_zerovec_mbc1_valid() {
        let mut rom = vec![0; 0x40000];
        rom[0x0147] = 0x01; // cartridge type is MBC1
        rom[0x0148] = 0x03; // ROM size is 256KiB
        rom[0x0149] = 0x00; // no ram

        let result = Memory::from(rom);
        assert!(if let Ok(_) = result { true } else { false });
    }
    #[test]
    fn reading_512kib_zerovec_mbc1_valid() {
        let mut rom = vec![0; 0x80000];
        rom[0x0147] = 0x01; // cartridge type is MBC1
        rom[0x0148] = 0x04; // ROM size is 512KiB
        rom[0x0149] = 0x00; // no ram

        let result = Memory::from(rom);
        assert!(if let Ok(_) = result { true } else { false });
    }
    // TODO: add 1MiB+ tests whenever we introduce that in implementation

    #[test]
    fn reading_zerovec_mbc1_with_ram_invalid() {
        let mut rom = vec![0; 0x8000];
        rom[0x0100 + 71] = 0x01; // cartridge type is MBC1
        rom[0x0100 + 73] = 0x02; // ram size is 8KiB, too large for cartridge type!

        let result = Memory::from(rom);
        assert!(if let Err(_) = result { true } else { false });
    }

    #[test]
    fn reading_zerovec_mbc1_invalid_shiftcount() {
        let mut rom = vec![0; 4000];
        rom[0x0100 + 71] = 0x01; // cartridge type is MBC1
        rom[0x0100 + 72] = 0x42; // invalid shift count

        let result = Memory::from(rom);
        if let Err(e) = result {
            assert_eq!(
                e,
                MemoryError::CartTypeMismatch {
                    ct: CartridgeType::MBC1,
                    reason: String::from("given ROM size is too large or incorrect"),
                }
            )
        } else {
            // not an error, fail this test
            assert!(false);
        }
    }

    #[test]
    fn error_on_unsupported_carttype() {
        // hopefully someday this test will be unnecessary.
        let mut rom = vec![0; 4000];
        rom[0x0100 + 71] = 0xfc; // pocket camera

        let result = Memory::from(rom);
        if let Err(e) = result {
            assert_eq!(
                e,
                MemoryError::UnsupportedCartType {
                    ct: CartridgeType::POCKET_CAMERA
                }
            );
        } else {
            // not an error, fail this test
            assert!(false);
        }
    }

    /*
       indexing tests
    */

    macro_rules! set_and_check_index_test {
        ($name:tt, $x:expr, $loc:expr) => {
            #[test]
            fn $name() {
                let mut rom = vec![0; 4000];
                rom[0x0100 + 71] = 0x08; // cartridge type includes ram
                rom[0x0100 + 73] = 0x02; // ram size is 8KiB

                let mut mem = Memory::from(rom).unwrap();

                mem[$loc] = $x;
                assert_eq!(mem[$loc], $x);
            }
        };
    }
    set_and_check_index_test!(set_and_check_rombank00, 0x42, 0x0200);
    set_and_check_index_test!(set_and_check_rombank01, 0x42, 0x4100);
    set_and_check_index_test!(set_and_check_vram, 0x42, 0x8100);
    set_and_check_index_test!(set_and_check_ram, 0x42, 0xa100);
    set_and_check_index_test!(set_and_check_wram1, 0x42, 0xc100);
    set_and_check_index_test!(set_and_check_wram2, 0x42, 0xd100);
    set_and_check_index_test!(set_and_check_mirror, 0x42, 0xe100);
    set_and_check_index_test!(set_and_check_oam, 0x42, 0xfe42);
    set_and_check_index_test!(set_and_check_ioreg, 0x42, 0xff42);
    set_and_check_index_test!(set_and_check_hram, 0x42, 0xff90);
    set_and_check_index_test!(set_and_check_interreg, 0x42, 0xffff);

    #[test]
    fn out_of_bounds_index_returns_zero() {
        let mem = Memory::from(vec![0; 4000]).unwrap();

        assert_eq!(mem[0x10000], 0);
    }
}
