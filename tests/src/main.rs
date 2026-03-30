use std::vec;

use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSlice};

mod huffcode;

struct EncodedSymbol{
    bits: u32,
    len: u8,
}

fn main() {
    let map = huffcode::jpeg_ac_luminance_table();

    let x: (u8, i64) = (0,0);

    let mut y = 0u64;

    println!("{:b}", y);

    let cat = categorie(x.1 as i16);
 
    let huff = map.get(&(x.0, cat)).unwrap(); 

    println!("CAT: {}", cat);


    y = (huff.code as u64) << cat;
    
    

    println!("{:b}", y);

    if x.1 > 0{
        y = y | x.1 as u64;
    }else{
        y = y | (x.1 + (1i64 << cat) - 1) as u64;
    }


    println!("{:b}", x.1);

    println!("{:b}", y);

    let bits = EncodedSymbol { bits: y as u32, len: cat + huff.len };


    println!("BITS: {:b} Länge: {}", bits.bits, bits.len);


}

fn categorie( x:i16)->u8{
    if x == 0{
        0
    }else{
        (x.abs() as f32).log2().floor() as u8 + 1
    }
}