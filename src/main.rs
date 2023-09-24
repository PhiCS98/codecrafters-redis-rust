use anyhow::Result;
use redis_server::{
    resp::Value::{BulkString, Error, Null, SimpleString},
    resp::{self, Value},
    store::RedisValueStore,
};
use tokio::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

mod redis_server;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let data_store = Arc::new(Mutex::new(RedisValueStore::new()));

    loop {
        let thread_store = data_store.clone();
        match listener.accept().await {
            Ok((socket, _)) => {
                println!("Accepted new connection");
                tokio::spawn(async move {
                    handle_connection(socket, thread_store).await.unwrap();
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

async fn handle_connection(socket: TcpStream, data_store: Arc<Mutex<RedisValueStore>>) -> Result<()> {
    let mut handler = resp::RespHandler::new(socket);
    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "set" => {
                    if let (Some(BulkString(key)), Some(BulkString(value))) =
                        (args.get(0), args.get(1))
                    {
                        if let (Some(BulkString(_)), Some(BulkString(amount))) =
                            (args.get(2), args.get(3))
                        {
                            data_store.lock().unwrap().set_with_expiry(
                                key.to_string(),
                                value.to_string(),
                                amount.parse::<u64>().unwrap(),
                            );
                        } else {
                            data_store.lock().unwrap().set(key.to_string(), value.to_string())
                        }
                        SimpleString("OK".to_string())
                    } else {
                        Error("invalid arguments".to_string())
                    }
                }
                "get" => {
                    if let Some(BulkString(key)) = args.get(0) {
                        if let Some(value) = data_store.lock().unwrap().get(key) {
                            SimpleString(value.to_string())
                        } else {
                            Null
                        }
                    } else {
                        Error("Invalid arguments given".to_string())
                    }
                }
                _ => Error(format!("Cannot handle command {}", command)),
            }
        } else {
            break;
        };
        handler.write_value(response).await.unwrap();
    }
    Ok(())
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_string(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_string(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be a bulk string")),
    }
}
