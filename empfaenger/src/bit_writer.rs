#![allow(dead_code)]
pub(crate) struct BitWriter{
    buffer: Vec<u8>,
    buffer_counter:u32,
    current:u32,
}

const MAX_SIZE:usize = 1400;

impl BitWriter{

    pub(crate) fn new() -> Self {
        BitWriter { buffer: Vec::new(), buffer_counter: 0, current: 0 }
    }

    //woorking
    pub(crate) fn write_bits(&mut self, code:u64, len:u8) -> bool{
        let mut newcode = code << (64 - len as usize);
        let mut len = len; 

        while len > 0 {
            if self.current == 0{
                self.buffer.push((newcode >> 64 - 8) as u8);
                newcode = newcode << 8;
                if len < 8{
                    self.current = len as u32;
                }
                len = len.checked_sub(8).unwrap_or(0);
            }else{
                newcode = newcode >> self.current;
                let last = self.buffer.len() - 1;
                self.buffer[last] = self.buffer[last] | (newcode >> 64 - 8) as u8;
                
                if (self.current + (len as u32)) < 8{
                    self.current = self.current + len as u32;
                    len = 0;
                }else{
                    newcode = newcode << 8;
                    len = len - (8 - self.current as u8);
                    self.current = 0;
                }
            }
        }
        
        self.buffer.len() >= MAX_SIZE 
    }

    pub(crate) fn getlen(&self) -> usize{
        self.buffer.len()
    }


    pub(crate) fn get_buffer(& mut self) ->Vec<u8>{
        self.buffer_counter = 0;
        self.current = 0;
        std::mem::replace(&mut self.buffer, Vec::new())
    }



    pub(crate) fn print_bits(&self){
        println!("BUFFER:");
        for buf in &self.buffer{
            print!("{:08b}", buf);
        }
        println!("\nLänge: {}", self.current);
        println!("Size: {}", self.buffer_counter);
        
    }


}