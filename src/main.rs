// src/main.rs

use rust_mini_redis::db::Db;
use rust_mini_redis::request::Request;
use rust_mini_redis::returns::Return;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::env;

static CLIENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

async fn read_success<W: AsyncWriteExt + Unpin>(db: Arc<Db>, line: String, writer: &mut W, client_id: u64) -> Option<broadcast::Receiver<String>> {
    if line.is_empty() {
        return None;
    }

    print!("Received: {}", line);

    let result = match Request::parse(&line) {
        Ok(request) => {
            let command = request.into_command();
            command.execute(&db, client_id)
        }
        Err(err) => Return::Err(err),
    };

    let response = match result {
        Return::Ok(value) => format!("{}\r\n", value),
        Return::Err(err) => format!("Error: {}\r\n", err),
        Return::NotFound(key) => format!("Key '{}' not found\r\n", key),
        Return::Subscribe(receiver) => {
            let _ = writer.write_all(b"Subscribed\r\n").await;
            return Some(receiver);
        }
        Return::Unsubscribe => {
            let _ = writer.write_all(b"Unsubscribed\r\n").await;
            return None;
        }
    };

    if writer.write_all(response.as_bytes()).await.is_err() {
        println!("Write error.");
        return None;
    }
    None
}

async fn handle_client(stream: TcpStream, db: Arc<Db>) {
    let client_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();
    let mut subscription: Option<broadcast::Receiver<String>> = None;

    loop {
        buffer.clear();

        if let Some(ref mut rx) = subscription {
            tokio::select! {
                result = reader.read_line(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            println!("Client {} disconnected.", client_id);
                            db.cleanup_client(client_id);
                            return;
                        }
                        Ok(_) => {
                            let result = read_success(db.clone(), buffer.clone(), &mut writer, client_id).await;
                            match result {
                                Some(rx) => subscription = Some(rx),
                                None => subscription = None,
                            }
                        }
                        Err(e) => {
                            println!("Read error: {}", e);
                            db.cleanup_client(client_id);
                            return;
                        }
                    }
                }
                result = rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if writer.write_all(format!("{}\r\n", msg).as_bytes()).await.is_err() {
                                println!("Write error.");
                                db.cleanup_client(client_id);
                                return;
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            println!("Client {}: Channel closed.", client_id);
                            subscription = None;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                    }
                }
            }
        } else {
            match reader.read_line(&mut buffer).await {
                Ok(0) => {
                    println!("Client {} disconnected.", client_id);
                    db.cleanup_client(client_id);
                    return;
                }
                Ok(_) => {
                    if let Some(rx) = read_success(db.clone(), buffer.clone(), &mut writer, client_id).await {
                        subscription = Some(rx);
                    }
                }
                Err(e) => {
                    println!("Read error: {}", e);
                    db.cleanup_client(client_id);
                    return;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "--ui" {
        run_ui_mode();
    } else {
        run_server_mode().await;
    }
}

fn run_ui_mode() {
    println!("Starting Mini Redis Dashboard...");
    let db = Arc::new(Db::new());
    
    // Start TCP server in background
    let db_clone = Arc::clone(&db);
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:6379").await.expect("Unable to open port");
        println!("TCP server started on port 6379");
        
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let db = Arc::clone(&db_clone);
                    tokio::spawn(async move {
                        handle_client(stream, db).await;
                    });
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }
    });
    
    // Run TUI in main thread
    if let Err(e) = rust_mini_redis::ui::run_ui(db) {
        eprintln!("UI error: {}", e);
    }
}

async fn run_server_mode() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.expect("Unable to open port");

    println!("Started on port 6379.");
    println!("Press Ctrl+C to shutdown.");
    println!("Waiting for connections...");
    println!("Tip: Run with --ui flag for interactive dashboard");

    let db = Arc::new(Db::new());

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        println!("New connection established!");
                        let db = Arc::clone(&db);
                        tokio::spawn(async move {
                            handle_client(stream, db).await;
                        });
                    }
                    Err(e) => {
                        println!("Connection error: {}", e);
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutdown signal received...");
                println!("Shutting down gracefully...");
                println!("Server stopped.");
                return;
            }
        }
    }
}
