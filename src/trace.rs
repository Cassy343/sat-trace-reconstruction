use bitvec::order::Lsb0;
use bitvec::{prelude::BitVec, slice::BitSlice};
use rand::{thread_rng, Rng};

pub fn count_occurrences(sequence: &BitSlice, subsequence: &BitSlice) -> usize {
    let mut positions = vec![0; subsequence.len()];
    let mut i = 0;

    while i < positions.len() {
        if positions[i] >= sequence.len() {
            return 0;
        }

        if sequence[positions[i]] != subsequence[i] {
            positions[i] += 1;
        } else {
            let next = positions[i] + 1;
            if let Some(pos) = positions.get_mut(i + 1) {
                *pos = next;
            }
            i += 1;
        }
    }

    let mut count = 1;

    while permute_subsequence(sequence, subsequence, &mut positions, subsequence.len() - 1, sequence.len()) {
        println!("---");
        count += 1;
    }

    count
}

pub fn permute_subsequence(
    sequence: &BitSlice,
    subsequence: &BitSlice,
    positions: &mut [usize],
    nth_pos: usize,
    bound: usize
) -> bool {
    // This algorithm works by taking combinations of the subsequence within the original sequence
    // through a recursive algorithm. We increment the nth index up until `bound` or until bit in
    // the sequence at the nth index matches the nth bit in the subsequence. If these bits do not
    // match, then we increment the n-1-th index until it matches, etc. If we reach the 0-th bit
    // of the subsequence and we cannot increment it further, we unwind the stack.

    println!("{}", nth_pos);
    // Eagerly increment the index of the nth bit until we find a match or hit the bound
    while positions[nth_pos] < bound - 1 {
        positions[nth_pos] += 1;

        if sequence[positions[nth_pos]] == subsequence[nth_pos] {
            println!("{:?}", positions);
            return true;
        }
    }

    // We break the recursion here
    if nth_pos == 0 {
        return false;
    }

    loop {
        // We can't increment the nth index any further, so try incrementing the n-1-th index
        if !permute_subsequence(sequence, subsequence, positions, nth_pos - 1, bound - 1) {
            return false;
        }

        // Set the nth index to be the index immediately after the n-1-th index
        positions[nth_pos] = positions[nth_pos - 1] + 1;

        // Increment the nth index until the bits match if needed
        while positions[nth_pos] < bound {
            if sequence[positions[nth_pos]] == subsequence[nth_pos] {
                println!("{:?}", positions);
                return true;
            }

            positions[nth_pos] += 1;
        }
    }
}

pub fn supersequences(trace: &BitSlice, message_len: usize) -> SupersequenceIter<'_> {
    SupersequenceIter::new(trace, message_len)
}

pub struct SupersequenceIter<'a> {
    trace: &'a BitSlice,
    message_len: usize,
    combination: Option<Vec<usize>>,
}

impl<'a> SupersequenceIter<'a> {
    fn new(trace: &'a BitSlice, message_len: usize) -> Self {
        Self {
            trace,
            message_len,
            combination: Some((0..trace.len()).collect()),
        }
    }
}

impl<'a> Iterator for SupersequenceIter<'a> {
    type Item = BitVec;

    fn next(&mut self) -> Option<Self::Item> {
        let combination = self.combination.as_deref()?;

        let sequence: BitVec = combination
            .iter()
            .map(|&index| self.trace[index])
            .collect();

        if !next_combination(self.combination.as_mut().unwrap(), self.message_len) {
            self.combination = None;
        }

        Some(sequence)
    }
}

pub fn new_message(len: usize) -> BitVec {
    let mut data = BitVec::new();
    let mut rng = thread_rng();

    let max = (len + usize::BITS as usize - 1) / usize::BITS as usize;
    for i in 0..max {
        let bits: usize = rng.gen();
        let bit_slice = BitSlice::<Lsb0, _>::from_element(&bits);

        let bound = if i != max - 1 {
            usize::BITS as usize
        } else {
            let rem = len % usize::BITS as usize;
            if rem != 0 {
                rem
            } else {
                usize::BITS as usize
            }
        };

        data.extend_from_bitslice(&bit_slice[..bound]);
    }

    data
}

pub fn new_trace(message: &BitSlice, deletion_prob: f32) -> BitVec {
    let mut trace = BitVec::new();
    let mut rng = thread_rng();

    for bit in message.iter().map(|bit_ref| *bit_ref) {
        if rng.gen::<f32>() >= deletion_prob {
            trace.push(bit);
        }
    }

    trace
}

/// Takes an input of numbers between 0 (inclusive) and bound (exclusive) in strictly increasing
/// order and computes the next combination of non-duplicated numbers between 0 and bound.
pub fn next_combination(set: &mut [usize], bound: usize) -> bool {
    let len = set.len();

    // Empty set, no combinations
    if len == 0 {
        return false;
    }

    // Increment the final index
    set[len - 1] += 1;

    // If we hit the bound, find the highest-indexed number we can increase, and then count up from
    // that number by one for the remaining numbers
    if set[len - 1] == bound {
        // If the set is length one then we just need to count up to bound
        if len == 1 {
            return false;
        }

        // Find the highest index we can increase by one without collision
        let mut offset = 2;
        loop {
            // We can increase the number without exceeding the bound of violating the invariant
            // that the set is strictly increasing
            if set[len - offset] < bound - offset {
                break;
            }

            // We could not find a number to increase, all combinations have been taken
            if offset == len {
                return false;
            }

            offset += 1;
        }

        set[len - offset] += 1;
        let mut value = set[len - offset];

        // Set the remaining numbers to increase by one after the number we just incremented
        for i in len - offset + 1..len {
            value += 1;
            set[i] = value;
        }
    }

    true
}