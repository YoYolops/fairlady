mod server;
mod types;
mod constants;

use server::listener::{spawn_tcp_listener};

fn main() {
    match spawn_tcp_listener() {
        Ok(_) => println!("It worked!11"),
        Err(e) => e.print_stack_trace(),
    }
}
