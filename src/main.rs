fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("gan {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    eprintln!("gan: the Gandora compiler (work in progress)");
    std::process::exit(2);
}
