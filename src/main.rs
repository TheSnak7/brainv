use std::{env, fs, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);

    let program_path = &args[1];

    let program_text = fs::read_to_string(Path::new(program_path)).expect("Failed to read the file");

    println!("BF Program\n =======\n{}\n=======", program_text);
}
