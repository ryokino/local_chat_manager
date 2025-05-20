use nix::sys::socket::sockopt::ReceiveTimeout;
use nix::sys::socket::{
    accept, connect, recv, send, setsockopt, socket, AddressFamily, MsgFlags, SockFlag, SockType,
    UnixAddr,
};
use nix::sys::time::{TimeVal, TimeValLike};
use nix::unistd::close;
use std::fs::remove_file;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // TCP/IPソケットを作成する。
    let sock = socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .unwrap();

    // 2秒間の受信タイムアウトの設定
    let timeout = TimeVal::seconds(2);
    setsockopt(&sock, ReceiveTimeout, &timeout).unwrap();

    // サーバへの接続
    let server_address = "/tmp/socket_file";

    // 以前のソケットが残っていたら削除する
    if Path::new(server_address).exists() {
        remove_file(server_address).expect("failed to remove socket file");
    }
    let sock_adr = UnixAddr::new(server_address).expect("failed to create socket address");
    connect(sock.as_raw_fd(), &sock_adr).expect("failed to connect socket");

    // メッセージの送信
    let message_as_bytes = b"Sending a message to the server";
    send(sock.as_raw_fd(), message_as_bytes, MsgFlags::empty())
        .expect("failed to send message to server");

    // メッセージの受信
    let mut buf = [0u8; 32];

    //     2秒休ませる。
    sleep(Duration::from_secs(2));

    loop {
        let connection_fd = accept(sock.as_raw_fd()).unwrap();

        loop {
            let data_from_server = recv(connection_fd, &mut buf, MsgFlags::empty()).unwrap();
            let message_from_server = String::from_utf8_lossy(&buf);

            if data_from_server != 0 {
                let response_from_server = format!("Server response: {}", message_from_server);

                println!("{}", response_from_server);
            } else {
                println!("Server closed connection");
                break;
            }
        }
        //     接続がなくなった場合(2秒以上でTimeoutError)
        println!("Closing connection");
        close(connection_fd).expect("failed to close connection");
        break;
    }
}