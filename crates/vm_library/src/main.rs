use vm_library::{pipe::IO, run_loop::run_loop};

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let io = IO::new(args.remove(1));
    run_loop(io);
}
