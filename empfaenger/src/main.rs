use std::{sync::mpsc::{self}, thread};
mod huffcode;
mod sdl_window;
mod reciever;
mod capture;
mod fast_dct;
mod bit_writer;

fn main() {
    println!("TEST JUSTX");
    let (tx ,rx)  = mpsc::sync_channel::<(Vec<u8>,Vec<u8>,Vec<u8>)>(1);
    let mut reciev = reciever::Reciever::new(tx).unwrap();
    
    thread::spawn(move || {
         reciev.reciever().unwrap();
    });

    sdl_window::start_window(rx);




}