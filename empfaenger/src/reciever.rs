use std::{collections::{BTreeMap, HashMap}, fmt::Error, net::UdpSocket, sync::mpsc::SyncSender};

use crate::{capture::Capture, huffcode::{self, InvertedHuf}};

pub struct Reciever{
    frame_map: HashMap<u16,BTreeMap<u8,Vec<(usize,i16)>>>,
    lut_ac: Vec<InvertedHuf>,
    lut_dc: Vec<InvertedHuf>,
    tx: SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>,
}


impl Reciever{

    pub fn new(tx: SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>) -> Result<Self,Error>{
        let dc_table= huffcode::jpeg_dc_luminance_table();
        let ac_table = huffcode::jpeg_ac_luminance_table();

        Ok( 
            Reciever { 
                frame_map: HashMap::new(),
                lut_dc: huffcode::lut_dc(&dc_table),
                lut_ac: huffcode::lut_ac(&ac_table),
                tx, 
            }
        )
    }

    fn packets_to_frame(&mut self,frame_id:u16, fragment_id:u8, data:&Vec<u8>, block_count:u16){
        let rle: Vec<(usize, i16)> = self.huff_decoding_new(data, block_count);
        
        //println!("RLE: {:?}\n", &rle[rle.len()- 100..rle.len()]);

        if self.frame_map.contains_key(&frame_id){
            let fragment_map: &mut BTreeMap<u8, Vec<(usize, i16)>> = self.frame_map.get_mut(&frame_id).unwrap();
            fragment_map.insert(fragment_id, rle);
        }else{
            if frame_id > 4 && fragment_id > 80{
                self.frame_from_fragments(frame_id - 4);
                self.frame_map.remove(&(frame_id - 4));
            }
            let mut map:BTreeMap<u8,Vec<(usize,i16)>> = BTreeMap::new();
            map.insert(fragment_id, rle);
            self.frame_map.insert(frame_id, map);
        }


    }

    fn frame_from_fragments(&mut self, frame_id:u16){
        let mut rle: Vec<(usize, i16)> = Vec::new();

        let fragment_map= self.frame_map.get(&frame_id).unwrap();

        for (_, value) in fragment_map{
            rle.extend_from_slice(value);
        }

        let mut dc_counter = 0;
        let mut y_rle = Vec::new();
        let mut cb_rle = Vec::new();
        let mut cr_rle = Vec::new();


        for (run, value) in rle{
            if run == 17 {
                dc_counter += 1;
                
                if dc_counter > 6 {
                    dc_counter = 1;
                }
            }

            if dc_counter <= 4 {
                y_rle.push((run, value));
            } else if dc_counter == 5 {
                cb_rle.push((run, value));
            } else if dc_counter == 6 {
                cr_rle.push((run, value));
            }
        }

       println!("Y blocks: {}, Cb blocks: {}, Cr blocks: {}", 
        y_rle.len(), cb_rle.len(), cr_rle.len());
        self.to_y_cb_cr(y_rle, cb_rle, cr_rle);
    }




    fn decode_coefficient(&self, bits: u32, cat: u8) -> i16 {
        if cat == 0 { return 0; }
        
        let threshold = 1i32 << (cat - 1); 
        
        if bits as i32 >= threshold {
            bits as i16  
        } else {
            (bits as i32 - (1i32 << cat) + 1) as i16  
        }
    }

    fn huff_decoding_new(&mut self, data:&Vec<u8>, block_count:u16) -> Vec<(usize, i16)>{
        let mut rle:Vec<(usize, i16)> = Vec::new();
        let mut huff_inverted:InvertedHuf;


        let mut buffer:u64 = u64::from_be_bytes(data[0..8].try_into().unwrap());
        let mut buffer_counter = 8;
        
        let mut run_counter = 0;
        let mut shift:u32 = 0;

        let mut dc_counter = 0;

        loop{
            let buffer_shifted = buffer.unbounded_shl(shift as u32);

            if run_counter == 0{
                huff_inverted = self.lut_dc[(buffer_shifted >> (64 - 16)) as usize];
                run_counter += 1;
            }else{
                huff_inverted = self.lut_ac[(buffer_shifted >> (64 - 16)) as usize];
                run_counter += huff_inverted.run + 1;
                if huff_inverted.run == 0 && huff_inverted.cat == 0{
                    run_counter = 64;
                }
            }   

            let value = self.decode_coefficient(((buffer_shifted << huff_inverted.huf_len).unbounded_shr(64 - huff_inverted.cat as u32)) as u32, huff_inverted.cat);

            //println!("{:?} \nValue: {} \n Buffer_shifted: {:064b}", huff_inverted, value, buffer_shifted);

            rle.push((huff_inverted.run as usize, value));
            

            shift = huff_inverted.cat as u32 + huff_inverted.huf_len as u32 + shift;

            for _ in 0..shift/8{
                buffer = buffer << 8;
                if buffer_counter < data.len(){
                    buffer |= data[buffer_counter] as u64;
                    buffer_counter += 1;
                }
                shift -= 8;
            }
            
            //if dc_counter > 3{while true {}}

            //println!("dc_counter: {}", dc_counter);
            if run_counter >= 64 {
                run_counter = 0;
                dc_counter += 1;
            }

            if dc_counter == block_count{break}

        }


        rle
    }


    fn to_y_cb_cr(&self, y_rle: Vec<(usize, i16)>, cb_rle: Vec<(usize, i16)>, cr_rle: Vec<(usize, i16)>){
        let capture = Capture::new(self.tx.clone()).unwrap();

        let werte = capture.rle_decoding(&y_rle);
        let wertecr = capture.rle_decoding(&cr_rle);
        let wertecb = capture.rle_decoding(&cb_rle);

        let mut unzigzag = capture.inverse_zigzag(&werte);
        let mut unzigzagcr = capture.inverse_zigzag(&wertecr);
        let mut unzigzagcb = capture.inverse_zigzag(&wertecb);
    
        let target_y  = 1920 * 1080; 
        unzigzag.resize(target_y, 0);
        
        let target_cb = (1920 / 2) * (1080 / 2);


        let y_blocks = capture.inverse_fast_dct(&unzigzag);
        unzigzagcr.resize(target_cb, 0);
        unzigzagcb.resize(target_cb, 0);

        let (cr,cb)= capture.inverse_fast_dct_crcb(&unzigzagcr, &unzigzagcb);

        let y_linear = capture.block_linear_fast(&y_blocks);
        let (cr,cb)= capture.block_linear_fast_crcb(&cr, &cb);

        self.tx.send((y_linear,cr,cb)).unwrap();
    }



    pub fn reciever(&mut self)-> std::io::Result<()>{
        let socket = UdpSocket::bind("127.0.0.1:26262")?;
        
        let mut allblocks = 0;
        let mut old_frame_id = 0;
        let mut counter = 0;
        let mut old_fragment_id = 0;
        loop{


            let mut buf = [0; 1500];
            let (amt, _) = socket.recv_from(&mut buf)?;

            counter += 1;

            let frame_id = ((buf[0] as u16) << 8)  |(buf[1] as u16);
            let fragment_id = buf[2];
            let block_count= ((buf[3] as u16) << 8 )| buf[4] as u16;
            
            let data = &buf[5..amt].to_vec();

            if old_frame_id != frame_id{
                println!("Frame_id: {}  | Fragment_id: {} | block_count: {} | length: {} | gesBlocks: {} | counterFragments: {}",frame_id, old_fragment_id,block_count, amt, allblocks, counter);
                allblocks = 0;
                counter = 0;
            }

            allblocks += block_count;

         //   println!("Frame_id: {}  | Fragment_id: {} | block_count: {} | length: {} | gesBlocks: {}",frame_id, fragment_id,block_count, amt, allblocks);
            
                //println!("data: {:08b}", data[0]);
                //let mut buffer:u64 = u64::from_be_bytes(data[0..8].try_into().unwrap());
                //println!("datau64: {:08b}", buffer);
              //  println!();
/*
                for bits in data{
                    print!("{:08b}", bits);
                }

 */

              //  println!();
                self.packets_to_frame(frame_id, fragment_id, data, block_count);
                //let x = self.huff_decoding_new(data, block_count);
                //println!("RLE: {:?}", x);
               old_fragment_id =  fragment_id;
            old_frame_id = frame_id;
        }

    }

}
//1101100110101101100110101101100110101101100110101110111101011101110000100110001100111
//1101100110101101100110101101100110101101100110101110111101011101
//1010110110011010110110011010110110011010111011110101110111000010