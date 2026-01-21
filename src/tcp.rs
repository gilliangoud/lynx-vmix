use crate::parser::LynxParser;
use crate::state::SharedState;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use log::{info, error};

pub async fn start_listener(state: SharedState, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind TCP listener");
    info!("TCP Listener waiting for FinishLynx on {}", addr);

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                info!("Accepted connection from {}", addr);
                let state_clone = state.clone();
                
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    let mut parser = LynxParser::new(state_clone);
                    
                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) => {
                                info!("Connection closed by {}", addr);
                                break;
                            }
                            Ok(n) => {
                                let chunk = &buf[0..n];
                                log::debug!("Received {} bytes: {:02X?}", n, chunk);
                                parser.process_chunk(chunk);
                            }
                            Err(e) => {
                                error!("Socket error: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}
