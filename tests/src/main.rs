use std::vec;

use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSlice};

fn main() {

    let mut vector = vec![2.3f32;7];
    let vec:Vec<u8> = vector.iter().map(|x|{*x as u8}).collect();
    println!("VEC u8: {:?}" ,&vec[..]);
}
