use std::{env, fs};

fn main() {
    let input = env::args().nth(1).expect("no input file provided");
    let path = fs::canonicalize(&input).expect("failed to canonicalize input");
    let xcursor = xcur::Xcursor::open(path).expect("failed to parse Xcursor");

    for image in xcursor.images() {
        println!("Width: {}", image.width());
        println!("Height: {}", image.height());
        println!("Hotspot X: {}", image.hotspot_x());
        println!("Hotspot Y: {}", image.hotspot_y());
        println!("Delay: {:?}", image.delay());
        println!("{:-^80}", "")
    }
}
