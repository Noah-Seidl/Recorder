struct BitWriter{
    buffer: Vec<u8>
}

const MAX_SIZE:usize = 1400;

impl BitWriter{
    pub(crate) fn new() -> Self {
        BitWriter { buffer: Vec::new() }
    }

    fn write_bits(&mut self, code:u32, len:u8)
    {



    }


}