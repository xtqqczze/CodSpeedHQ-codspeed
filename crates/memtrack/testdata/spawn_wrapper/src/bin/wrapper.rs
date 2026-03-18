use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: wrapper <command> [args...]");
        std::process::exit(1);
    }

    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .spawn()
        .expect("Failed to spawn child process");

    let status = child.wait().expect("Failed to wait for child");
    std::process::exit(status.code().unwrap_or(1));
}
