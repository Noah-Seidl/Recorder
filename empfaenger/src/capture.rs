use std::{collections::HashMap, net::UdpSocket, sync::mpsc, time::Instant, vec};

use rayon::{iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator}, slice::{ParallelSlice, ParallelSliceMut}};


use crate::{bit_writer, fast_dct, huffcode::{self, HuffCode, InvertedHuf}};


const BLOCK_SIZE: u32 = 8;

const RESULTING_WIDTH:usize = 1920;
const RESULTING_HEIGHT:usize = 1080;
const RESULTING_RESOLUTION:usize = RESULTING_HEIGHT * RESULTING_WIDTH;
const IP_ADDR:&str = "127.0.0.1:1234";

const PI:f32 = std::f32::consts::PI;
const MATRIX: [f32; 64] = [
    16.0, 11.0, 10.0, 16.0, 24.0, 40.0, 51.0, 61.0,
    12.0, 12.0, 14.0, 19.0, 26.0, 58.0, 60.0, 55.0,
    14.0, 13.0, 16.0, 24.0, 40.0, 57.0, 69.0, 56.0,
    14.0, 17.0, 22.0, 29.0, 51.0, 87.0, 80.0, 62.0,
    18.0, 22.0, 37.0, 56.0, 68.0, 109.0, 103.0, 77.0,
    24.0, 35.0, 55.0, 64.0, 81.0, 104.0, 113.0, 92.0,
    49.0, 64.0, 78.0, 87.0, 103.0, 121.0, 120.0, 101.0,
    72.0, 92.0, 95.0, 98.0, 112.0, 100.0, 103.0, 99.0,
];

const ZIGZAG_ORDER:[usize;64] = [
     0,  1,  8, 16,  9,  2,  3, 10,
    17, 24, 32, 25, 18, 11,  4,  5,
    12, 19, 26, 33, 40, 48, 41, 34,
    27, 20, 13,  6,  7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36,
    29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46,
    53, 60, 61, 54, 47, 55, 62, 63
];

pub(crate) struct Capture {
    pub(crate) start: Instant,
    pub(crate) counter: i128,
    pub(crate) ycbcr:(Vec<u8>,Vec<u8>,Vec<u8>),
    pub(crate) width: u32,
    pub(crate) height:u32,
    sender: mpsc::SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>,
    pub(crate) second_last : u64,
    buffer_y: Vec<u8>,  //Statt immer wieder neuen vec zu erstellen einfach den buffer benutzen
    huff_table_dc:HashMap<u8,HuffCode>,
    huff_table_ac:HashMap<(u8, u8),HuffCode>,
    lut_ac: Vec<InvertedHuf>,
    lut_dc: Vec<InvertedHuf>,
    bit_writer: bit_writer::BitWriter,
    frame_id:u8,
}


impl Capture{

    pub(crate) fn new(sender:mpsc::SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dc_table= huffcode::jpeg_dc_luminance_table();
        let ac_table = huffcode::jpeg_ac_luminance_table();
       
        Ok(Self {
            start: Instant::now(),
            counter: -1,
            width: 1920,
            height: 1080,
            sender,
            second_last: 0,
            ycbcr: (Vec::new(), Vec::new(), Vec::new()) ,
            buffer_y: vec![0u8;RESULTING_RESOLUTION],
            lut_dc: huffcode::lut_dc(&dc_table),
            lut_ac: huffcode::lut_ac(&ac_table), 
            huff_table_dc: dc_table,
            huff_table_ac: ac_table,
            bit_writer: bit_writer::BitWriter::new(),
            frame_id: 0,
        })
    } 


    pub fn convert_rgbto_yuv_threaded(&mut self, pixels: &[u8])
    {        
        let mut y: Vec<u8> = vec![0u8;RESULTING_RESOLUTION as usize];
        let mut cb: Vec<u8> = vec![0u8;RESULTING_RESOLUTION as usize];
        let mut cr: Vec<u8> = vec![0u8;RESULTING_RESOLUTION as usize];

        let width_difference:f32 = self.width as f32 / RESULTING_WIDTH as f32;
        let height_difference:f32 = self.height as f32 / RESULTING_HEIGHT as f32;
        
        let mut block_length = 2;

        if self.width == RESULTING_WIDTH as u32{
            block_length = 1;
        }

        
        y
        .par_iter_mut()
        .zip(cb.par_iter_mut())
        .zip(cr.par_iter_mut())
        .enumerate()
        .for_each(|(index,((y,cb),cr))|{
            let x_pos = index % RESULTING_WIDTH as usize;
            let y_pos = index / RESULTING_WIDTH as usize;

            let dest_x = (x_pos as f32 * width_difference);
            let dest_y = (y_pos as f32 * height_difference); 
            let dest_pos = dest_x as u32 + dest_y as u32 * self.width;

            let mut r_average  = 0;
            let mut g_average = 0;
            let mut b_average = 0;

            for i in 0..block_length{
                for j in 0..block_length{
                    let pos =((dest_pos + j + (i * self.width)) * 4) as usize;
                    r_average += pixels[pos] as u32;
                    g_average += pixels[pos + 1] as u32;
                    b_average += pixels[pos + 2] as u32;
                }
            }
            let r = (r_average / (block_length * block_length)) as u8;
            let b = (b_average / (block_length * block_length)) as u8;
            let g = (g_average / (block_length * block_length)) as u8;

            *y = Self::convert_y(r, g, b);
            *cr = Self::convert_cr(r, g, b);
            *cb = Self::convert_cb(r, g, b);
        });

        let mut cb_reduced: Vec<u8> = vec![0u8;RESULTING_RESOLUTION as usize / 4];
        let mut cr_reduced: Vec<u8> = vec![0u8;RESULTING_RESOLUTION as usize / 4];

        cb_reduced
        .par_iter_mut()
        .zip(cr_reduced.par_iter_mut())
        .enumerate()
        .for_each(|(index,(cb_reduced,cr_reduced))|{
            let x_pos = index % (RESULTING_WIDTH as usize / 2);
            let y_pos = index / (RESULTING_WIDTH as usize / 2);
            let src_index = x_pos * 2 + y_pos * 2 * RESULTING_WIDTH as usize;
            let mut cr_average:u32 = 0;
            let mut cb_average:u32 = 0;
            for i in 0..2 as usize{
                for j in 0..2 as usize{
                    let pos = src_index + j + (i * RESULTING_WIDTH as usize);
                    cb_average += cb[pos] as u32; 
                    cr_average += cr[pos] as u32;
                }
            }

            *cb_reduced = (cb_average / 4) as u8;
            *cr_reduced = (cr_average / 4) as u8;

        });
        


        self.ycbcr = (y,cb_reduced,cr_reduced);
    }



    fn convert_y(r:u8, g:u8, b:u8) -> u8{
        let y = 0.299  * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        y as u8
    }

    fn convert_cb(r:u8, g:u8, b:u8) -> u8 {
        (-0.168736*r as f32 - 0.331264*g as f32 + 0.5*b as f32 + 128.0) as u8
    }

    fn convert_cr(r:u8, g:u8, b:u8) -> u8 {
        (0.5*r as f32 - 0.418688*g as f32 - 0.081312*b as f32 + 128.0) as u8
    }


    //man müsste noch mehr reihen also horizontale werte einfügen dar sonst nicht durch 8 teilbar 
    //und somit nicht in blöcke verarbeitbar man müsste auf 544 Reihen ohne extras sind es 540
    pub(crate) fn linear_to_block_cb_cr(&self,cb:&Vec<u8>, cr:&Vec<u8>)->(Vec<u8>,Vec<u8>){
    //Schlechte implementierung vielleicht wennman hier einfach 255/2 wert reingibt und len des vecs vergößert für extra reihen
        let mut cb_block: Vec<u8> = vec![0; cb.len() + 4 * (1920 / 2)];
        let mut cr_block: Vec<u8> = vec![0; cb.len() + 4 * (1920 / 2)];

        let decreased_width = RESULTING_WIDTH as usize / 2;
        let decreased_height = RESULTING_HEIGHT as usize / 2;


        let blocks_per_row = decreased_width / 8;

        cb_block
        .par_iter_mut()
        .zip(cr_block.par_iter_mut())
        .enumerate()
        .for_each(|(index, (cb_block, cr_block))|{
            let block_index = index / 64;
            let local_index = index % 64;

            let local_x = local_index % 8;
            let local_y = local_index / 8;

            let block_x = (block_index % blocks_per_row) * 8;
            let block_y = (block_index / blocks_per_row) *8;

            let src_x = (block_x + local_x).min(decreased_width - 1);
            let src_y = (block_y + local_y).min(decreased_height - 1);

            let source_index = src_y * decreased_width + src_x;

            *cb_block = cb[source_index];
            *cr_block = cr[source_index];

        });


        (cb_block, cr_block)
    }

    pub(crate) fn linear_block_fast(&self,pixels:& Vec<u8>) -> Vec<u8> {
        let block_size = 8;
        let block_area = block_size * block_size;
        let blocks_x = RESULTING_WIDTH as usize / block_size;
        let blocks_y = RESULTING_HEIGHT as usize / block_size;

        let mut vec_block: Vec<u8> = vec![0u8; RESULTING_RESOLUTION as usize];

        for global_y in 0..blocks_y{
            for global_x in 0..blocks_x{
                for local_y in 0..block_size{
                    for local_x in 0..block_size{
                        let y = global_y * block_size + local_y;
                        let x = global_x * block_size + local_x;

                        let linear_index = x + y * RESULTING_WIDTH as usize;
                        let block_index = (global_y * blocks_x + global_x) * block_area + local_y * block_size + local_x;

                        vec_block[block_index] = pixels[linear_index];
                    }
                }
            }
        }

        vec_block
    }

    pub(crate) fn linear_block_fast_crcb(&self,cr:& Vec<u8>,cb:& Vec<u8>) -> (Vec<u8>,Vec<u8>) {
        let block_size = 8;
        let block_area = block_size * block_size;
        let blocks_x = (RESULTING_WIDTH / 2) / block_size;
        let blocks_y = (RESULTING_HEIGHT / 2) / block_size;

        let mut cr_block: Vec<u8> = vec![0u8; cr.len()];
        let mut cb_block: Vec<u8> = vec![0u8; cr.len()];

        for global_y in 0..blocks_y{
            for global_x in 0..blocks_x{
                for local_y in 0..block_size{
                    for local_x in 0..block_size{
                        let y = global_y * block_size + local_y;
                        let x = global_x * block_size + local_x;

                        let linear_index = x + y * RESULTING_WIDTH / 2;
                        let block_index = (global_y * blocks_x + global_x) * block_area + local_y * block_size + local_x;

                        cr_block[block_index as usize] = cr[linear_index as usize];
                        cb_block[block_index as usize] = cb[linear_index as usize];
                    }
                }
            }
        }

        cr_block.resize(cb.len() + 4 * (1920 / 2), 128);
        cb_block.resize(cb.len() + 4 * (1920 / 2), 128);

        (cr_block,cb_block)
    }
    


    pub(crate) fn block_linear_fast(&self, pixels:& Vec<u8>) -> Vec<u8>{
        let block_size = 8;
        let block_area = block_size * block_size;
        let blocks_x = RESULTING_WIDTH as usize / block_size;
        let blocks_y = RESULTING_HEIGHT as usize / block_size;

        let mut linear_block: Vec<u8> = vec![0u8; RESULTING_RESOLUTION as usize];

        for global_y in 0..blocks_y{
            for global_x in 0..blocks_x{
                for local_y in 0..block_size{
                    for local_x in 0..block_size{
                        let y = global_y * block_size + local_y;
                        let x = global_x * block_size + local_x;

                        let linear_index = x + y * RESULTING_WIDTH as usize;
                        let block_index = (global_y * blocks_x + global_x) * block_area + local_y * block_size + local_x;

                        linear_block[linear_index] = pixels[block_index];
                    }
                }
            }
        }

        linear_block
    }

    pub(crate) fn block_linear_fast_crcb(&self,cr:& Vec<u8>,cb:& Vec<u8>) -> (Vec<u8>,Vec<u8>) {
        let block_size = 8;
        let block_area = block_size * block_size;
        let blocks_x = (RESULTING_WIDTH / 2) / block_size;
        let blocks_y = (RESULTING_HEIGHT / 2) / block_size;

        let mut cr_block: Vec<u8> = vec![0u8; cr.len()];
        let mut cb_block: Vec<u8> = vec![0u8; cr.len()];

        for global_y in 0..blocks_y{
            for global_x in 0..blocks_x{
                for local_y in 0..block_size{
                    for local_x in 0..block_size{
                        let y = global_y * block_size + local_y;
                        let x = global_x * block_size + local_x;

                        let linear_index = x + y * RESULTING_WIDTH / 2;
                        let block_index = (global_y * blocks_x + global_x) * block_area + local_y * block_size + local_x;

                        cr_block[linear_index as usize] = cr[block_index as usize];
                        cb_block[linear_index as usize] = cb[block_index as usize];
                    }
                }
            }
        }

        cr_block.resize(RESULTING_RESOLUTION as usize / 4, 0);
        cb_block.resize(RESULTING_RESOLUTION as usize / 4, 0);

        (cr_block,cb_block)
    }




    //müsste man auch für cr und cb implementieren
    pub(crate) fn fast_dct(&self,pixels:& Vec<u8>) -> Vec<i16>
    {
        let mut dct_vec:Vec<f32> = Vec::with_capacity(RESULTING_RESOLUTION as usize);

        //Könnte man mit par iter chunk warscheinlich um eineiges schneller machen threaded
        for block in (0..RESULTING_RESOLUTION as usize).step_by(64){
            let mut block_f32 = [0.0f32; 64]; 
            for (i, &p) in pixels[block..block+64].iter().enumerate() {
                block_f32[i] = p as f32;
            }
            fast_dct::dct_quant(&mut block_f32);
            dct_vec.extend_from_slice(&block_f32);
        }

        dct_vec.iter().map(|x|{ (*x).round() as i16}).collect()
    }

    pub(crate) fn fast_dct_crcb(&self,cr:& Vec<u8>, cb: & Vec<u8>) -> (Vec<i16>,Vec<i16>)
    {
        let mut dct_cr:Vec<f32> = Vec::with_capacity(cr.len() as usize);
        let mut dct_cb:Vec<f32> = Vec::with_capacity(cr.len() as usize);

        //Könnte man mit par iter chunk warscheinlich um eineiges schneller machen threaded
        for block in (0..cr.len()).step_by(64){
            let mut cr_block_f32 = [0.0f32; 64]; 
            let mut cb_block_f32 = [0.0f32; 64]; 

            for (i,(&cr,&cb)) in cr[block..block+64].iter().zip(cb[block..block+64].iter()).enumerate() {
                cr_block_f32[i] = cr as f32;
                cb_block_f32[i] = cb as f32;
            }

            fast_dct::dct_quant(&mut cr_block_f32);
            fast_dct::dct_quant(&mut cb_block_f32);
            
            dct_cr.extend_from_slice(&cr_block_f32);
            dct_cb.extend_from_slice(&cb_block_f32);
        }

        (dct_cr.iter().map(|x|{ (*x).round() as i16}).collect(),dct_cb.iter().map(|x|{ (*x).round() as i16}).collect())
    }

    pub(crate) fn inverse_fast_dct(&self,pixels:& Vec<i16>) ->Vec<u8>
    {
        let mut dct_vec:Vec<f32> = Vec::with_capacity(RESULTING_RESOLUTION as usize);
        let pixels:Vec<f32> = pixels.iter().map(|x|{ *x as f32}).collect();


        //hier durch threading auch schneller möglich
        for block in (0..RESULTING_RESOLUTION as usize).step_by(64){
            let mut block_f32: Vec<f32> = pixels[block..block + 64].to_vec();
            fast_dct::inverse_dct_quant(&mut block_f32);
            dct_vec.extend_from_slice(&block_f32);
        }

        let yuv_dct: Vec<u8> = dct_vec.iter()
            .map(|&f| f.round() as u8)
            .collect();

        yuv_dct
    }

    pub(crate) fn inverse_fast_dct_crcb(&self,cr:& Vec<i16>, cb: & Vec<i16>) -> (Vec<u8>,Vec<u8>)
    {
        let mut dct_cr:Vec<f32> = Vec::with_capacity(cr.len() as usize);
        let mut dct_cb:Vec<f32> = Vec::with_capacity(cb.len() as usize);

        let cr_f32:Vec<f32> = cr.iter().map(|&x|{ x as f32}).collect();
        let cb_f32:Vec<f32> = cb.iter().map(|&x|{ x as f32}).collect();

        //Könnte man mit par iter chunk warscheinlich um eineiges schneller machen threaded
        for block in (0..cr.len()).step_by(64){
            let mut cr_block_f32 = cr_f32[block..block+64].to_vec();
            let mut cb_block_f32 = cb_f32[block..block+64].to_vec(); 

            fast_dct::inverse_dct_quant(&mut cr_block_f32);
            fast_dct::inverse_dct_quant(&mut cb_block_f32);
            
            dct_cr.extend_from_slice(&cr_block_f32);
            dct_cb.extend_from_slice(&cb_block_f32);
        }

        let cr: Vec<u8> = dct_cr.iter()
            .map(|&f| f.round() as u8)
            .collect();

        let cb: Vec<u8> = dct_cb.iter()
            .map(|&f| f.round() as u8)
            .collect();

        (cr,cb)
    }
    


    pub(crate) fn zigzag(&self,vector:&Vec<i16>) ->Vec<i16>{
        let mut zigzag = vec![0i16;vector.len()];
        for i in 0..vector.len(){
            zigzag[i] = vector[((i / 64) * 64) + ZIGZAG_ORDER[i%64]];
        }
        zigzag
    }

    pub(crate) fn inverse_zigzag(&self,vector:&Vec<i16>) ->Vec<i16>{
        let mut zigzag = vec![0i16;vector.len()];
        for i in 0..vector.len(){
            zigzag[((i / 64) * 64) + ZIGZAG_ORDER[i%64]] = vector[i];
        }
        zigzag
    }

    pub(crate) fn rle_encoding(&self,vector:&Vec<i16>) ->Vec<(usize, i16)>
    {
        let mut rle:Vec<(usize,i16)> = Vec::new();
        let mut zero_counter = 0;
        

        for i in 0..vector.len(){
            if i % 64 == 0{
                zero_counter = 0;
                rle.push((17,vector[i]));
                continue;
            }

            if i % 64 == 63{
                if vector[i] == 0{
                    while *rle.last().unwrap() == (15,0) {
                        rle.pop();
                    }
                    rle.push((0,0));
                    zero_counter = 0;
                }else{
                    rle.push((zero_counter, vector[i]));
                    zero_counter = 0;
                }
                continue;
            }
            //wenn vector i 16 al 0 ist muss 16 o in tuppel und fals dann aber doch ende kommt muss es wieder gepoppt werden und nur (0,0) reingeschireben werden also schreibt man und wenn es eob ist while schleife mit allen 0 davo weg
            if vector[i] == 0{
                zero_counter+= 1;

                if zero_counter == 16{
                    zero_counter = 0;
                    rle.push((15,0));
                }
            }else{
                rle.push((zero_counter, vector[i]));
                zero_counter = 0;
            }
        }

        rle
    }


    pub(crate) fn rle_decoding(&self,vector:&Vec<(usize, i16)>) -> Vec<i16>{
        let mut pixel:Vec<i16> = Vec::new();

        for rle in vector{
            if rle.0 == 17{
                pixel.push(rle.1);
            }else if *rle == (0,0) {
                pixel.resize(pixel.len() + 64 - (pixel.len() % 64),0);
            }else{
                pixel.resize(pixel.len() + rle.0,0);
                pixel.push(rle.1);
            }
        }

        pixel
    }


    pub(crate) fn send_packets(&mut self, y_rle:&Vec<(usize, i16)>, cb_rle:&Vec<(usize, i16)>, cr_rle:&Vec<(usize, i16)>){
        self.frame_id = self.frame_id.wrapping_add(1);
        let mut y_counter = 0;
        let mut cb_counter = 0;
        let mut old_cb_counter = 0;
        let mut cr_counter  = 0;
        let mut old_cr_counter = 0;
        let mut old_y_counter = 0;
        let mut over_max_size = false;
        let mut block_counter:u32 = 0;
        let mut fragment_id = 0;


            println!("Y blocks: {}, Cb blocks: {}, Cr blocks: {}", 
        y_rle.len(), cb_rle.len(), cr_rle.len());

            println!("RLE: {:?}", &y_rle[..8]);
            println!("cbRLE: {:?}", &cb_rle[..8]);
            println!("crRLE: {:?}", &cr_rle[..8]);

        while y_counter < y_rle.len() {
            let slice_end = self.bit_writer.getlen();
            old_y_counter = y_counter;
            old_cb_counter = cb_counter;
            old_cr_counter = cr_counter;


            let (counter, over) = self.blocks_to_bits(&y_rle, y_counter, 4);
            y_counter = counter;
            over_max_size |= over;
            //println!("COUNTER: {}", counter);
            let (counter, over) = self.blocks_to_bits(&cb_rle, cb_counter, 1);
            cb_counter = counter;
            over_max_size |= over;
            let (counter, over) = self.blocks_to_bits(&cr_rle, cr_counter, 1);
            cr_counter = counter;
            over_max_size |= over;


            if over_max_size {
                //println!("END");
                over_max_size = false;
                y_counter = old_y_counter;
                cb_counter = old_cb_counter;
                cr_counter = old_cr_counter;
                let packet = self.bit_writer.get_buffer()[0..slice_end].to_vec();
                

                self.udp_send(fragment_id, block_counter as u16,packet);
              
                fragment_id += 1;
                block_counter = 0;
            }else{
                block_counter += 6;
            }
        }

        if self.bit_writer.getlen() > 0{
            let packet = self.bit_writer.get_buffer().to_vec();
            self.udp_send(fragment_id, block_counter as u16,packet);
        }
        println!("FRAGMENTS: {}",fragment_id);
  
    }

   
    fn blocks_to_bits(&mut self, rle:&Vec<(usize,i16)>, counter:usize, block_count: usize) -> (usize,bool){
        let mut dc_counter = 0;
        let mut huff_run;
        let mut huff;
        let mut over_max_packet_size = false;
        let mut counter = counter;

        while counter < rle.len(){
            huff_run = rle[counter].0;

            if dc_counter == block_count && huff_run == 17{
                break;
            }

            //println!("BB: {}", block_count);
            //print!("{:?}, ", rle[counter]);
            if huff_run == 17{
                dc_counter += 1;
                huff = self.rle_to_bits_dc(rle[counter]);
            }else{
                huff = self.rle_to_bits_ac(rle[counter]);
            }
            over_max_packet_size |= self.bit_writer.write_bits(huff.0 as u64, huff.1);
            counter += 1;
        }

        (counter, over_max_packet_size)
    }



    fn rle_to_bits_ac(&self, rle:(usize,i16)) -> (u32, u8){    
        let cat = huffcode::categorie(rle.1 as i16);
        let huff = self.huff_table_ac.get(&(rle.0 as u8, cat as u8)).unwrap(); 
        let mut y = (huff.code as u32) << cat;
        if rle.1 > 0{
            y = y | rle.1 as u32;
        }else{
            y = y | (rle.1 + (1i16 << cat) - 1) as u32;
        }

        (y,huff.len + cat as u8)
    }

    fn rle_to_bits_dc(&self, rle:(usize,i16)) -> (u32, u8){    
        let cat = huffcode::categorie(rle.1 as i16);
        let huff = self.huff_table_dc.get(&(cat as u8)).unwrap(); 
        let mut y = (huff.code as u32) << cat;
        if rle.1 > 0{
            y = y | rle.1 as u32;
        }else{
            y = y | (rle.1 + (1i16 << cat) - 1) as u32;
        }

        (y,huff.len + cat as u8)
    }


    fn udp_send(&self, fragment_id:u8,block_count:u16, data:Vec<u8>){
        let mut buffer:Vec<u8> = Vec::new();
        buffer.push(self.frame_id);
        buffer.push(fragment_id);
        buffer.push((block_count >> 8) as u8);
        buffer.push(block_count as u8);
        buffer.extend_from_slice(&data);
        //self.socket.send_to(&buffer,IP_ADDR).expect("FAiled udp");
    }



    //lansgamer als serieller dct func
    //mehr testen nötig um schnelleren ablauf zu gewährleisten in kleiner teile aufteilen
    pub(crate) fn threaded_dct(&self,pixels:&mut Vec<u8>)
    {
        let mut dct_vec:Vec<f32> = vec![0f32;pixels.len()];

        dct_vec.par_chunks_mut(64)
            .enumerate()
            .for_each(|(index, chunk)|{
                let mut block_f32 = [0.0f32; 64];  // liegt auf dem Stack, kein malloc/free
                for (i, &p) in pixels[index * 64..index * 64 + 64].iter().enumerate() {
                    block_f32[i] = p as f32;
                }

                fast_dct::transform_horizontal(&mut block_f32);
                fast_dct::transform_vertical(&mut block_f32);
                fast_dct::dct_matrix(&mut block_f32);

                chunk.copy_from_slice(&block_f32);
            });

        //self.inverse_fast_dct(&mut dct_vec);
    }

   


    pub fn send_ycrcb(&self)
    {
        self.sender.send(self.ycbcr.clone()).ok();
    }

}