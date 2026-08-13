use std::{env, error::Error, fs, process};

use rust_trace_perfetto::{TraceEvent, print_generated_perfetto_traces};

fn main() {
    println!("Hello, world!");

    let args_iter = env::args();

    let config = Config::build(args_iter).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    println!("File {}", config.file_path);

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;
    let output = parse_raw_file(&contents);
    print_generated_perfetto_traces(&output);
    Ok(())
}

struct Config {
    file_path: String,
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(args) => args,
            None => return Err("Didn't get a file path"),
        };

        Ok(Config { file_path })
    }
}

pub fn parse_raw_file(content: &str) -> Vec<TraceEvent> {
    let mut traces: Vec<TraceEvent> = Vec::new();
    let mut counter_missed = 0;

    for line in content.lines() {
        let trace_event = TraceEvent::parse_line(line);

        match trace_event {
            Some(evt) => traces.push(evt),
            None => counter_missed += 1,
        }
    }

    if counter_missed > 0 {
        println!("Warning, #{} parsing errors", counter_missed);
    }

    traces
}
