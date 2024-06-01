use std::{
    fs::File,
    io::{
        BufReader, 
        Read, 
        Write
    },
    os::{
        fd::RawFd,
        unix::io::FromRawFd
    },
    sync::Arc
};
use futures::{
    SinkExt,
    stream::SplitSink
};
use tokio::sync::{Mutex, MutexGuard};
use warp::ws::{ Message, WebSocket };

#[derive(Debug)]
pub struct Terminal;
impl Terminal{
    pub fn reader(master_raw_fd: RawFd, wx_tx_state_reader: Arc<Mutex<SplitSink<WebSocket, Message>>>) -> tokio::task::JoinHandle<()> {                              
        tokio::spawn(async move {
            let mut buf: BufReader<File> = BufReader::new( unsafe {  File::from_raw_fd(master_raw_fd) });    
            let mut output_buffer: Vec<u8> = vec![0;1024];
            let mut output_all: Vec<u8> = Vec::new();
            loop {
                if let Ok(b) = buf.read(&mut output_buffer) {
                    if b > 0 {
                        for &byte in &output_buffer {
                            if byte != 0 {
                                output_all.push(byte);
                            }
                        }                        
                        let mut ws_tx: MutexGuard<SplitSink<WebSocket, Message>> = wx_tx_state_reader.lock().await;
                        match ws_tx.send(Message::binary(output_all)).await {
                            Ok(_) => {
                                output_all = Vec::new();
                                output_buffer = vec![0;1024];
                                drop(ws_tx);
                            },
                            Err(error) => {
                                eprintln!("UNABLE_TO_SEND_MESSAGE: {:?}",error);
                                drop(ws_tx);
                                break;
                            }
                        }
                    }
                }
                else{                    
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        })
    }

    pub fn writer(f: &mut File, command: &[u8]) -> Result<bool,std::io::Error> { 
        if let Err(error) = f.write_all(command) {
            return Err(error);
        }
        if let Err(error) = f.flush() {
            return Err(error);
        }
        Ok(true)
    }
}