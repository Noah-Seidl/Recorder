pub(crate) struct BitWriter{
    buffer: Vec<u8>,
    buffer_counter:u32,
    current:u8,
}

const MAX_SIZE:usize = 1400;

impl BitWriter{

    pub(crate) fn new() -> Self {
        BitWriter { buffer: Vec::new(), buffer_counter: 0, current: 0 }
    }

    pub(crate) fn write_bits(&mut self, code:u32, len:u8)
    {
        let mut newcode = code << 32 - len;
        println!("VERSCHOBEN: {:032b}", newcode);
        for i in (len as usize/8 + 1..4).rev(){
            self.buffer.push((newcode >> i * 8) as u8);
            self.current = (i as u8 * 8 - len);
        }
        self.current = 8 - self.current;


    }

    pub(crate) fn print_bits(&self){
        for buf in &self.buffer{
            print!("{:08b}|", buf);
        }
        println!("\nLänger: {}", self.current);
    }


}