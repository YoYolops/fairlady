// LLM MADE
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";

pub fn error(message: String) {
    println!("{}{}{}", RED, message, RESET);
}

pub fn warn(message: String) {
    println!("{}{}{}", YELLOW, message, RESET);
}

pub fn success(message: String) {
    println!("{}{}{}", GREEN, message, RESET);
}

pub fn info(message: String) {
    println!("{}{}{}", BLUE, message, RESET);
}
