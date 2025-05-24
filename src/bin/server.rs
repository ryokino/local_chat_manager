use fake::Fake;
use fake::faker::company::en::CompanyName;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use nix::sys::socket::{
    AddressFamily, Backlog, MsgFlags, SockFlag, SockType, UnixAddr, accept, bind, getpeername,
    listen, recv, send, socket,
};
use nix::unistd::close;
use std::fs::remove_file;
use std::os::fd::AsRawFd;
use std::path::Path;

const BUFFER_SIZE: usize = 128;
const SERVER_ADDRESS: &str = "/tmp/socket_file";
const MAX_CONNECTIONS: i32 = 128;

fn generate_fake_response(original_message: &str) -> String {
    // メッセージの内容に応じて異なるタイプのfakeデータを生成
    let message_lower = original_message.to_lowercase();

    if message_lower.contains("name") || message_lower.contains("user") {
        let fake_name: String = Name().fake();
        let fake_email: String = SafeEmail().fake();
        format!("Generated user: {} <{}>", fake_name, fake_email)
    } else if message_lower.contains("company") || message_lower.contains("business") {
        let fake_company: String = CompanyName().fake();
        format!("Random company: {}", fake_company)
    } else if message_lower.contains("quote") || message_lower.contains("text") {
        let fake_sentence: String = Sentence(3..8).fake();
        format!("Random quote: \"{}\"", fake_sentence)
    } else {
        // デフォルトの場合は複数の情報を組み合わせ
        let fake_name: String = Name().fake();
        let fake_company: String = CompanyName().fake();
        let fake_sentence: String = Sentence(2..5).fake();
        format!(
            "Server response for '{}': {} from {} says: \"{}\"",
            original_message.trim(),
            fake_name,
            fake_company,
            fake_sentence
        )
    }
}

fn handle_client_connection(
    connection_fd: i32,
    peer_addr: UnixAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; BUFFER_SIZE];

    // ピア情報の表示を関数化
    let peer_info = match peer_addr.path() {
        Some(path) if path.as_os_str().is_empty() => "unnamed peer".to_string(),
        Some(path) => format!("peer {}", path.display()),
        None => "unknown peer (no path)".to_string(),
    };

    println!("Handling connection from {}", peer_info);

    // クライアントからのデータを継続的に受信
    loop {
        // バッファをクリア
        buf.fill(0);

        match recv(connection_fd, &mut buf, MsgFlags::empty()) {
            Ok(0) => {
                // データサイズが0 = 接続終了
                println!("Client {} disconnected normally", peer_info);
                break;
            }
            Ok(bytes_received) => {
                // データを受信した場合
                let message = String::from_utf8_lossy(&buf[..bytes_received]);
                println!(
                    "Received from {}: {} ({} bytes)",
                    peer_info,
                    message.trim(),
                    bytes_received
                );

                // fake クレートを使用してレスポンスを生成
                let response = generate_fake_response(message.trim());

                if let Err(e) = send_complete_message(connection_fd, response.as_bytes()) {
                    eprintln!("Failed to send response to {}: {}", peer_info, e);
                    break;
                }

                println!("Sent fake response to {}: {}", peer_info, response);
            }
            Err(e) => {
                eprintln!("Error receiving data from {}: {}", peer_info, e);
                break;
            }
        }
    }

    Ok(())
}

fn send_complete_message(fd: i32, message: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut sent = 0;

    while sent < message.len() {
        match send(fd, &message[sent..], MsgFlags::empty()) {
            Ok(0) => {
                return Err("Connection closed by peer during send".into());
            }
            Ok(n) => {
                sent += n;
            }
            Err(e) => {
                return Err(format!("Send error: {}", e).into());
            }
        }
    }

    Ok(())
}

fn cleanup_socket_file() {
    if Path::new(SERVER_ADDRESS).exists() {
        if let Err(e) = remove_file(SERVER_ADDRESS) {
            eprintln!("Warning: Failed to remove existing socket file: {}", e);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Starting TCP server with fake responses on {}",
        SERVER_ADDRESS
    );

    // 既存のソケットファイルをクリーンアップ
    cleanup_socket_file();

    // UNIXソケットの作成
    let server_sock = socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )?;

    // アドレスの作成とバインド
    let sock_addr = UnixAddr::new(SERVER_ADDRESS)?;
    bind(server_sock.as_raw_fd(), &sock_addr)?;

    // 接続をリッスン
    listen(&server_sock, Backlog::new(MAX_CONNECTIONS)?)?;

    println!("Server listening for connections... (Press Ctrl+C to stop)");
    println!("Try sending messages with keywords: 'name', 'company', 'quote'");

    // メインループ: クライアント接続を受け入れる
    loop {
        match accept(server_sock.as_raw_fd()) {
            Ok(connection_fd) => {
                // クライアントのピア情報を取得
                match getpeername::<UnixAddr>(connection_fd) {
                    Ok(peer_addr) => {
                        println!("New connection accepted");

                        // クライアント接続をハンドル
                        if let Err(e) = handle_client_connection(connection_fd, peer_addr) {
                            eprintln!("Error handling client connection: {}", e);
                        }

                        // 接続を閉じる
                        if let Err(e) = close(connection_fd) {
                            eprintln!("Error closing connection: {}", e);
                        } else {
                            println!("Connection closed successfully");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get peer name: {}", e);
                        let _ = close(connection_fd);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                // 一時的なエラーの場合は続行
                continue;
            }
        }
    }
}
