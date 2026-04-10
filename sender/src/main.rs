use std::net::UdpSocket;

#[derive(Debug)]
struct Data<'a>{
    len:u16,
    fragment_id: u8,
    frame_id: u8,
    data:&'a [u8],
}


fn main() {
    println!("Hello, world!");
    let dat = &[23,53,65,87,23,12,43,75,34,12,43];
    let data = Data{len: dat.len() as u16, fragment_id: 12, frame_id: 40, data: dat };

    let mut buf:Vec<u8> = Vec::new();

    buf.push((data.len >> 8) as u8);
    buf.push((data.len & 0xFF) as u8);
    buf.push(data.fragment_id);
    buf.push(data.frame_id);
    buf.extend_from_slice(data.data);

    println!("DATALEN: {:b}", data.len >> 8);
    println!("DATALEN: {:b}", data.len & 0xFF);

    buf[0] = 0b11111110;
    buf[1] = 0b01111110;


    let x = u16::from_be_bytes([buf[0], buf[1]]);
    println!("{:016b}", x); 
/*
    for (index, &buffer) in buf.iter().enumerate(){
        println!("BYTES BUF{}:\t{:08b}", index, buffer);
    }
    println!("{:?}", data);

 */


    //send(buf);

}


fn send(buffer:Vec<u8>) ->std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:1235")?;
    socket.send_to(&buffer, "127.0.0.1:1234").unwrap();
    Ok(())
}

