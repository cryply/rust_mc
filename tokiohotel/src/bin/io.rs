use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// Echo server — reads from client, writes it back
async fn echo_server() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("server listening on 127.0.0.1:8080");

    loop {
        // accept() returns (TcpStream, SocketAddr)
        let (socket, addr) = listener.accept().await.unwrap();
        println!("new connection from {}", addr);

        // Spawn a task per connection — each owns its socket
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buf = [0u8; 1024];

    loop {
        // read() returns 0 when connection is closed
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                println!("client disconnected");
                return;
            }
            Ok(n) => n,
            Err(e) => {
                println!("read error: {}", e);
                return;
            }
        };

        // Echo back what we received
        if let Err(e) = socket.write_all(&buf[..n]).await {
            println!("write error: {}", e);
            return;
        }
    }
}

// Alternative: use tokio::io::copy for zero-copy echo
async fn echo_server_copy() {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("copy-server listening on 127.0.0.1:8081");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            // split() gives separate read/write halves — can use concurrently
            let (mut reader, mut writer) = socket.split();
            // copy pipes all bytes from reader to writer
            tokio::io::copy(&mut reader, &mut writer).await.ok();
        });
    }
}

// Client — connects, sends a message, reads the echo back
async fn echo_client(port: u16) {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    let messages = ["hello", "tokio", "echo"];

    for msg in messages {
        stream.write_all(msg.as_bytes()).await.unwrap();

        let mut buf = [0u8; 1024];
        let mut buf2 = [3u8; 2048];
        let n = stream.read(&mut buf).await.unwrap();
        println!("client got echo: {}", std::str::from_utf8(&buf[..n]).unwrap());
    }
}

#[tokio::main]
async fn main() {
    // Start server in background
    tokio::spawn(echo_server());
    // Give server time to bind
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Run client against the manual echo server
    echo_client(8080).await;

    // Start copy-based server and test it too
    tokio::spawn(echo_server_copy());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    echo_client(8081).await;

    println!("done — servers would keep running in a real app");
}
