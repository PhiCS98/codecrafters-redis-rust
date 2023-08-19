// Uncomment this block to pass the first stage
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((mut socket, _)) => {
                println!("Accepted new connection");
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 512];

                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) => return,
                            Ok(_) => {
                                if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                                    eprintln!("Failed to write to socket; err = {:?}", e);
                                    return;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to read from socket; err = {:?}", e);
                                return;
                            }
                        }
                    }
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
