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
        let newcode = code << (32 - len) - self.current;
        let len = len - self.current;
        println!("VERSCHOBEN123: {:032b} len: {}", newcode, len);
        for i in (len as usize/8 + 1..4).rev(){
            
            self.buffer.push((newcode >> i * 8) as u8);
            self.current = i as u8 * 8 - len;
            println!("current: {}", self.current);
            self.buffer_counter += 1;
        }
        self.current = 8 - self.current;
    }

    pub(crate) fn print_bits(&self){
        println!("BUFFER:");
        for buf in &self.buffer{
            print!("{:08b}|", buf);
        }
        println!("\nLänge: {}", self.current);
        println!("Size: {}", self.buffer_counter);

    }


}