use std::vec;

use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSlice};

fn main() {

    let mut vector = vec![0u32;10];
    vector[5] = 10;
    println!("VEctor: {:?}", &vector[..]);
}
