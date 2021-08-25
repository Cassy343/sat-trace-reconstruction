use bitvec::order::Lsb0;
use bitvec::{prelude::BitVec, slice::BitSlice};
use rand::{thread_rng, Rng};

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
