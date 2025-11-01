fn main() {
    if let Err(e) = fwtype::get_args().and_then(fwtype::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
