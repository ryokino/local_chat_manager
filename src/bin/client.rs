use nix::sys::socket::sockopt::ReceiveTimeout;
use nix::sys::socket::{
    AddressFamily, MsgFlags, SockFlag, SockType, UnixAddr, connect, recv, send, setsockopt, socket,
};
use nix::sys::time::{TimeVal, TimeValLike};
use nix::unistd::close;
use std::os::fd::AsRawFd;
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
    let sock_adr = UnixAddr::new(server_address).expect("failed to create socket address");
    connect(sock.as_raw_fd(), &sock_adr).expect("failed to connect socket 😢");

    // メッセージの送信
    let message_as_bytes = b"Hello from client!!!!";
    send(sock.as_raw_fd(), message_as_bytes, MsgFlags::empty())
        .expect("failed to send message to server");

    // メッセージの受信
    let mut buf = [0u8; 32];
    //     2秒休ませる。
    sleep(Duration::from_secs(2));

    let n = recv(sock.as_raw_fd(), &mut buf, MsgFlags::empty()).expect("failed to receive message");
    println!(
        "Server response: {}",
        std::str::from_utf8(&buf[..n]).unwrap()
    );

    close(sock).unwrap();
}
