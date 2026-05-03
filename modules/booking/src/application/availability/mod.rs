pub mod slot_generator;
pub mod slot_subtractor;

pub use slot_generator::{WorkingWindow, generate_slots};
pub use slot_subtractor::subtract_booked;
