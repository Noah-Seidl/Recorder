use std::{collections::HashMap, net::UdpSocket, sync::mpsc::SyncSender};

struct Reciever{
    frame_map: HashMap<u8,Frame>
}

struct Frame{
    data: Vec<u8>
    
}


impl Reciever{

    pub fn new(&self) -> Reciever{
        Reciever { frame_map: HashMap::new() }
    }


    pub fn reciever(&self, tx:SyncSender<(Vec<u8>,Vec<u8>,Vec<u8>)>)-> std::io::Result<()>{
        let socket = UdpSocket::bind("127.0.0.1:1234")?;

        loop{

            // Receives a single datagram message on the socket. If `buf` is too small to hold
            // the message, it will be cut off.
            let mut buf = [0; 1500];
            let (amt, src) = socket.recv_from(&mut buf)?;

            let frame_id = buf[0];
            let fragment_id = buf[1];
            let block_count= ((buf[2] as u16) << 8 )| buf[3] as u16;
            
            let data = &buf[4..amt];

            println!("Frame_id: {}  | Fragment_id: {} | block_count: {}",frame_id, fragment_id,block_count);
        }

    }

}