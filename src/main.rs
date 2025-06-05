use std::process;

mod cli;
mod crypto;
mod plugin;

fn main() {
    env_logger::init();
    
    match cli::run() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
} 