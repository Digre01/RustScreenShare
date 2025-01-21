use std::net::UdpSocket;
use std::sync::mpsc::{self, Receiver,TryRecvError};
use std::error::Error;
use std::io::{self,ErrorKind};

use crate::ControlMessage;
pub struct DiscoveryServer {
    sender: mpsc::Sender<String>,
    clients: String,
}

/* enum ControlMessage {
    Pause,
    Resume,
    Stop,
} */

impl DiscoveryServer {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        Self {
            sender,
            clients: String::new(),
        }
    }

    pub fn run_discovery_listener( &mut self,control_receiver:Receiver<ControlMessage>)  -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:9000")?;
        socket.set_nonblocking(true)?;
                loop {
                    match control_receiver.try_recv() {
                       
                        Ok(ControlMessage::Stop) => {
                            println!("Received STOP signal. Stopping discovery listener...");
                            return Ok(());
                        }         
     
                        Err(TryRecvError::Empty) => {
                            //do nothing
                        }
                        
                        Err(e) => {
                            eprintln!("Control channel error: {}", e);
                            return Err(Box::new(io::Error::new(ErrorKind::Other, e.to_string())));
                        }
                    }


                    let mut buf = [0; 1024];
                    let (amt, src) = match socket.recv_from(&mut buf) {
                        Ok(result) => result,
                        Err(e) => {
                            if e.kind() == ErrorKind::WouldBlock {
                                // If the socket would block (no data), we simply continue
                                continue;
                            } else {
                                // Handle other errors (e.g., IO errors)
                                println!("Error receiving data: {}", e);
                                continue;
                        }}
                    };
        
                    let received_message = String::from_utf8_lossy(&buf[..amt]);
                    println!("Received message: '{}' from {}", received_message, src);
        
                    if received_message.trim() == "DISCOVERY" {
                        //Responds to the client by providing the IP address that will be assigned in the multi-UDP.
                        let response = format!("{}", src.ip().to_string());
        
                        if let Err(e) = socket.send_to(response.as_bytes(), &src) {
                            println!("Failed to send response: {}", e);
                        } else {
                            println!("Sent response '{}' to client {}", response, src);
                        }
        
        
                        
                        if !self.clients.is_empty() {
                            self.clients.push_str(&format!(",{}", src.to_string()));
                        }
                        else{
                            self.clients.push_str(&format!("{}", src.to_string()));
                        }
        
        
                        
        
                        //if let Err(e) = self.sender.send(src) {
                        if let Err(e) = self.sender.send(self.clients.clone()) {
                            println!("Failed to send client list: {}", e);
                        }
                    }
                    else if received_message.trim() == "DISCONNECT" { 
                        
                        let port = src.port().to_string();

                        
                        
                        let clients_str: Vec<String> = self.clients.split(',')
                                        .filter(|&s| {
                                                
                                                if let Some(port_str) = s.split(":").nth(1) {
                                                    port_str != port 
                                                } else {
                                                    true // Keep the client if the port is not present (unexpected case)
                                                }
                                                
                                            })
                        .map(|s| s.to_string()) 
                        .collect();
                        
                        self.clients = if clients_str.is_empty() {
                            String::new() 
                        } else {
                            clients_str.join(",") 
                        };
                        
                    

                        if let Err(e) = self.sender.send(self.clients.clone()) {
                            println!("Failed to send client list: {}", e);
                        }
                    }
              
            


        }

    }
}
