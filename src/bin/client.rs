use nix::sys::socket::sockopt::ReceiveTimeout;
use nix::sys::socket::{
    AddressFamily, MsgFlags, SockFlag, SockType, UnixAddr, connect, recv, send, setsockopt, socket,
};
use nix::sys::time::{TimeVal, TimeValLike};
use nix::unistd::close;
use std::os::fd::AsRawFd;
use std::thread::sleep;
use std::time::Duration;

const SOCKET_TIMEOUT_SECONDS: i64 = 2;
const BUFFER_SIZE: usize = 128;
const SOCKET_FILE_PATH: &str = "/tmp/socket_file";

fn main() {
    // TCP/IPソケットを作成する。
    let sock = socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .unwrap();

    // タイムアウトの設定
    let timeout = TimeVal::seconds(SOCKET_TIMEOUT_SECONDS);
    setsockopt(&sock, ReceiveTimeout, &timeout).unwrap();

    // サーバへの接続
    let sock_adr = UnixAddr::new(SOCKET_FILE_PATH).expect("failed to create socket address");
    connect(sock.as_raw_fd(), &sock_adr).expect("failed to connect socket 😢");

    // ユーザーに送信する文章を入力してもらうようprompをする
    println!("Enter a message to send to the server:");
    let mut message = String::new();
    std::io::stdin().read_line(&mut message).unwrap();

    // メッセージの送信
    let message_as_bytes = message.trim().as_bytes();
    send(sock.as_raw_fd(), message_as_bytes, MsgFlags::empty())
        .expect("failed to send message to server");

    // メッセージの受信
    let mut buf = [0u8; BUFFER_SIZE];
    //     タイムアウト時間分休ませる。
    sleep(Duration::from_secs(SOCKET_TIMEOUT_SECONDS as u64));

    let n = recv(sock.as_raw_fd(), &mut buf, MsgFlags::empty()).expect("failed to receive message");
    println!(
        "Server response: {}",
        std::str::from_utf8(&buf[..n]).unwrap()
    );

    close(sock).unwrap();
}
