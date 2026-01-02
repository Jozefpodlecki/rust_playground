use std::env;

use crate::inspect::Analyser;

mod inspect;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Invalid args len");
        return;
    }

    let path = args[1].clone().into();
    let output = "output.txt".into();
    let analyser = Analyser::new(path, output);

    analyser.read_pdata().unwrap();
}
