mod sat;
mod trace;

use sat::*;
use trace::*;

const MESSAGE_LEN: usize = 8;
const DELETION_PROBABILITY: f32 = 0.5;

fn main() {
    let message = new_message(MESSAGE_LEN);

    let trace = new_trace(&message, DELETION_PROBABILITY);
    let mut td = TraceDisjunction::from_trace(&trace, MESSAGE_LEN);

    let reconstructed = loop {
        println!("{}", td);

        let trace = new_trace(&message, DELETION_PROBABILITY);
        td.and(TraceDisjunction::clauses_from_trace(&trace, MESSAGE_LEN));

        if let Some(message) = td.message() {
            break message;
        }
    };

    assert_eq!(message, reconstructed);
}
