use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use webrtc::{
    api::APIBuilder,
    ice_transport::{
        ice_candidate::RTCIceCandidate, ice_gatherer::RTCIceGatherOptions,
        ice_parameters::RTCIceParameters, ice_role::RTCIceRole, ice_server::RTCIceServer,
        ice_transport_state::RTCIceTransportState,
    },
    util::Conn,
};

const MESSAGE_SIZE: usize = 1236;

#[tokio::main]
async fn main() -> Result<()> {
    let ice_options = RTCIceGatherOptions {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create an API object
    let api = APIBuilder::new().build();

    // Create the ICE gatherer
    let gatherer = Arc::new(api.new_ice_gatherer(ice_options)?);

    // Construct the ICE transport
    let ice = Arc::new(api.new_ice_transport(Arc::clone(&gatherer)));

    let (gather_finished_tx, mut gather_finished_rx) = tokio::sync::mpsc::channel::<()>(1);
    gatherer.on_local_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
        let gather_finished_tx = gather_finished_tx.clone();
        Box::pin(async move {
            if c.is_none() {
                gather_finished_tx.send(()).await.unwrap();
            }
        })
    }));

    // Gather candidates
    gatherer.gather().await?;

    let _ = gather_finished_rx.recv().await;

    let ice_candidates = gatherer.get_local_candidates().await?;

    let ice_parameters = gatherer.get_local_parameters().await?;

    let local_signal = Signal {
        ice_candidates,
        ice_parameters,
    };

    // Exchange the information
    let json_str = serde_json::to_string(&local_signal)?;
    let b64 = signal::encode(&json_str);
    println!("{b64}");

    let line = signal::must_read_stdin()?;
    let json_str = signal::decode(line.as_str())?;
    let remote_signal = serde_json::from_str::<Signal>(&json_str)?;

    let ice_role = RTCIceRole::Controlling;

    ice.set_remote_candidates(&remote_signal.ice_candidates)
        .await?;

    let (ice_connected_tx, mut ice_connected_rx) = tokio::sync::mpsc::channel::<()>(1);
    ice.on_connection_state_change(Box::new(move |state| {
        println!("{state:?}");
        let ice_connected_tx = ice_connected_tx.clone();
        Box::pin(async move {
            if state == RTCIceTransportState::Connected {
                ice_connected_tx.send(()).await.unwrap();
            }
        })
    }));

    ice.on_selected_candidate_pair_change(Box::new(move |pair| {
        println!("{pair:?}");
        Box::pin(async move {})
    }));

    // Start the ICE transport
    ice.start(&remote_signal.ice_parameters, Some(ice_role))
        .await?;

    println!("Awaiting connection...");
    ice_connected_rx.recv().await;
    println!("Connected!");
    let ep = ice.new_endpoint(Box::new(|_| true)).await.unwrap();
    let bytes = [0; MESSAGE_SIZE];
    loop {
        let _ = ep.send(&bytes).await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Signal {
    #[serde(rename = "iceCandidates")]
    ice_candidates: Vec<RTCIceCandidate>, //   `json:"iceCandidates"`

    #[serde(rename = "iceParameters")]
    ice_parameters: RTCIceParameters, //    `json:"iceParameters"`
}
