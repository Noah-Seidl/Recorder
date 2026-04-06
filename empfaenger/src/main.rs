use std::net::UdpSocket;


#[derive(Debug)]
struct Data<'a>{
    len:u16,
    fragment_id: u8,
    frame_id: u8,
    data:&'a [u8],
}


fn main() -> std::io::Result<()> {
    loop{
        let socket = UdpSocket::bind("127.0.0.1:1234")?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 1400];
        let (amt, src) = socket.recv_from(&mut buf)?;

        for (index, &buffer) in buf.iter().enumerate(){
            println!("BYTES BUF{}:\t{:08b}", index, buffer);
        }


        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        //let buf = &mut buf[..amt];
        //buf.reverse();
        //socket.send_to(buf, &src)?;
    } // the socket is closed here
}

/*
// Pseudocode zum Füllen der LUT für EINEN bekannten Code
uint16_t code = 0b101;      // Der Huffman-Code
uint8_t code_len = 3;       // Länge des Codes
uint8_t symbol = 0x15;      // Das dazugehörige Symbol (Run/Size)

// Wir shiften den Code an die obersten Bits, um den Startindex zu finden
uint16_t start_index = code << (16 - code_len);
// Die Anzahl der zu füllenden Einträge ist 2^(16 - code_len)
uint16_t num_entries = 1 << (16 - code_len);

for (int i = 0; i < num_entries; i++) {
    huffman_lut[start_index + i].symbol = symbol;
    huffman_lut[start_index + i].length = code_len;
}

*/