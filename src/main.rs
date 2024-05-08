use std::fs::File;
use std::io::{stdin, Read};

mod memory;
use memory::Memory;

mod cpu;
use cpu::CPU;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
    println!("enter file to read from");
    let mut filename = String::new();
    stdin().read_line(&mut filename)?;

    println!("(-) reading from {}...", filename.trim());

    let mut f = File::open(filename.trim().to_string())?;
    let mut rom_data = Vec::new();
    // read the whole file
    f.read_to_end(&mut rom_data)?;

    // TODO: investigate more boot-time initialization things at https://gbdev.io/pandocs/Power_Up_Sequence.html
    let mut mem = Memory::from(rom_data).expect("Memory given was invalid and cannot be read"); // TODO: handle more gracefully
    let mut cpu = CPU::new(0, false, mem.header.header_checksum);
    loop {
        // TODO: display graphics, handle errors more gracefully
        let execution_result = cpu.fetch_decode_execute(&mut mem);

        if let Err(e) = execution_result {
            println!("{}", e);
            break Ok(());
        }
    }
}
