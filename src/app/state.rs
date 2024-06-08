use std::time::Instant;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Initialising,
    Calibrating { second_stage: bool },
    Running { start: Instant },
    Exiting,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event {
    Initialised,
    Calibrated,
    Exited,
}
