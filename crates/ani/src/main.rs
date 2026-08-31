use std::fs;

use ani::Ani;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = fs::File::open("./IRyS_01.ani")?;
    let ani = Ani::from_reader(&mut reader)?;

    println!("Metadata: {:?}", ani.metadata());
    println!("Header: {:?}", ani.header());
    println!("Rates: {:?}", ani.rates());
    println!("Sequence: {:?}", ani.sequence());
    println!("Total Frames: {:?}", ani.frames().len());

    Ok(())
}
