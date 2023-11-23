use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio::io::copy;

#[tokio::main]
async fn main() {
    // 启动代理监听在 127.0.0.1:3307
    let listener = TcpListener::bind("127.0.0.1:3307").await.expect("Failed to bind to address");

    println!("MySQL proxy listening on 127.0.0.1:3307");

    // 接受传入的连接
    while let Ok((inbound, _)) = listener.accept().await {
        // 每个连接都在自己的任务中处理
        tokio::spawn(handle_client(inbound));
    }
}

async fn handle_client(mut inbound: TcpStream) {
    // 与 MySQL 服务器建立连接
    let server_addr = "127.0.0.1:3306";
    let server_conn = timeout(std::time::Duration::from_secs(5), TcpStream::connect(server_addr))
        .await
        .expect("Failed to connect to MySQL server")
        .expect("Connection to MySQL server timed out");

    let (mut client_reader, mut client_writer) = tokio::io::split(inbound);
    let (mut server_reader, mut server_writer) = tokio::io::split(server_conn);

    // 从客户端读取数据并转发到服务器
    tokio::spawn(async move {
        let _ = copy(&mut client_reader, &mut server_writer).await;
    });

    // 从服务器读取数据并转发到客户端
    let _ = copy(&mut server_reader, &mut client_writer).await;
}
