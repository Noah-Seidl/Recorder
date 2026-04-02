use std::{collections::HashMap, net::UdpSocket};

#[derive(Debug, Clone, Copy)]
pub struct HuffCode {
    pub code: u32,
    pub len: u8,
}

impl HuffCode {
    const fn new(code: u32, len: u8) -> Self {
        Self { code, len }
    }
}

fn categorie(x:i16)->usize{
    if x == 0{
        0
    }else{
        (x.abs() as f32).log2().floor() as usize + 1
    }
}


pub fn jpeg_dc_luminance_table() -> HashMap<u8, HuffCode>{
    let mut map = HashMap::new();

    map.insert(0, HuffCode { code: 0b00, len: 2 });
    map.insert(1, HuffCode { code: 0b010, len: 3 });
    map.insert(2, HuffCode { code: 0b011, len: 3 });
    map.insert(3, HuffCode { code: 0b100, len: 3 });
    map.insert(4, HuffCode { code: 0b101, len: 3 });
    map.insert(5, HuffCode { code: 0b110, len: 3 });
    map.insert(6, HuffCode { code: 0b1110, len: 4 });
    map.insert(7, HuffCode { code: 0b11110, len: 5 });
    map.insert(8, HuffCode { code: 0b111110, len: 6 });
    map.insert(9, HuffCode { code: 0b1111110, len: 7 });
    map.insert(10, HuffCode { code: 0b11111110, len: 8 });
    map.insert(11, HuffCode { code: 0b111111110, len: 9 });

    
    map
}



pub fn jpeg_ac_luminance_table() -> HashMap<(u8, u8), HuffCode> {
    let mut map = HashMap::new();

    // Run 0
    map.insert((0x0, 0x0), HuffCode::new(0b1010,             4));  // EOB
    map.insert((0x0, 0x1), HuffCode::new(0b00,               2));
    map.insert((0x0, 0x2), HuffCode::new(0b01,               2));
    map.insert((0x0, 0x3), HuffCode::new(0b100,              3));
    map.insert((0x0, 0x4), HuffCode::new(0b1011,             4));
    map.insert((0x0, 0x5), HuffCode::new(0b11010,            5));
    map.insert((0x0, 0x6), HuffCode::new(0b1111000,          7));
    map.insert((0x0, 0x7), HuffCode::new(0b11111000,         8));
    map.insert((0x0, 0x8), HuffCode::new(0b1111110110,       10));
    map.insert((0x0, 0x9), HuffCode::new(0b1111111110000010, 16));
    map.insert((0x0, 0xA), HuffCode::new(0b1111111110000011, 16));

    // Run 1
    map.insert((0x1, 0x1), HuffCode::new(0b1100,             4));
    map.insert((0x1, 0x2), HuffCode::new(0b11011,            5));
    map.insert((0x1, 0x3), HuffCode::new(0b1111001,          7));
    map.insert((0x1, 0x4), HuffCode::new(0b111110110,        9));
    map.insert((0x1, 0x5), HuffCode::new(0b11111110110,      11));
    map.insert((0x1, 0x6), HuffCode::new(0b1111111110000100, 16));
    map.insert((0x1, 0x7), HuffCode::new(0b1111111110000101, 16));
    map.insert((0x1, 0x8), HuffCode::new(0b1111111110000110, 16));
    map.insert((0x1, 0x9), HuffCode::new(0b1111111110000111, 16));
    map.insert((0x1, 0xA), HuffCode::new(0b1111111110001000, 16));

    // Run 2
    map.insert((0x2, 0x1), HuffCode::new(0b11100,            5));
    map.insert((0x2, 0x2), HuffCode::new(0b11111001,         8));
    map.insert((0x2, 0x3), HuffCode::new(0b1111110111,       10));
    map.insert((0x2, 0x4), HuffCode::new(0b111111110100,     12));
    map.insert((0x2, 0x5), HuffCode::new(0b1111111110001001, 16));
    map.insert((0x2, 0x6), HuffCode::new(0b1111111110001010, 16));
    map.insert((0x2, 0x7), HuffCode::new(0b1111111110001011, 16));
    map.insert((0x2, 0x8), HuffCode::new(0b1111111110001100, 16));
    map.insert((0x2, 0x9), HuffCode::new(0b1111111110001101, 16));
    map.insert((0x2, 0xA), HuffCode::new(0b1111111110001110, 16));

    // Run 3
    map.insert((0x3, 0x1), HuffCode::new(0b111010,           6));
    map.insert((0x3, 0x2), HuffCode::new(0b111110111,        9));
    map.insert((0x3, 0x3), HuffCode::new(0b111111110101,     12));
    map.insert((0x3, 0x4), HuffCode::new(0b1111111110001111, 16));
    map.insert((0x3, 0x5), HuffCode::new(0b1111111110010000, 16));
    map.insert((0x3, 0x6), HuffCode::new(0b1111111110010001, 16));
    map.insert((0x3, 0x7), HuffCode::new(0b1111111110010010, 16));
    map.insert((0x3, 0x8), HuffCode::new(0b1111111110010011, 16));
    map.insert((0x3, 0x9), HuffCode::new(0b1111111110010100, 16));
    map.insert((0x3, 0xA), HuffCode::new(0b1111111110010101, 16));

    // Run 4
    map.insert((0x4, 0x1), HuffCode::new(0b111011,           6));
    map.insert((0x4, 0x2), HuffCode::new(0b1111111000,       10));
    map.insert((0x4, 0x3), HuffCode::new(0b1111111110010110, 16));
    map.insert((0x4, 0x4), HuffCode::new(0b1111111110010111, 16));
    map.insert((0x4, 0x5), HuffCode::new(0b1111111110011000, 16));
    map.insert((0x4, 0x6), HuffCode::new(0b1111111110011001, 16));
    map.insert((0x4, 0x7), HuffCode::new(0b1111111110011010, 16));
    map.insert((0x4, 0x8), HuffCode::new(0b1111111110011011, 16));
    map.insert((0x4, 0x9), HuffCode::new(0b1111111110011100, 16));
    map.insert((0x4, 0xA), HuffCode::new(0b1111111110011101, 16));

    // Run 5
    map.insert((0x5, 0x1), HuffCode::new(0b1111010,          7));
    map.insert((0x5, 0x2), HuffCode::new(0b11111110111,      11));
    map.insert((0x5, 0x3), HuffCode::new(0b1111111110011110, 16));
    map.insert((0x5, 0x4), HuffCode::new(0b1111111110011111, 16));
    map.insert((0x5, 0x5), HuffCode::new(0b1111111110100000, 16));
    map.insert((0x5, 0x6), HuffCode::new(0b1111111110100001, 16));
    map.insert((0x5, 0x7), HuffCode::new(0b1111111110100010, 16));
    map.insert((0x5, 0x8), HuffCode::new(0b1111111110100011, 16));
    map.insert((0x5, 0x9), HuffCode::new(0b1111111110100100, 16));
    map.insert((0x5, 0xA), HuffCode::new(0b1111111110100101, 16));

    // Run 6
    map.insert((0x6, 0x1), HuffCode::new(0b1111011,          7));
    map.insert((0x6, 0x2), HuffCode::new(0b111111110110,     12));
    map.insert((0x6, 0x3), HuffCode::new(0b1111111110100110, 16));
    map.insert((0x6, 0x4), HuffCode::new(0b1111111110100111, 16));
    map.insert((0x6, 0x5), HuffCode::new(0b1111111110101000, 16));
    map.insert((0x6, 0x6), HuffCode::new(0b1111111110101001, 16));
    map.insert((0x6, 0x7), HuffCode::new(0b1111111110101010, 16));
    map.insert((0x6, 0x8), HuffCode::new(0b1111111110101011, 16));
    map.insert((0x6, 0x9), HuffCode::new(0b1111111110101100, 16));
    map.insert((0x6, 0xA), HuffCode::new(0b1111111110101101, 16));

    // Run 7
    map.insert((0x7, 0x1), HuffCode::new(0b11111010,         8));
    map.insert((0x7, 0x2), HuffCode::new(0b111111110111,     12));
    map.insert((0x7, 0x3), HuffCode::new(0b1111111110101110, 16));
    map.insert((0x7, 0x4), HuffCode::new(0b1111111110101111, 16));
    map.insert((0x7, 0x5), HuffCode::new(0b1111111110110000, 16));
    map.insert((0x7, 0x6), HuffCode::new(0b1111111110110001, 16));
    map.insert((0x7, 0x7), HuffCode::new(0b1111111110110010, 16));
    map.insert((0x7, 0x8), HuffCode::new(0b1111111110110011, 16));
    map.insert((0x7, 0x9), HuffCode::new(0b1111111110110100, 16));
    map.insert((0x7, 0xA), HuffCode::new(0b1111111110110101, 16));

    // Run 8
    map.insert((0x8, 0x1), HuffCode::new(0b111111000,        9));
    map.insert((0x8, 0x2), HuffCode::new(0b111111111000000,  15)); // ⚠ 15 Bit laut Quelle – gegen ITU-T T.81 prüfen!
    map.insert((0x8, 0x3), HuffCode::new(0b1111111110110110, 16));
    map.insert((0x8, 0x4), HuffCode::new(0b1111111110110111, 16));
    map.insert((0x8, 0x5), HuffCode::new(0b1111111110111000, 16));
    map.insert((0x8, 0x6), HuffCode::new(0b1111111110111001, 16));
    map.insert((0x8, 0x7), HuffCode::new(0b1111111110111010, 16));
    map.insert((0x8, 0x8), HuffCode::new(0b1111111110111011, 16));
    map.insert((0x8, 0x9), HuffCode::new(0b1111111110111100, 16));
    map.insert((0x8, 0xA), HuffCode::new(0b1111111110111101, 16));

    // Run 9
    map.insert((0x9, 0x1), HuffCode::new(0b111111001,        9));
    map.insert((0x9, 0x2), HuffCode::new(0b1111111110111110, 16));
    map.insert((0x9, 0x3), HuffCode::new(0b1111111110111111, 16));
    map.insert((0x9, 0x4), HuffCode::new(0b1111111111000000, 16));
    map.insert((0x9, 0x5), HuffCode::new(0b1111111111000001, 16));
    map.insert((0x9, 0x6), HuffCode::new(0b1111111111000010, 16));
    map.insert((0x9, 0x7), HuffCode::new(0b1111111111000011, 16));
    map.insert((0x9, 0x8), HuffCode::new(0b1111111111000100, 16));
    map.insert((0x9, 0x9), HuffCode::new(0b1111111111000101, 16));
    map.insert((0x9, 0xA), HuffCode::new(0b1111111111000110, 16));

    // Run A
    map.insert((0xA, 0x1), HuffCode::new(0b111111010,        9));
    map.insert((0xA, 0x2), HuffCode::new(0b1111111111000111, 16));
    map.insert((0xA, 0x3), HuffCode::new(0b1111111111001000, 16));
    map.insert((0xA, 0x4), HuffCode::new(0b1111111111001001, 16));
    map.insert((0xA, 0x5), HuffCode::new(0b1111111111001010, 16));
    map.insert((0xA, 0x6), HuffCode::new(0b1111111111001011, 16));
    map.insert((0xA, 0x7), HuffCode::new(0b1111111111001100, 16));
    map.insert((0xA, 0x8), HuffCode::new(0b1111111111001101, 16));
    map.insert((0xA, 0x9), HuffCode::new(0b1111111111001110, 16));
    map.insert((0xA, 0xA), HuffCode::new(0b1111111111001111, 16));

    // Run B
    map.insert((0xB, 0x1), HuffCode::new(0b1111111001,       10));
    map.insert((0xB, 0x2), HuffCode::new(0b1111111111010000, 16));
    map.insert((0xB, 0x3), HuffCode::new(0b1111111111010001, 16));
    map.insert((0xB, 0x4), HuffCode::new(0b1111111111010010, 16));
    map.insert((0xB, 0x5), HuffCode::new(0b1111111111010011, 16));
    map.insert((0xB, 0x6), HuffCode::new(0b1111111111010100, 16));
    map.insert((0xB, 0x7), HuffCode::new(0b1111111111010101, 16));
    map.insert((0xB, 0x8), HuffCode::new(0b1111111111010110, 16));
    map.insert((0xB, 0x9), HuffCode::new(0b1111111111010111, 16));
    map.insert((0xB, 0xA), HuffCode::new(0b1111111111011000, 16));

    // Run C
    map.insert((0xC, 0x1), HuffCode::new(0b1111111010,       10));
    map.insert((0xC, 0x2), HuffCode::new(0b1111111111011001, 16));
    map.insert((0xC, 0x3), HuffCode::new(0b1111111111011010, 16));
    map.insert((0xC, 0x4), HuffCode::new(0b1111111111011011, 16));
    map.insert((0xC, 0x5), HuffCode::new(0b1111111111011100, 16));
    map.insert((0xC, 0x6), HuffCode::new(0b1111111111011101, 16));
    map.insert((0xC, 0x7), HuffCode::new(0b1111111111011110, 16));
    map.insert((0xC, 0x8), HuffCode::new(0b1111111111011111, 16));
    map.insert((0xC, 0x9), HuffCode::new(0b1111111111100000, 16));
    map.insert((0xC, 0xA), HuffCode::new(0b1111111111100001, 16));

    // Run D
    map.insert((0xD, 0x1), HuffCode::new(0b11111111000,      11));
    map.insert((0xD, 0x2), HuffCode::new(0b1111111111100010, 16));
    map.insert((0xD, 0x3), HuffCode::new(0b1111111111100011, 16));
    map.insert((0xD, 0x4), HuffCode::new(0b1111111111100100, 16));
    map.insert((0xD, 0x5), HuffCode::new(0b1111111111100101, 16));
    map.insert((0xD, 0x6), HuffCode::new(0b1111111111100110, 16));
    map.insert((0xD, 0x7), HuffCode::new(0b1111111111100111, 16));
    map.insert((0xD, 0x8), HuffCode::new(0b1111111111101000, 16));
    map.insert((0xD, 0x9), HuffCode::new(0b1111111111101001, 16));
    map.insert((0xD, 0xA), HuffCode::new(0b1111111111101010, 16));

    // Run E
    map.insert((0xE, 0x1), HuffCode::new(0b1111111111101011, 16));
    map.insert((0xE, 0x2), HuffCode::new(0b1111111111101100, 16));
    map.insert((0xE, 0x3), HuffCode::new(0b1111111111101101, 16));
    map.insert((0xE, 0x4), HuffCode::new(0b1111111111101110, 16));
    map.insert((0xE, 0x5), HuffCode::new(0b1111111111101111, 16));
    map.insert((0xE, 0x6), HuffCode::new(0b1111111111110000, 16));
    map.insert((0xE, 0x7), HuffCode::new(0b1111111111110001, 16));
    map.insert((0xE, 0x8), HuffCode::new(0b1111111111110010, 16));
    map.insert((0xE, 0x9), HuffCode::new(0b1111111111110011, 16));
    map.insert((0xE, 0xA), HuffCode::new(0b1111111111110100, 16));

    // Run F
    map.insert((0xF, 0x0), HuffCode::new(0b11111111001,      11)); // ZRL
    map.insert((0xF, 0x1), HuffCode::new(0b1111111111110101, 16));
    map.insert((0xF, 0x2), HuffCode::new(0b1111111111110110, 16));
    map.insert((0xF, 0x3), HuffCode::new(0b1111111111110111, 16));
    map.insert((0xF, 0x4), HuffCode::new(0b1111111111111000, 16));
    map.insert((0xF, 0x5), HuffCode::new(0b1111111111111001, 16));
    map.insert((0xF, 0x6), HuffCode::new(0b1111111111111010, 16));
    map.insert((0xF, 0x7), HuffCode::new(0b1111111111111011, 16));
    map.insert((0xF, 0x8), HuffCode::new(0b1111111111111100, 16));
    map.insert((0xF, 0x9), HuffCode::new(0b1111111111111101, 16));
    map.insert((0xF, 0xA), HuffCode::new(0b1111111111111110, 16));

    map
}


pub(crate) fn send_packets(socket:UdpSocket, frame_id: u8, y_rle:Vec<(usize, i16)>, cb_rle:Vec<(usize, i16)>, cr_rle:Vec<(usize, i16)>){

    let mut counter = 0usize;

    while counter < y_rle.len() {
        

    }

}