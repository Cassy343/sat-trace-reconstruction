mod sat;
mod trace;

use std::collections::HashMap;

use sat::*;
use trace::*;

const MESSAGE_LEN: usize = 10;
const DELETION_PROBABILITY: f32 = 0.5;

fn main() {
    let message = new_message(MESSAGE_LEN);

    let mut weights = HashMap::new();
    let reconstructed = loop {
        let trace = new_trace(&message, DELETION_PROBABILITY);

        for seq in supersequences(&trace, MESSAGE_LEN) {
            let weight = count_occurrences(&trace, &seq);

            if weight == 0 {
                weights.remove(&seq);
                continue;
            }

            weights
                .entry(seq)
                .and_modify(|w| *w *= weight)
                .or_insert(weight);
        }

        let weight_sum = weights.values().sum::<usize>() as f64;
        if let Some((msg, _)) = weights.iter().find(|(_, &weight)| weight as f64 / weight_sum > 0.95) {
            break msg;
        }
    };

    println!("{}\n{}", message, reconstructed);

    // let trace = new_trace(&message, DELETION_PROBABILITY);
    // let mut td = TraceDisjunction::from_trace(&trace, MESSAGE_LEN);

    // let reconstructed = loop {
    //     // println!("{}", td);

    //     let trace = new_trace(&message, DELETION_PROBABILITY);

    //     println!("{}", trace.len());

    //     td.and(TraceDisjunction::clauses_from_trace(&trace, MESSAGE_LEN));

    //     if let Some(message) = td.message() {
    //         break message;
    //     }
    // };

    // assert_eq!(message, reconstructed);

    // use bitvec::{bits, prelude::*};

    // println!("{}", count_occurrences(bits![1,1,0,0,1,1,0,1], bits![0,0]));
}
