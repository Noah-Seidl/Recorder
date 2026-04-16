use std::{collections::HashMap, vec};

use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSlice};

use crate::huffcode::HuffCode;

mod huffcode;
mod bit_writer;
struct EncodedSymbol{
    bits: u32,
    len: u8,
}

fn rle_to_bits_ac(rle:(u8,i64), map: & HashMap<(u8, u8), HuffCode>) -> u32{    
    let cat = categorie(rle.1 as i16);
    let huff = map.get(&(rle.0, cat)).unwrap(); 
    let mut y = (huff.code as u32) << cat;
    if rle.1 > 0{
        y = y | rle.1 as u32;
    }else{
        y = y | (rle.1 + (1i64 << cat) - 1) as u32;
    }

    y
}

fn rle_to_bits_dc(rle:(u8,i64), map: & HashMap< u8, HuffCode>) -> u32{    
    let cat = categorie(rle.1 as i16);
    let huff = map.get(&cat).unwrap(); 
    let mut y = (huff.code as u32) << cat;
    if rle.1 > 0{
        y = y | rle.1 as u32;
    }else{
        y = y | (rle.1 + (1i64 << cat) - 1) as u32;
    }

    y
}

fn main() {
    let map = huffcode::jpeg_ac_luminance_table();

    let x: (u8, i64) = (0,16);

    let mut y = 0u64;

    //println!("{:b}", y);

    let cat = categorie(x.1 as i16);
 
    let huff = map.get(&(x.0, cat)).unwrap(); 

    println!("CAT: {}", cat);


    y = (huff.code as u64) << cat;
    
    

    //println!("{:b}", y);

    if x.1 > 0{
        y = y | x.1 as u64;
    }else{
        y = y | (x.1 + (1i64 << cat) - 1) as u64;
    }
    
    let u = rle_to_bits_ac(x, &map);

    //println!("{:b}", x.1);
    println!("Y: {:b}", y);
    println!("ERG: {:b}", u);

    let bits = EncodedSymbol { bits: y as u32, len: cat + huff.len };

    println!("BITS: {:032b} Länge: {}", bits.bits, bits.len);
    println!("BITS: {:b} Länge: {}", bits.bits, bits.len);

    let mut writer = bit_writer::BitWriter::new();

    writer.write_bits(bits.bits as u64, bits.len);
    writer.write_bits(bits.bits as u64, bits.len);
    writer.write_bits(bits.bits as u64, bits.len);
    writer.write_bits(bits.bits as u64, bits.len);
    writer.write_bits(bits.bits as u64, bits.len);
    writer.write_bits(bits.bits as u64, bits.len);
    writer.write_bits(bits.bits as u64, bits.len);
    writer.write_bits(bits.bits as u64, bits.len);

    writer.print_bits();

    let lut:Vec<huffcode::InvertedHuf> = huffcode::lut_ac(&map);

    let buffer = writer.get_buffer();
    let wert = ((buffer[0]as u16) << 8) | buffer[1] as u16;


    let inv_huf = lut[wert as usize];
    println!("run: {} cat: {} huf_len: {}", inv_huf.run, inv_huf.cat, inv_huf.huf_len);
}

fn categorie( x:i16)->u8{
    if x == 0{
        0
    }else{
        (x.abs() as f32).log2().floor() as u8 + 1
    }
}

