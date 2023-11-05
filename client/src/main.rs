use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine,
        setting_engine::SettingEngine, APIBuilder,
    },
    data_channel::data_channel_init::RTCDataChannelInit,
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

const MESSAGE_SIZE: usize = 1236;

#[tokio::main]
async fn main() -> Result<()> {
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;
    let mut s = SettingEngine::default();
    s.detach_data_channels();
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .with_setting_engine(s)
        .build();
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    let data_channel = peer_connection
        .create_data_channel(
            "data",
            Some(RTCDataChannelInit {
                ordered: Some(false),
                max_packet_life_time: Some(1),
                ..Default::default()
            }),
        )
        .await?;
    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            println!("Peer Connection has gone to failed exiting");
            let _ = done_tx.try_send(());
        }

        Box::pin(async {})
    }));

    let d = Arc::clone(&data_channel);
    data_channel.on_open(Box::new(move || {
        println!("Data channel '{}'-'{}' open.", d.label(), d.id());

        let d2 = Arc::clone(&d);
        Box::pin(async move {
            println!(
                "candidate pair: {:?}",
                d.transport()
                    .await
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .transport()
                    .ice_transport()
                    .get_selected_candidate_pair()
                    .await
                    .unwrap()
            );
            let raw = match d2.detach().await {
                Ok(raw) => raw,
                Err(err) => {
                    println!("data channel detach got err: {err}");
                    return;
                }
            };

            tokio::spawn(async move {
                let _ = write_loop(raw).await;
            });
        })
    }));

    let offer = peer_connection.create_offer(None).await?;
    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(offer).await?;
    let _ = gather_complete.recv().await;

    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        let b64 = signal::encode(&json_str);
        println!("{b64}");
    } else {
        println!("generate local_description failed!");
    }

    // Wait for the answer to be pasted
    let line = signal::must_read_stdin()?;
    let desc_data = signal::decode(line.as_str())?;
    let answer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;

    // Apply the answer as the remote description
    peer_connection.set_remote_description(answer).await?;

    println!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };

    peer_connection.close().await?;

    Ok(())
}

// write_loop shows how to write to the datachannel directly
async fn write_loop(d: Arc<webrtc::data::data_channel::DataChannel>) -> Result<()> {
    let bytes = [0; MESSAGE_SIZE];
    let bytes = Bytes::copy_from_slice(&bytes);
    loop {
        let _ = d.write(&bytes).await;
    }
}
