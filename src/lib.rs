use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, fs};

#[derive(Serialize, Debug)]
pub enum EventType {
    #[serde(rename = "B")]
    DurationBegin,
    #[serde(rename = "E")]
    DurationEnd,
    #[serde(rename = "X")]
    Complete,
    #[serde(rename = "i")]
    Instant,
    #[serde(rename = "b")]
    AsyncStart,
    #[serde(rename = "n")]
    AsyncInstant,
    #[serde(rename = "e")]
    AsyncEnd,
    #[serde(rename = "s")]
    FlowStart,
    #[serde(rename = "t")]
    FlowStep,
    #[serde(rename = "f")]
    FlowEnd,
    #[serde(rename = "P")]
    Sample,
    #[serde(rename = "N")]
    ObjectCreated,
    #[serde(rename = "O")]
    ObjectSnapshot,
    #[serde(rename = "D")]
    ObjectDestroyed,
    #[serde(rename = "M")]
    Metadata,
    #[serde(rename = "V")]
    MemoryDumpGlobal,
    #[serde(rename = "v")]
    MemoryDumpProcess,
    #[serde(rename = "R")]
    Mark,
    #[serde(rename = "c")]
    ClockSync,
    #[serde(rename = "(,)")]
    Context,
}

#[derive(Serialize, Debug)]
pub struct TraceEvent {
    name: String,

    #[serde(rename = "cat", skip_serializing_if = "String::is_empty")]
    categories: String,

    #[serde(rename = "ph")]
    event_type: EventType,

    #[serde(rename = "ts")]
    timestamp: u64,

    pid: u16,
    tid: u16,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, String>,
}

pub fn process_trace_lines(lines: Vec<&str>) -> Vec<TraceEvent> {
    let mut valid_events: Vec<TraceEvent> = Vec::new();

    // Simulates the call stack per thread: HashMap<tid, Vec<TraceEvent>>
    let mut active_calls: HashMap<u16, Vec<TraceEvent>> = HashMap::new();

    for (line_num, line) in lines.iter().enumerate() {
        if let Some(event) = TraceEvent::parse_line(line) {
            match event.event_type {
                EventType::DurationBegin => {
                    active_calls.entry(event.tid).or_default().push(event);
                }
                EventType::DurationEnd => {
                    let stack = active_calls.entry(event.tid).or_default();

                    if let Some(start_event) = stack.pop() {
                        // Check if the names match
                        if start_event.name == event.name {
                            valid_events.push(start_event);
                            valid_events.push(event);
                        } else {
                            // Mismatch! (e.g., entered A, but exited B)
                            eprintln!(
                                "WARNING (Line {}): Mismatched exit on Thread {}. Expected to exit '{}', but got exit for '{}'. Discarding both.",
                                line_num + 1,
                                event.tid,
                                start_event.name,
                                event.name
                            );
                        }
                    } else {
                        eprintln!(
                            "WARNING (Line {}): Orphaned exit event for '{}' on Thread {}. No matching 'enter' found. Discarding.",
                            line_num + 1,
                            event.name,
                            event.tid
                        );
                    }
                }
                _ => {
                    valid_events.push(event);
                }
            }
        }
    }

    // END OF FILE: Check for 'B' events that never got an 'E'
    for (tid, stack) in active_calls {
        for unfinished_event in stack {
            eprintln!(
                "WARNING (EOF): Function '{}' on Thread {} started but never exited. Discarding.",
                unfinished_event.name, tid
            );
        }
    }

    valid_events
}

impl TraceEvent {
    pub fn parse_line(line: &str) -> Option<Self> {
        let (main_part, args_part) = line.split_once(" , ").unwrap_or((line, ""));
        let parts: Vec<&str> = main_part.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }

        // Parse Timestamp (split at '.')
        let ts_parts: Vec<&str> = parts[0].split('.').collect();
        let seconds: u64 = ts_parts[0].parse().unwrap_or(0);
        let nanos: u64 = ts_parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
        let timestamp = (seconds * 1_000_000_000) + nanos;

        let tid: u16 = parts[1].parse().unwrap_or(0);
        let pid: u16 = parts[2].parse().unwrap_or(0); // Assuming cpu id = pid

        // Parse EventType
        let event_type = match parts[3] {
            "enter" => EventType::DurationBegin,
            "exit" => EventType::DurationEnd,
            _ => return None,
        };

        // Parse module and function name
        let module_and_func = parts[5];
        let (category, name) = module_and_func
            .split_once(':')
            .unwrap_or(("", module_and_func));

        // Parse Arguments into a HashMap
        let mut args = HashMap::new();
        if !args_part.is_empty() {
            // Split by comma for each param
            for arg_pair in args_part.split(',') {
                let arg_parts: Vec<&str> = arg_pair.trim().splitn(2, ' ').collect();
                if arg_parts.len() == 2 {
                    args.insert(arg_parts[0].to_string(), arg_parts[1].to_string());
                } else {
                    args.insert("info".to_string(), arg_parts[0].to_string());
                }
            }
        }

        Some(TraceEvent {
            name: name.to_string(),
            categories: category.to_string(),
            event_type,
            timestamp,
            pid,
            tid,
            args,
        })
    }
}

pub fn print_generated_perfetto_traces(events: &Vec<TraceEvent>) {
    let output = json!({
        "traceEvents": events,
        "displayTimeUnit": "ns"
    });

    let json_string = serde_json::to_string_pretty(&output).unwrap();
    //println!("{}", json_string);
    //
    //
    fs::write("trace.json", json_string).expect("Failed to write to file");

    println!("Trace saved to trace.json");
}
