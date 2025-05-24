use nix::sys::socket::{
    AddressFamily, MsgFlags, SockFlag, SockType, UnixAddr, bind, recvfrom, sendto, socket,
};
use std::fs::remove_file;
use std::os::fd::AsRawFd;
use std::path::Path;

fn main() {
    //     新しいsocketを作成する。
    let sock = socket(
        AddressFamily::Unix,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )
    .expect("failed to create socket");

    //     UNIXソケットで使用するserver_pathを作成する。
    let client_path = "/tmp/udp_client";
    if Path::new(client_path).exists() {
        remove_file(client_path).unwrap();
    }
    let client_addr = UnixAddr::new(client_path).unwrap();

    //     アドレスの紐付け
    bind(sock.as_raw_fd(), &client_addr).expect("failed to bind socket");

    // サーバーのアドレスを作成する。
    let server_path = "/tmp/udp_server";
    let server_addr = UnixAddr::new(server_path).unwrap();

    // メッセージをサーバーに送信する
    let request_message_str = String::from("Message to send to the server");
    let request_message_bytes = request_message_str.as_bytes();

    match sendto(
        sock.as_raw_fd(),
        &request_message_bytes,
        &server_addr,
        MsgFlags::empty(),
    ) {
        Ok(send_bytes) => {
            println!("Successfully sent {} bytes to the server", send_bytes);
        }
        Err(e) => {
            eprintln!("Failed to send message: {}", e);
            return;
        }
    }

    // サーバーからの受信待機
    let mut recv_buf = [0u8; 4096];

    println!("Waiting for a response from the server ...");

    match recvfrom::<UnixAddr>(sock.as_raw_fd(), &mut recv_buf) {
        Ok((bytes_received, sender_addr_option)) => {
            if bytes_received > 0 {
                let received_message_slice = &recv_buf[..bytes_received];

                if let Ok(message) = str::from_utf8(received_message_slice) {
                    println!(
                        "Received {} bytes from {:?}: {}",
                        bytes_received, sender_addr_option, message,
                    );
                } else {
                    println!("Received empty response from server")
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to receive message: {}", e);
        }
    }

    if Path::new(client_path).exists() {
        let _ = remove_file(client_path);
    }

    println!("Client finished");
}
