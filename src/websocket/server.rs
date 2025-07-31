use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use crossbeam::channel::Receiver;

use crate::websocket::events::MarketDataEvent;

pub type ClientId = Uuid;

#[derive(Debug)]
pub struct WebSocketServer {
    clients: Arc<Mutex<HashMap<ClientId, broadcast::Sender<MarketDataEvent>>>>,
    event_receiver: Receiver<MarketDataEvent>,
}

impl WebSocketServer {
    pub fn new(event_receiver: Receiver<MarketDataEvent>) -> Self {
        WebSocketServer {
            clients: Arc::new(Mutex::new(HashMap::new())),
            event_receiver,
        }
    }
    
    pub async fn start(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting WebSocket server on {}", addr);
        
        let listener = TcpListener::bind(addr).await?;
        let clients = Arc::clone(&self.clients);
        
        let event_receiver = self.event_receiver.clone();
        let broadcast_clients = Arc::clone(&clients);
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv() {
                let clients_lock = broadcast_clients.lock().await;
                let event_json = event.to_json();
                println!("Broadcasting: {}", event_json);
                
                for (client_id, sender) in clients_lock.iter() {
                    if let Err(_) = sender.send(event.clone()) {
                        println!("Failed to send to client {}", client_id);
                    }
                }
            }
        });
        
        while let Ok((stream, addr)) = listener.accept().await {
            println!("New WebSocket connection from: {}", addr);
            
            let clients = Arc::clone(&clients);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, clients).await {
                    println!("WebSocket connection error: {}", e);
                }
            });
        }
        
        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    clients: Arc<Mutex<HashMap<ClientId, broadcast::Sender<MarketDataEvent>>>>
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let client_id = Uuid::new_v4();
    let (tx, mut rx) = broadcast::channel::<MarketDataEvent>(100);
    
    {
        let mut clients_lock = clients.lock().await;
        clients_lock.insert(client_id, tx);
        println!("Client {} registered", client_id);
    }
    
    let welcome = format!(
        r#"{{"type":"welcome","client_id":"{}","message":"Connected to trading engine"}}"#, 
        client_id
    );
    ws_sender.send(Message::Text(welcome)).await?;
    
    loop {
        tokio::select! {
            event_result = rx.recv() => {
                match event_result {
                    Ok(event) => {
                        let message = Message::Text(event.to_json());
                        if let Err(_) = ws_sender.send(message).await {
                            println!("Failed to send message to client {}", client_id);
                            break;
                        }
                    }
                    Err(_) => {
                        println!("Broadcast channel closed for client {}", client_id);
                        break;
                    }
                }
            }
            
            msg_result = ws_receiver.next() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        println!("Received from client {}: {}", client_id, text);
                        let response = format!(r#"{{"type":"echo","original":"{}"}}"#, text);
                        if let Err(_) = ws_sender.send(Message::Text(response)).await {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Client {} disconnected", client_id);
                        break;
                    }
                    Some(Err(e)) => {
                        println!(" WebSocket error for client {}: {}", client_id, e);
                        break;
                    }
                    None => {
                        println!("Connection closed for client {}", client_id);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    
    {
        let mut clients_lock = clients.lock().await;
        clients_lock.remove(&client_id);
        println!(" Client {} removed", client_id);
    }
    
    Ok(())
}