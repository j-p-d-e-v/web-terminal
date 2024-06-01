use futures::{
    stream::SplitSink,
    SinkExt
};
use futures_util::StreamExt;
use warp::{
    ws::{Message, WebSocket}, 
    Filter
};
use serde_derive::{Deserialize, Serialize};
use web_terminal::terminal::Terminal;
use tokio::sync::Mutex;
use nix::{
    sys::signal::kill,
    pty::{
        forkpty,
        Winsize,
        ForkptyResult
    }
};
use std::{
    net::Ipv4Addr,
    sync::Arc,
    process::Command,
    time::Instant,
    fs::File,
    os::{
        unix::io::FromRawFd,
        fd::AsRawFd
    }
};
use clap::Parser;
#[derive(Debug, Deserialize, Serialize)]
struct QueryParams {
    terminal_name: String,
    #[serde(default)]
    terminal_shell: String,
    terminal_rows: u16,
    terminal_cols: u16
}
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = "Parameters when running the web terminal app.")]
struct Args {
    /// Host in IPV4 IP Address format.
    #[arg(long, default_value="127.0.0.1",help="The ip of the server.")]
    host: String,
    
    /// The port number to use.
    #[arg(long, default_value="3030",help="The port of the server.")]
    port: u16,
    
    /// The heartbeat interval.
    #[arg(long, default_value="30",help="The heartbeat interval.")]
    heartbeat_interval: u64,
}
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let args: Args  = Args::parse();
    let port: u16 = args.port;
    let host: Ipv4Addr = args.host.parse().expect("Unable to parse host as IPv4 Address");  
    let heartbeat_interval: u64 = args.heartbeat_interval;

    let with_args = warp::any().map(move || heartbeat_interval.clone());

    let ws_terminal = warp::path("terminal")
    .and(warp::ws())
    .and(warp::query::<QueryParams>())
    .and(with_args)
    .map(| ws: warp::ws::Ws, query: QueryParams, heartbeat_interval: u64 | {
        ws.on_upgrade(move|socket| websocket_terminal(
            socket,
            query.terminal_name,
            query.terminal_cols,
            query.terminal_rows,
            query.terminal_shell,
            heartbeat_interval
        ))
    });
    let routes = ws_terminal;
    println!("Server running at {}:{}",host.to_string(),port);
    warp::serve(routes).run((host.octets(),port)).await;
}

/// Runs the websocket terminal session
async fn websocket_terminal(
    socket: WebSocket,
    terminal_name: String,
    terminal_cols: u16,
    terminal_rows: u16,
    terminal_shell: String,
    heartbeat_interval: u64
){
    let shell: String = if terminal_shell.len() > 0 {
        terminal_shell
    }
    else {
        "/bin/bash".to_string()
    };
    let mut ws_client_last_seen: Instant = Instant::now();

    match unsafe { forkpty(&Winsize {
        ws_row: terminal_rows,
        ws_col: terminal_cols,
        ws_xpixel:12,
        ws_ypixel:12,
    }, None) } {
        Ok(forkpty_result) => {
            match forkpty_result {
                ForkptyResult::Parent { master, child } => {
                    println!("Terminal Name:{}, Master: {:?}, Child: {:?}",terminal_name,master,child);
                    let (ws_tx, mut ws_rx) = socket.split();
                    let ws_tx_state: Arc<Mutex<SplitSink<WebSocket, Message>>> = Arc::new(Mutex::new(ws_tx));
                    let master_fd_write = master.as_raw_fd();
                    let master_fd_read = master_fd_write.clone();                    
                    let term_tx_thread = Terminal::reader(master_fd_read,ws_tx_state.clone());
                    let mut file_handler = unsafe { File::from_raw_fd(master_fd_write) };

                    loop {
                        if ws_client_last_seen.elapsed() >= std::time::Duration::from_secs(heartbeat_interval) {
                            eprintln!("WEBSOCKET_CLIENT_DISCONNECTED");
                            break;
                        }
                        
                        if let Ok(result) = tokio::time::timeout(std::time::Duration::from_millis(100), async { ws_rx.next().await } ).await {
                            match result {
                                Some(ws_result) =>{
                                    match ws_result {
                                        Ok(message) => {     
                                            if message.is_close() {
                                                eprintln!("WEBSOCKET_CLIENT_CLOSED: {}",terminal_name);
                                                break;
                                            }
                                            if message.to_str().unwrap().to_string() == "__HEARTBEAT__".to_string() {   
                                                println!("__HEARTBEAT__ received from: {}, {:?}",terminal_name,master);                                                                    
                                                ws_client_last_seen = Instant::now();
                                            } 
                                            else { 
                                                if let Err(error) = Terminal::writer(&mut file_handler, message.as_bytes()) {
                                                    eprintln!("TERM_WRITER_ERROR: {:?}",error);
                                                }
                                            }
                                        }
                                        Err(error) => {
                                            eprint!("WS_RESULT_ERROR: {:?}",error);
                                        }
                                    };
                                }
                                None => {}
                            };
                        }
                    }
                    // Terminates/Aborts the thread spawn by the tokio.
                    term_tx_thread.abort();
                    // Terminates the child process spawned by the forkpty.
                    if let Err(err) = kill(child,nix::sys::signal::Signal::SIGQUIT)  {
                        println!("UNABLE_TO_EXIT_CHILD_PROCESS: {:?}. ERROR: {:?}",child,err);
                    }
                    // Closes the websocket transceiver that will also close the websocket session.
                    if let Err(err) = ws_tx_state.lock().await.close().await {
                        println!("UNABLE_CLOSE_SOCKET with Master: {:?}, Child:{:?}. ERROR: {:?}",master,child,err);
                    }
                    // Drops the websocket transceiver shared state.
                    drop(ws_tx_state);
                    println!("DISCONNECTED: {}",terminal_name);
                }
                ForkptyResult::Child => {
                    Command::new(shell).spawn().unwrap();
                }
            }
        }
        Err(error) => {
            if let Err(error) = socket.close().await {
                eprintln!("SOCKET_CLOSE_ERROR: {:?}",error);
            }
            panic!("FORKPTY_ERROR: {:?}",error);
        }
    }
    
}