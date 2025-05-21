use nix::sys::socket::{
    accept, bind, getpeername, listen, recv, send, socket, AddressFamily, Backlog,
    MsgFlags, SockFlag, SockType, UnixAddr,
};
use nix::unistd::close;
use std::fs::remove_file;
use std::os::fd::AsRawFd;
use std::path::Path;

fn main() {
    let mut buf = [0u8; 32];
    // 3. クライアントからの接続があるとサーバーはその接続を受け入れて通信を始める
    // 4. サーバはクライアントからもうメッセージが送られないと判断すると接続を終了する

    // server_addressは UNIXソケットで通信をする際の出入り口となっている。
    let server_address = "/tmp/socket_file";

    // 前に同じソケットが残っていたら以下でbindする(窓口を設定する)ことができないのでそれを削除する
    if Path::new(server_address).exists() {
        remove_file(server_address).unwrap();
    }

    // UNIXソケットの作成 (https://docs.rs/nix/latest/nix/sys/socket/fn.socket.html)
    let sock = socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .expect("failed to create socket");

    // アドレスの作成
    let sock_adr = UnixAddr::new(server_address).unwrap();
    //     ソケットをバインドする
    bind(sock.as_raw_fd(), &sock_adr).expect("failed to bind socket");

    // clientからの接続を listenしている
    // backlog: 接続リクエストの最大数
    listen(&sock, Backlog::new(128).unwrap()).expect("failed to listen on socket");

    // 無限ループでクライアントからの接続を待ち続ける
    loop {
        // クライアントからの接続を受け入れる
        let connection_fd = accept(sock.as_raw_fd()).unwrap(); // ここで connection_fd の方が RawFdに固定されている
        let peer: UnixAddr = getpeername(connection_fd).expect("failed to get peername");

        if let Some(path) = peer.path() {
            if path.as_os_str().is_empty() {
                println!("connection from unnamed peer");
            } else {
                println!("connection from {}", path.display());
            }
        } else {
            println!("connection from unknown peer (no path associated)");
        }

        // サーバが新しいデータを待ち続けるための無限ループ
        loop {
            // let data = read(connection_fd, &mut buf).unwrap(); // コンパイルエラー: connection_idがRawFdだから。Fdにしたい
            let data = recv(connection_fd, &mut buf, MsgFlags::empty()).unwrap();

            let message = String::from_utf8_lossy(&buf[..data]);

            println!("Received: {}", message);

            if data != 0 {
                let response = format!("Processing: {}", message);
                println!("{}", response);
                //     バイナリ形式に直してからクライアントに送り返す
                let bytes = response.as_bytes();

                let mut sent = 0;
                while sent < bytes.len() {
                    let n = send(connection_fd, &bytes[sent..], MsgFlags::empty())
                        .expect("failed to send message");
                    sent += n;
                }
            } else {
                if let Some(path) = peer.path() {
                    if path.as_os_str().is_empty() {
                        println!("no data from unnamed peer");
                    } else {
                        println!("no data from {}", path.display());
                    }
                } else {
                    println!("no data from unknown peer (no path associated)");
                }
                break;
            }
        }
        // // 接続がないと判断した場合
        println!("Closing connection");
        close(connection_fd).expect("failed to close connection");
        // break;
    }
}