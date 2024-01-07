use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let program_input = &args[1];
    println!("{}", program_input);
}
