use nix::sys::socket::{
    AddressFamily, MsgFlags, SockFlag, SockType, UnixAddr, bind, recvfrom, sendto, socket,
};
use std::fs::remove_file;
use std::os::fd::AsRawFd;
use std::path::Path;

fn main() {
    // 新しいsocketを作成する
    let sock = socket(
        AddressFamily::Unix,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )
    .expect("failed to create socket");

    // UNIXソケットで使用するserver_pathを作成する
    let server_path = "/tmp/udp_server";
    if Path::new(server_path).exists() {
        remove_file(server_path).unwrap();
    }
    let sock_addr = UnixAddr::new(server_path).unwrap();

    // アドレスの紐付け
    bind(sock.as_raw_fd(), &sock_addr).expect("failed to bind socket");

    println!("UDP Server started and listening on {}", server_path);

    // bufの大きさ
    let mut recv_buf = [0u8; 4096];

    // Ctrl+Cでの終了処理のためのハンドラー（シンプルなバージョン）
    println!("Press Ctrl+C to stop the server");

    // ソケットはデータの受信を永遠に待ち続ける
    loop {
        println!("\nWaiting to receive messages...");

        // ソケットからのデータを受信する
        let (bytes_received, sender_addr_option) =
            match recvfrom::<UnixAddr>(sock.as_raw_fd(), &mut recv_buf) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    continue;
                }
            };

        // 送信者のアドレスが取得できた場合のみ処理
        if let Some(sender_addr) = sender_addr_option {
            // 受信したデータの処理
            if bytes_received > 0 {
                let received_message_slice = &recv_buf[..bytes_received];

                let message = match std::str::from_utf8(received_message_slice) {
                    Ok(msg) => {
                        println!(
                            "Received {} bytes from {:?}: {}",
                            bytes_received, sender_addr, msg
                        );
                        msg
                    }
                    Err(_) => {
                        println!(
                            "Received {} bytes of non-UTF8 data from {:?}",
                            bytes_received, sender_addr
                        );
                        "non-UTF8 data"
                    }
                };

                // レスポンスメッセージの作成
                let response_message = format!(
                    "Server response: Successfully received your message \"{}\" ({} bytes)",
                    if message == "non-UTF8 data" {
                        message
                    } else {
                        &message[..std::cmp::min(message.len(), 50)]
                    },
                    bytes_received
                );
                let response_bytes = response_message.as_bytes();

                // クライアントにレスポンスを送信
                match sendto(
                    sock.as_raw_fd(),
                    response_bytes,
                    &sender_addr,
                    MsgFlags::empty(),
                ) {
                    Ok(sent_bytes) => {
                        println!(
                            "Sent {} bytes response back to {:?}",
                            sent_bytes, sender_addr
                        );
                    }
                    Err(e) => {
                        eprintln!("Error sending response to {:?}: {}", sender_addr, e);
                    }
                }
            } else {
                println!("Received empty message from {:?}", sender_addr);
            }
        } else {
            println!(
                "Received {} bytes, but sender address is unknown.",
                bytes_received
            );
        }

        // 受信バッファをクリア（次の受信のため）
        recv_buf.fill(0);
    }
}
