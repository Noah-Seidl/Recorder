use std::vec;

use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSlice};

fn main() {
    let mut vector = vec![2;30];
    let mut erg = vec![1;30];

    erg.chunks_mut(8)
    .enumerate()
    .for_each(|(index, chunk)|{
        println!("{}", index);
        if(index == 2){
            chunk.copy_from_slice(&vector[0..8]);
        }
    });




    println!("ERGEBNIS: {:?}", erg);

}
