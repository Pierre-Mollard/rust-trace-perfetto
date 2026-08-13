pub enum EventType {
    DurationBegin,
    DurationEnd,
    Complete,
    Instant,
    AsyncStart,
    AsyncInstant,
    AsyncEnd,
    FlowStart,
    FlowStep,
    FlowStepSample,
    ObjectCreated,
    ObjectSnapshot,
    ObjectDestroyed,
    Metadata,
    MemoryDumpGlobal,
    MemoryDumpProcess,
    Mark,
    ClockSync,
    Context,
}

pub struct Trace {
    name: String,
    categories: Vec<String>,
    event_type: EventType,
    timestamp: u64,
    pid: u16,
    tid: u16,
    args: Vec<String>,
}

pub fn print_generated_perfetto_traces(traces: &Vec<Trace>) {
    println!("todo implement");
    //TODO: implement
}
