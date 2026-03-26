use std::{sync::mpsc, time::Instant, vec};

use rayon::{iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator}, slice::ParallelSlice};

use crate::{fastDct};

const BLOCK_SIZE: u32 = 8;

const RESULTING_WIDTH:u32 = 1920;
const RESULTING_HEIGHT:u32 = 1080;
const RESULTING_RESOLUTION:u32 = RESULTING_HEIGHT * RESULTING_WIDTH;
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

pub(crate) struct Capture {
    pub(crate) start: Instant,
    pub(crate) counter: i128,
    pub(crate) ycbcr:(Vec<u8>,Vec<u8>,Vec<u8>),
    pub(crate) width: u32,
    pub(crate) height:u32,
    sender: mpsc::SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>,
    pub(crate) second_last : u64,
}


impl Capture{

    pub(crate) fn new(width:u32, height: u32, sender:mpsc::SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
       

       
        Ok(Self {
            start: Instant::now(),
            counter: -1,
            width,
            height,
            sender,
            second_last: 0,
            ycbcr: (Vec::new(), Vec::new(), Vec::new()) ,
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

        if self.width == RESULTING_WIDTH {
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




    //müsste man auch für cr und cb implementieren
    pub(crate) fn fast_dct(&self,pixels:&mut Vec<u8>)
    {
        let mut dct_vec:Vec<f32> = Vec::with_capacity(RESULTING_RESOLUTION as usize);

        println!("Yuv werte REAL:");
        for i in (0..64).step_by(8){
            println!("yuv: Werte: {:?}", &pixels[i..i+8]);
        }

        //Könnte man mit par iter chunk warscheinlich um eineiges schneller machen threaded
        for block in (0..RESULTING_RESOLUTION as usize).step_by(64){
            let mut block_f32: Vec<f32> = pixels[block..block + 64]
                .iter()
                .map(|&p| p as f32)
                .collect();
            fastDct::transform_horizontal(&mut block_f32);
            fastDct::transform_vertical(&mut block_f32);
            fastDct::dct_matrix(&mut block_f32);
            dct_vec.extend_from_slice(&block_f32);
        }



        println!("Yuv werte REAL:");
        for i in (0..64).step_by(8){
            println!("yuv: Werte: {:?}", &dct_vec[i..i+8]);
        }

        self.inverse_fast_dct(&mut dct_vec);

    }



    pub(crate) fn inverse_fast_dct(&self,pixels:&mut Vec<f32>)
    {
        let mut dct_vec:Vec<f32> = Vec::with_capacity(RESULTING_RESOLUTION as usize);

        //hier durch threading auch schneller möglich
        for block in (0..RESULTING_RESOLUTION as usize).step_by(64){
            let mut block_f32: Vec<f32> = pixels[block..block + 64].to_vec();
            fastDct::inverse_horizontal(&mut block_f32);
            fastDct::inverse_vertical(&mut block_f32);
            fastDct::inverse_dct_matrix(&mut block_f32);
            dct_vec.extend_from_slice(&block_f32);
        }

        let yuv_dct: Vec<u8> = dct_vec.iter()
            .map(|&f| f.round() as u8)
            .collect();

        println!("Inverse DCT:");
        for i in (0..64).step_by(8){
            println!("yuv: Werte: {:?}", &yuv_dct[i..i+8]);
        }
    }

    //schneller dct mit par iter:


    pub(crate) fn threaded_dct(&self,pixels:&mut Vec<u8>)
    {
        let mut dct_vec:Vec<f32> = vec![0f32;pixels.len()];
        println!("Yuv werte REAL:");
        for i in (0..64).step_by(8){
            println!("yuv: Werte: {:?}", &pixels[i..i+8]);
        }

        dct_vec.par_chunks(64)
            .enumerate()
            .for_each(|(index, chunk)|{
                let mut block_f32: Vec<f32> = pixels[index * 64..index * 64 + 64]
                .iter()
                .map(|&p| p as f32)
                .collect();

                fastDct::transform_horizontal(&mut block_f32);
                fastDct::transform_vertical(&mut block_f32);
                fastDct::dct_matrix(&mut block_f32);

                chunk.copy_from_slice(&block_f32);
            });
     

        println!("Yuv werte REAL:");
        for i in (0..64).step_by(8){
            println!("yuv: Werte: {:?}", &dct_vec[i..i+8]);
        }

        self.inverse_fast_dct(&mut dct_vec);

    }






    pub fn send_ycrcb(&self)
    {
        self.sender.send(self.ycbcr.clone()).ok();
    }


    ////////////////////////////////////////////////////////////
    //Langsame Algos die nichtmehr grbraucht werden vl cleanup
    ////////////////////////////////////////////////////////////



    pub fn convert_rgbto_yuv(&mut self, pixels: &[u8])
    {
        let mut y:Vec<u8> = Vec::with_capacity(RESULTING_RESOLUTION as usize);
        let mut cb:Vec<u8> = Vec::with_capacity(RESULTING_RESOLUTION as usize);
        let mut cr:Vec<u8> = Vec::with_capacity(RESULTING_RESOLUTION as usize);

        let mut block_length = 2;

        if self.width == RESULTING_WIDTH {
            block_length = 1;
        }
        
        let width_difference:f32 = self.width as f32 / RESULTING_WIDTH as f32;
        let height_difference:f32 = self.height as f32 / RESULTING_HEIGHT as f32;

        let mut pixel_index1 = 0;
        let mut pixel_index2 = 0;

        let mut x_pos = 0;
        let mut y_pos = 0; 
        let mut dest_x:f32 = 0.0;
        let mut dest_y:f32 = 0.0;

        let mut r_average:u32 = 0;
        let mut g_average:u32 = 0;
        let mut b_average:u32 = 0;

        let mut r: u8;
        let mut g: u8;
        let mut b: u8;

        for index in 0..RESULTING_RESOLUTION{
            x_pos = index % RESULTING_WIDTH;
            y_pos = index / RESULTING_WIDTH;

            dest_x = (x_pos as f32 * width_difference);
            dest_y = (y_pos as f32 * height_difference);    
            pixel_index1 = dest_x as u32 + dest_y as u32 * self.width;
            for i in 0..block_length{
                for j in 0..block_length{
                    pixel_index2 =((pixel_index1 + j + (i * self.width)) * 4) as usize;
                    r_average += pixels[pixel_index2] as u32;
                    g_average += pixels[pixel_index2 + 1] as u32;
                    b_average += pixels[pixel_index2 + 2] as u32;
                }
            }
            r = (r_average / (block_length * block_length)) as u8;
            b = (b_average / (block_length * block_length)) as u8;
            g = (g_average / (block_length * block_length)) as u8;

            r_average = 0;
            b_average = 0;
            g_average = 0;

            y.push(Self::convert_y(r, g, b));
            
            if x_pos % 2 == 0 && y_pos % 2 == 0{
                cb.push(Self::convert_cb(r, g, b));
                cr.push(Self::convert_cr(r, g, b));
            }
        }


        println!("cr normal: {:?}", &cr[..64]);

        self.ycbcr = (y,cb,cr);
    }

  

    pub(crate) fn dct_transformation(&self)
    {
        let (y,cb,cr) = &self.ycbcr;
        println!("Y WeRTE VOR inverse:\n{:?}", &y[0..64]);

        let mut dct_y: Vec<f32> = vec![0.0; RESULTING_RESOLUTION as usize];

        for y_pos in (0..RESULTING_HEIGHT as usize).step_by(8){
            for x_pos in (0..RESULTING_WIDTH as usize).step_by(8){
                let pos_block = x_pos + y_pos * RESULTING_WIDTH as usize;
                
                for m in 0..8{
                    for n in 0..8{
                        let index = pos_block + n + m * RESULTING_WIDTH as usize;
                        dct_y[index] = self.dc_berechnung(&y, pos_block, m, n);
                    }
                }
            }
        }

        println!("DCT::");
        for i in 0..8{
            let index = i * RESULTING_WIDTH as usize;
            println!("DCT: CONVERSION:::: {:?}", &dct_y[index..index + 8]);
        }
    
    }

    fn dc_berechnung(&self,yuv:&Vec<u8>,pos_block:usize,m:usize,n:usize) -> f32{
        let mut sum = 0.0;
        for i in 0..8{
            for j in 0..8{
                let index = pos_block + j + i * RESULTING_WIDTH as usize;
                sum += yuv[index] as f32 * self.cos_berechnung(i, m) * self.cos_berechnung(j, n);
            }
        }

        (sum * self.c_func(m) * self.c_func(n)) / 4.0
    }

    fn dct_inverse_transformation(&self, dct:&Vec<f32>){
        let mut yuv: Vec<u8> = vec![0; RESULTING_RESOLUTION as usize];

        for y_pos in (0..RESULTING_HEIGHT as usize).step_by(8){
            for x_pos in (0..RESULTING_WIDTH as usize).step_by(8){
                let pos_block = x_pos + y_pos * RESULTING_WIDTH as usize;
                
                for m in 0..8{
                    for n in 0..8{
                        let index = pos_block + n + m * RESULTING_WIDTH as usize;
                        yuv[index] = self.dc_invers_berechnung(&dct, pos_block, m, n);
                    }
                }
            }
        }


        println!("Y WeRTE NACH inverse:\n{:?}", &yuv[0..64]);
    }


    fn dc_invers_berechnung(&self,dct:&Vec<f32>,pos_block:usize,i:usize,j:usize) -> u8{
        let mut sum = 0.0;
        for m in 0..8{
            for n in 0..8{
                let index = pos_block + n + m * RESULTING_WIDTH as usize;
                sum += self.c_func(m) * self.c_func(n) * dct[index] * self.cos_berechnung(i, m) * self.cos_berechnung(j, n);
            }
        }

        sum /= 4.0;
        f32::clamp(sum, 0.0, 255.0) as u8
    }

    fn cos_berechnung(&self, ij:usize,mn:usize) -> f32{
        f32::cos(((2.0 * ij as f32 + 1.0) * mn as f32 * PI) / 16.0)
    }

    fn c_func(&self,k:usize) -> f32{
        if k == 0{
            1.0 / f32::sqrt(2.0)
        }else{
            1.0
        }
    }


    fn quantization(&self, yuv_dct:Vec<f32>){
        let mut quant_y: Vec<i16> = vec![0; RESULTING_RESOLUTION as usize];

        
        for y_pos in (0..RESULTING_HEIGHT as usize).step_by(8){
            for x_pos in (0..RESULTING_WIDTH as usize).step_by(8){
                let pos_block = x_pos + y_pos * RESULTING_WIDTH as usize;
                
                for i in 0..8{
                    for j in 0..8{
                        let matrix_index = j + i * 8;
                        let yuv_index = pos_block + matrix_index;
                        quant_y[yuv_index] = self.quantization_matrix(yuv_dct[yuv_index], matrix_index);
                    }
                }
            }
        }


        println!("Quant::");
        for i in 0..8{
            let index = i * RESULTING_WIDTH as usize;
            println!("Quant::::: {:?}", &quant_y[index..index + 8]);
        }



    }

    fn quantization_matrix(&self, dct_wert:f32, index:usize)-> i16{
        f32::round(dct_wert / MATRIX[index]) as i16
    }


    //wird noch für schnellere dct trafo gebraucht
    pub(crate) fn linear_to_block_y(&self,yuv:&Vec<u8>)->Vec<u8>{
        let mut yuv_block: Vec<u8> = vec![0; yuv.len()];

        let blocks_per_row = RESULTING_WIDTH as usize / 8;

        yuv_block
        .par_iter_mut()
        .enumerate()
        .for_each(|(index, yuv_block)|{
            let block_index = index / 64;
            let local_index = index % 64;

            let local_x = local_index % 8;
            let local_y = local_index / 8;

            let block_x = (block_index % blocks_per_row) * 8;
            let block_y = (block_index / blocks_per_row) *8;

            let source_index = (block_y + local_y) * RESULTING_WIDTH as usize + block_x + local_x;

            *yuv_block = yuv[source_index];
        });


        yuv_block
    }






}