use std::io;
use std::sync::Arc;
use std::fs;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time;
use local_ip_address::local_ip;
use dashmap::DashMap;
use chrono::{DateTime, Timelike, Local, naive::NaiveDate};
use serde_json;

pub mod db;

use db::{Database, Db};

type Res = Result<(), Box<dyn std::error::Error>>;

const PRICE_COMMAND: &str = "[[current price]] ";
const EOT: u8 = 0x4;

#[tokio::main]
async fn main() -> Res {
    let mut database = Db::new();
    database.load().await;
    let database = Arc::new(database);
    let ip_addr = local_ip().unwrap();
    let address = format!("{:?}:35000", ip_addr);
    println!("Listening on {}", address);
    let listener = TcpListener::bind(address).await?;

    let db_clone = database.clone();
    tokio::spawn(async move {
        backup_db(db_clone).await;
    });

    loop {
        let (socket, ip) = listener.accept().await?;
    
        let db = database.clone();
        tokio::spawn(async move {
            process(db, socket, ip).await;
        });
    }
}

async fn process(db: Database, socket: TcpStream, peer: std::net::SocketAddr) -> Res {
    let mut socket = socket;
    println!("Connection established to {:?}", peer);
    socket.write_all(b"Connection established to MASKINEN\n").await?;
 
    loop {
        socket.readable().await?;
        let mut buf = vec![0; 4096];

        match socket.try_read(&mut buf) {
            Ok(0) => break,
            Ok(1) => {
                if buf[0] == EOT {
                    break;
                }
            },
            Ok(n) => {
                let string = std::string::String::from_utf8(buf)?;
                let string = string.trim();
                println!("Read {} bytes: {}", n, string);
                if let Some(response) = parse_input(db.clone(), &string).await {
                    socket.write_all(response.as_bytes()).await?;
                    socket.write_all(b"\n").await?;
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    println!("Terminating connection to {:?}...", peer);
    socket.write_all(b"Goodbye!\n").await?;
    Ok(())
}

async fn parse_input(db: Database, input: &str) -> Option<String> {
    if handle_new_current_price_transfer(db.clone(), input).await {
        return None;
    } else if let Some(resp) = handle_get_current_price(db.clone(), input).await {
        return Some(resp);
    }

    return Some("Unknown input".to_string());
}

async fn handle_new_current_price_transfer(db: Database, input: &str) -> bool {
    let is_new_price_transfer = input.find(PRICE_COMMAND).is_some();
    
    if is_new_price_transfer {
        let val = extract_val(input).await;
        println!("Current el price: {}", val);
        let key = get_current_time_stamp().await; 
        db.insert(key, val).await;
    }

    is_new_price_transfer
}

async fn extract_val(input: &str) -> f64 {
    let val = input.replace(PRICE_COMMAND, "");
    let val = val.replace("\r\n", "");
    let val = val.replace("\0", "");
    val.parse().unwrap()
}

async fn get_current_time_stamp() -> String {
    Local::now().format(db::DATE_FORMAT).to_string()
}

async fn get_previous_time_stamp() -> String {
    let now = Local::now();
    let prev_time = now.checked_sub_signed(chrono::Duration::hours(1)).unwrap();
    prev_time.format(db::DATE_FORMAT).to_string()
}

async fn handle_get_current_price(db: Database, input: &str) -> Option<String> {
const GET_PRICE_COMMAND: &str = "get_current_price";
    let is_get_current_price_transfer = input.find(GET_PRICE_COMMAND).is_some();

    if is_get_current_price_transfer {
        let current_price = get_current_price(db).await;
        return Some(current_price);
    } else {
        return None;
    }
}

async fn get_current_price(db: Database) -> String {
    let key = get_current_time_stamp().await;
    let current_price: String;
    if let Some(val) = db.get(&key).await {
        current_price = format!("{}", val);
    } else {
        let key = get_previous_time_stamp().await;
        current_price = format!("{}", db.get(&key).await.unwrap());
    }
    current_price
}

async fn backup_db(db: Database) {
    let mut interval = time::interval(time::Duration::from_secs(60 * 10));
    loop {
        interval.tick().await;
        println!("Saving database...");
        db.save().await;
        println!("Database saved!");
    }
}

