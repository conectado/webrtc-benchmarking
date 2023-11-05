use std::net::SocketAddr;

use clap::Parser;
use tokio::net::UdpSocket;
const MESSAGE_SIZE: usize = 1236;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:0")]
    local: SocketAddr,
    #[arg(short, long)]
    remote: SocketAddr,
}

#[tokio::main]
async fn main() {
    let Args { local, remote } = Args::parse();
    let channel = UdpSocket::bind(local).await.unwrap();
    channel.connect(remote).await.unwrap();

    let bytes = [0; MESSAGE_SIZE];
    loop {
        let _ = channel.send(&bytes).await;
    }
}
