use std::{
    cell::RefCell,
    collections::HashMap,
    fs::OpenOptions,
    io::{BufWriter, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    rc::Rc,
    thread,
};
mod utils;

use flate2::write::GzDecoder;
use serde::Deserialize;

use tungstenite::{Error, Message, accept};

use crate::utils::tcp_filtransfer::UploadSession;

static ADDR: &str = "127.0.0.1:3031";

struct ConnectionState {
    current_upload_id: Option<Rc<str>>,
    uploads: HashMap<Rc<str>, UploadSession>,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct UploadStart {
    r#type: String,
    file_name: String,
    id: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct UploadEnd {
    r#type: String,
    id: String,
}

// Implement a TCP Websocket server for simple message passing!
fn main() {
    let listener =
        TcpListener::bind(ADDR).expect(&format!("Failed to bind tcp listener to {}", ADDR));

    println!("WebSocket listening on {ADDR}");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(move || {
                    handle_connections(s);
                });
            }

            Err(_) => {
                eprintln!("Error accepting connection!");
            }
        }
    }
}

fn handle_connections(stream: TcpStream) {
    let mut websocket = match accept(stream) {
        Ok(hand) => hand,

        Err(e) => {
            eprintln!("Websocket handshake failed! {}", e);
            return;
        }
    };

    let mut state = ConnectionState {
        current_upload_id: None,
        uploads: HashMap::new(),
    };

    println!("New Websocket connection established!");

    loop {
        match websocket.read() {
            Ok(msg) => match msg {
                // Handle Text Message
                tungstenite::Message::Text(data) => {
                    println!("Received Text: {data}",);
                    if let Ok(json) = serde_json::from_str::<UploadStart>(&data) {
                        let path: Rc<str> =
                            Rc::from(format!("uploads/{}", json.file_name).into_boxed_str());
                        let file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(Path::new(&*path))
                            .unwrap();
                        let writer = BufWriter::new(file);
                        let decoder = GzDecoder::new(writer);

                        let rc_file_id = Rc::from(json.id.as_str());

                        state.uploads.insert(
                            Rc::clone(&rc_file_id),
                            UploadSession {
                                buffer: Vec::new(),
                                threshold: 10 * 1024 * 1024, // 8 bits 1 byte
                                decoder: Some(decoder),
                                file_path: Rc::clone(&path),
                            },
                        );
                        state.current_upload_id = Some(Rc::clone(&rc_file_id));
                    } else if let Ok(json) = serde_json::from_str::<UploadEnd>(&data) {
                        if let Some(mut session) = state.uploads.remove(json.id.as_str()) {
                            session.write_to_disk();
                            if let Some(mut decoder) = session.decoder.take() {
                                if let Err(e) = decoder.flush() {
                                    eprintln!("Error flushing decoder : {}", e);
                                }
                            }
                            println!("Completed upload : {}", session.file_path);
                        }
                    } else {
                        println!("Unknwon text message : {}", &data);
                    }

                    if let Err(e) = websocket.send(Message::Text(data)) {
                        eprintln!("error sending text back {e}");
                        break;
                    }
                }

                // Handle Binary Data
                tungstenite::Message::Binary(data) => {
                    if data.len() < 16 {
                        eprintln!("Binary message is too short to contain ID ");
                        break;
                    }

                    let upload_id_bytes = &data[0..16];
                    let id = String::from_utf8_lossy(upload_id_bytes)
                        .trim_matches(char::from(0))
                        .to_string();
                    //println!("Received binary data: {} bytes", data.len());
                    if let Some(session) = state.uploads.get_mut(id.as_str()) {
                        let compressed_data = &data[16..];

                        session.write(compressed_data);
                    }
                }
                // Handle Pings
                tungstenite::Message::Ping(data) => {
                    println!("Received Ping!");
                    if let Err(e) = websocket.send(Message::Pong(data)) {
                        eprintln!("error sending pong back! {}", e);
                        break;
                    }
                }

                // Handle Frame // IDK What this is
                tungstenite::Message::Frame(_) => {}

                // Handle Pong
                tungstenite::Message::Pong(_) => {
                    println!("Received Pong!")
                }

                // Handle Connection Close
                tungstenite::Message::Close(data) => {
                    println!("Connection closing: {:?}", data);
                    break;
                }
            },
            Err(Error::ConnectionClosed) => {
                println!("Connection closed normally");
                break;
            }
            Err(e) => {
                eprintln!("Failed to read websocket message! {}", e);
                return;
            }
        }
    }
}
