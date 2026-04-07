use std::{net::UdpSocket, sync::mpsc, thread};
mod huffcode;
mod sdl_window;
mod reciever;


fn main() {

    let (tx ,rx)  = mpsc::sync_channel::<(Vec<u8>,Vec<u8>,Vec<u8>)>(1);


    thread::spawn(move || {
        reciever::reciever(tx);
    });

    sdl_window::start_window(rx);




}