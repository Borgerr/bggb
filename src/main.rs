use std::fs::File;
use std::io::{stdin, Read};

mod memory;
use memory::Memory;

mod cpu;

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

    let mem = Memory::from(rom_data).expect("Memory given was invalid and cannot be read");

    Ok(())
}
