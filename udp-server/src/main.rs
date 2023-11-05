use std::net::SocketAddr;

use clap::Parser;
use tokio::net::UdpSocket;

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

    let mut buff = [0u8; 65535];
    let mut recv = 0;
    let mut pkt_num = 0;
    let mut loop_num = 0;
    let mut now = tokio::time::Instant::now();
    while let Ok(n) = channel.recv(&mut buff).await {
        recv += n;
        if n != 0 {
            pkt_num += 1;
        }
        loop_num += 1;
        if now.elapsed().as_secs() > 1 {
            let recv_mbits = (recv / (1024 * 1024)) * 8;
            println!("Throughput: {recv_mbits} MBits/s, {pkt_num} pkts, {loop_num} loops");
            now = tokio::time::Instant::now();
            recv = 0;
            loop_num = 0;
            pkt_num = 0;
        }
    }
}
