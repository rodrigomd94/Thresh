use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::thread;
use std::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NativeMessage {
    #[serde(rename = "ping")]
    Ping { 
        #[serde(rename = "requestId")]
        request_id: Option<serde_json::Value> 
    },
    
    #[serde(rename = "pong")]
    Pong { 
        #[serde(rename = "requestId")]
        request_id: Option<serde_json::Value> 
    },
    
    #[serde(rename = "cip30")]
    Cip30Request {
        #[serde(rename = "requestId")]
        request_id: Option<serde_json::Value>,
        method: String,
        params: serde_json::Value,
    },
    
    #[serde(rename = "response")]
    Response {
        #[serde(rename = "requestId")]
        request_id: Option<serde_json::Value>,
        data: serde_json::Value,
        error: Option<String>,
    },
}

pub fn start_native_messaging() -> mpsc::Receiver<NativeMessage> {
    let (tx, rx) = mpsc::channel();
    
    // Start stdin reader thread
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        
        loop {
            // Read message length (4 bytes, little-endian)
            let mut length_bytes = [0u8; 4];
            match handle.read_exact(&mut length_bytes) {
                Ok(_) => {},
                Err(_) => break, // Exit on EOF
            }
            
            let length = u32::from_le_bytes(length_bytes) as usize;
            
            // Validate length (Chrome's limit is 1MB)
            if length > 1_048_576 {
                eprintln!("Message too large: {}", length);
                continue;
            }
            
            // Read message
            let mut buffer = vec![0u8; length];
            match handle.read_exact(&mut buffer) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to read message: {}", e);
                    continue;
                }
            }
            
            // Parse JSON
            match serde_json::from_slice::<NativeMessage>(&buffer) {
                Ok(message) => {
                    if let Err(e) = tx.send(message) {
                        eprintln!("Failed to send message to main thread: {}", e);
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                }
            }
        }
    });
    
    rx
}

pub fn send_message(message: &NativeMessage) -> io::Result<()> {
    let json = serde_json::to_string(message)?;
    let bytes = json.as_bytes();
    let length = bytes.len() as u32;
    
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    // Write length (4 bytes, little-endian)
    handle.write_all(&length.to_le_bytes())?;
    
    // Write message
    handle.write_all(bytes)?;
    handle.flush()?;
    
    Ok(())
}