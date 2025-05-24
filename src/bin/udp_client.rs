use nix::sys::socket::sockopt::ReceiveTimeout;
use nix::sys::socket::{
    AddressFamily, MsgFlags, SockFlag, SockType, UnixAddr, connect, recv, send, setsockopt, socket,
};
use nix::sys::time::{TimeVal, TimeValLike};
use nix::unistd::close;
use std::os::fd::AsRawFd;

const BUFFER_SIZE: usize = 1024; // バッファサイズを適切に設定
const SERVER_ADDRESS: &str = "/tmp/socket_file";
const RECEIVE_TIMEOUT_SECS: i64 = 5; // タイムアウトを少し長めに

fn main() {
    // TCP/IPソケットを作成する
    let sock = socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .expect("failed to create socket");

    // 受信タイムアウトの設定
    let timeout = TimeVal::seconds(RECEIVE_TIMEOUT_SECS);
    setsockopt(&sock, ReceiveTimeout, &timeout).expect("failed to set receive timeout");

    // サーバーアドレスの作成
    let sock_addr = UnixAddr::new(SERVER_ADDRESS).expect("failed to create socket address");

    // サーバーへの接続
    println!("Connecting to server at {}", SERVER_ADDRESS);
    match connect(sock.as_raw_fd(), &sock_addr) {
        Ok(_) => println!("Successfully connected to server"),
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            let _ = close(sock);
            return;
        }
    }

    // メッセージの準備と送信
    let message = "Hello from client! This is a test message.";
    let message_bytes = message.as_bytes();

    println!("Sending message: {}", message);

    // 完全な送信を保証
    let mut sent = 0;
    while sent < message_bytes.len() {
        match send(sock.as_raw_fd(), &message_bytes[sent..], MsgFlags::empty()) {
            Ok(0) => {
                eprintln!("Connection closed by server during send");
                let _ = close(sock);
                return;
            }
            Ok(n) => {
                sent += n;
                println!("Sent {} bytes (total: {}/{})", n, sent, message_bytes.len());
            }
            Err(e) => {
                eprintln!("Failed to send message: {}", e);
                let _ = close(sock);
                return;
            }
        }
    }

    // サーバーからの応答を受信
    let mut buf = [0u8; BUFFER_SIZE];
    println!("Waiting for server response...");

    match recv(sock.as_raw_fd(), &mut buf, MsgFlags::empty()) {
        Ok(0) => {
            println!("Server closed connection without response");
        }
        Ok(bytes_received) => {
            let response =
                std::str::from_utf8(&buf[..bytes_received]).unwrap_or("Invalid UTF-8 response");
            println!("Server response ({} bytes): {}", bytes_received, response);
        }
        Err(e) => {
            eprintln!("Failed to receive response: {}", e);
        }
    }

    // 接続のクローズ
    match close(sock) {
        Ok(_) => println!("Connection closed successfully"),
        Err(e) => eprintln!("Error closing connection: {}", e),
    }
}
