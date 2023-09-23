use anyhow::Result;
use redis_server::{
    resp::{self, Value},
    store::RedisValueStore,
};
use tokio::net::{TcpListener, TcpStream};

mod redis_server;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                println!("Accepted new connection");
                tokio::spawn(async move {
                    handle_connection(socket).await;
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

async fn handle_connection(socket: TcpStream) {
    let mut handler = resp::RespHandler::new(socket);
    let mut store = RedisValueStore::new();
    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "set" => {
                    let key = unpack_bulk_string(args.first().unwrap().clone()).unwrap();
                    let value = unpack_bulk_string(args.last().unwrap().clone()).unwrap();
                    store.set(key, value);
                    Value::SimpleString("OK".to_string())
                }
                "get" => {
                    let key = unpack_bulk_string(args.first().unwrap().clone()).unwrap();
                    match store.get(&key) {
                        Some(value) => Value::BulkString(value.clone()),
                        None => Value::SimpleString("$-1\r\n".to_string()),
                    }
                }
                c => panic!("Cannot handle command {}", c),
            }
        } else {
            break;
        };
        handler.write_value(response).await.unwrap();
    }
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
