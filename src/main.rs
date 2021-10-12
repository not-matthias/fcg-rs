use log::LevelFilter;
use simple_logger::SimpleLogger;

pub mod config;
pub mod parser;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();

    println!("Hello, world!");
}
