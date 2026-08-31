use std::io::Write as _;
use std::{env, fs, io};

use inf::{Entry, Inf, Value};

fn main() {
    let path = env::args().nth(1).expect("expected path as first argument");
    let mut reader = fs::File::open(path).expect("failed to open file");
    let inf = Inf::from_reader(&mut reader).expect("failed to parse INF file");
    let mut stdout = io::stdout().lock();

    for section in inf.sections() {
        writeln!(stdout, "[{}]", section.name()).ok();

        for entry in section.entries() {
            match entry {
                Entry::Item(key, Value::Raw(value)) => println!("{key} = \"{value}\""),
                Entry::Item(key, Value::List(values)) => {
                    writeln!(
                        stdout,
                        "{key} = {}",
                        values
                            .iter()
                            .map(|v| format!("\"{v}\""))
                            .collect::<Vec<String>>()
                            .join(",")
                    )
                    .ok();
                }
                Entry::Value(Value::Raw(value)) => {
                    writeln!(stdout, "\"{value}\"").ok();
                }
                Entry::Value(Value::List(values)) => {
                    writeln!(
                        stdout,
                        "{}",
                        values
                            .iter()
                            .map(|v| format!("\"{v}\""))
                            .collect::<Vec<String>>()
                            .join(",")
                    )
                    .ok();
                }
            }
        }

        writeln!(stdout).ok();
    }
}
