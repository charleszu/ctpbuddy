//! ZeroMQ ROUTER transport for the TD front.
//!
//! The world still owns all protocol state. This module only translates a
//! ROUTER identity plus one ZMTP message into the existing `WorldMsg` stream,
//! and translates outbound CB frames back to the same identity. ADMIN remains
//! on the TCP listener because it is an operator-only control plane.

use std::collections::HashMap;
use std::io::{self, Cursor};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;

use bytes::Bytes;
use futures::StreamExt;
use tokio::runtime::Builder;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::time::{timeout, Duration};
use zeromq::{PubSocket, RouterSocket, Socket, SocketEvent, SocketRecv, SocketSend, ZmqMessage};

use ctpbuddy_wire::frame::Frame;

use crate::{allocate_conn_id, Broadcast, WorldMsg, ZMQ_PUB_ADDR_ENV};

struct Outbound {
    identity: Vec<u8>,
    frame: Frame,
}

/// Start a ZeroMQ TD listener and wait until the endpoint has been bound.
///
/// `endpoint` accepts `zmq://tcp://host:port`, `zmq://ipc://path`, or the
/// convenient `zmq://host:port` form (which is normalized to TCP).
pub(crate) fn start(
    endpoint: &str,
    world_tx: Sender<WorldMsg>,
) -> io::Result<Option<tokio_mpsc::Sender<Broadcast>>> {
    let endpoint = normalize_endpoint(endpoint)?;
    let pub_endpoint = std::env::var(ZMQ_PUB_ADDR_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| normalize_pub_endpoint(&value))
        .transpose()?;
    let (broadcast_tx, broadcast_rx) = tokio_mpsc::channel(1024);
    let result_tx = pub_endpoint.as_ref().map(|_| broadcast_tx.clone());
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let runtime = match Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = ready_tx.send(Err(io::Error::other(format!("ZeroMQ runtime: {error}"))));
                return;
            }
        };
        let result = runtime.block_on(async move {
            let mut socket = RouterSocket::new();
            let mut monitor = socket.monitor();
            let mut publisher = if let Some(pub_endpoint) = pub_endpoint {
                let mut publisher = PubSocket::new();
                publisher.bind(&pub_endpoint).await.map_err(|error| {
                    io::Error::other(format!("ZeroMQ PUB bind {pub_endpoint}: {error}"))
                })?;
                Some(publisher)
            } else {
                None
            };
            socket
                .bind(&endpoint)
                .await
                .map_err(|error| io::Error::other(format!("ZeroMQ bind {endpoint}: {error}")))?;
            ready_tx.send(Ok(())).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "world loop stopped before ZeroMQ bind",
                )
            })?;
            run_loop(
                &mut socket,
                &mut monitor,
                world_tx.clone(),
                crate::configured_max_conns(),
                crate::shared_connection_counter(),
                &mut publisher,
                broadcast_rx,
            )
            .await;
            let _ = world_tx.send(WorldMsg::TransportClosed);
            Ok::<(), io::Error>(())
        });
        if let Err(error) = result {
            eprintln!("[ctpbuddy] ZeroMQ transport stopped: {error}");
        }
    });
    ready_rx.recv().map_err(|_| {
        io::Error::new(io::ErrorKind::BrokenPipe, "ZeroMQ transport thread stopped")
    })??;
    Ok(result_tx)
}

fn normalize_pub_endpoint(endpoint: &str) -> io::Result<String> {
    if endpoint.starts_with("zmq://") {
        normalize_endpoint(endpoint)
    } else if endpoint.starts_with("tcp://") || endpoint.starts_with("ipc://") {
        Ok(endpoint.to_string())
    } else {
        Ok(format!("tcp://{endpoint}"))
    }
}

pub(crate) fn normalize_endpoint(endpoint: &str) -> io::Result<String> {
    let rest = endpoint.strip_prefix("zmq://").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "ZeroMQ endpoint must start with zmq://",
        )
    })?;
    if rest.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ZeroMQ endpoint is empty",
        ));
    }
    if rest.starts_with("tcp://") || rest.starts_with("ipc://") {
        Ok(rest.to_string())
    } else {
        Ok(format!("tcp://{rest}"))
    }
}

async fn run_loop(
    socket: &mut RouterSocket,
    monitor: &mut futures::channel::mpsc::Receiver<SocketEvent>,
    world_tx: Sender<WorldMsg>,
    max_conns: usize,
    live: Arc<AtomicUsize>,
    publisher: &mut Option<PubSocket>,
    mut broadcast_rx: tokio_mpsc::Receiver<Broadcast>,
) {
    const OUTBOUND_QUEUE: usize = 1024;
    const SEND_TIMEOUT: Duration = Duration::from_secs(5);
    let (out_tx, mut out_rx) = tokio_mpsc::channel::<Outbound>(OUTBOUND_QUEUE);
    let mut identities: HashMap<Vec<u8>, u64> = HashMap::new();

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Ok(message) = incoming else { break };
                if message.len() != 2 {
                    continue;
                }
                let Some(identity_frame) = message.get(0) else { continue };
                let Some(payload_frame) = message.get(1) else { continue };
                let identity = identity_frame.to_vec();
                if !identities.contains_key(&identity)
                    && live.fetch_add(1, Ordering::SeqCst) >= max_conns
                {
                    live.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }
                let id = *identities.entry(identity.clone()).or_insert_with(|| {
                    let id = allocate_conn_id();
                    let (writer_tx, writer_rx) = mpsc::channel::<Frame>();
                    let _ = world_tx.send(WorldMsg::Opened { id, writer: writer_tx, is_admin: false });
                    let out_tx = out_tx.clone();
                    let identity_for_writer = identity.clone();
                    thread::spawn(move || {
                        for frame in writer_rx {
                            if out_tx
                                .blocking_send(Outbound {
                                    identity: identity_for_writer.clone(),
                                    frame,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                    });
                    id
                });
                let mut cursor = Cursor::new(payload_frame.as_ref());
                match Frame::read_from(&mut cursor) {
                    Ok(Some(frame)) if cursor.position() as usize == payload_frame.len() => {
                        if world_tx.send(WorldMsg::Inbound { id, frame }).is_err() {
                            break;
                        }
                    }
                    _ => {
                        if identities.remove(&identity).is_some() {
                            live.fetch_sub(1, Ordering::SeqCst);
                        }
                        let _ = world_tx.send(WorldMsg::Closed { id });
                    }
                }
            }
            Some(outbound) = out_rx.recv() => {
                let message = ZmqMessage::try_from(vec![
                    Bytes::from(outbound.identity),
                    Bytes::from(outbound.frame.encode()),
                ]);
                if let Ok(message) = message {
                    if timeout(SEND_TIMEOUT, socket.send(message)).await.is_err() {
                        // A disconnected peer is reported by the monitor; a
                        // failed send must not terminate other identities.
                    }
                }
            }
            event = monitor.next() => {
                match event {
                    Some(SocketEvent::Disconnected(identity)) => {
                        let identity: Vec<u8> = identity.into();
                        if let Some(id) = identities.remove(&identity) {
                            live.fetch_sub(1, Ordering::SeqCst);
                            let _ = world_tx.send(WorldMsg::Closed { id });
                        }
                    }
                    Some(_) => {}
                    None => break,
                }
            }
            broadcast = broadcast_rx.recv(), if publisher.is_some() => {
                let Some(broadcast) = broadcast else { continue };
                let message = ZmqMessage::try_from(vec![
                    Bytes::from(broadcast.topic),
                    Bytes::from(broadcast.frame.encode()),
                ]);
                if let (Some(publisher), Ok(message)) = (publisher.as_mut(), message) {
                    let _ = timeout(SEND_TIMEOUT, publisher.send(message)).await;
                }
            }
        }
    }
    for (_, id) in identities {
        live.fetch_sub(1, Ordering::SeqCst);
        let _ = world_tx.send(WorldMsg::Closed { id });
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_endpoint, normalize_pub_endpoint};
    use bytes::Bytes;
    use ctpbuddy_wire::frame::Frame;
    use tokio::runtime::Builder;
    use tokio::time::{sleep, timeout, Duration};
    use zeromq::{
        DealerSocket, PubSocket, RouterSocket, Socket, SocketRecv, SocketSend, SubSocket,
        ZmqMessage,
    };

    #[test]
    fn normalize_zmq_endpoints() {
        assert_eq!(
            normalize_endpoint("zmq://127.0.0.1:5560").unwrap(),
            "tcp://127.0.0.1:5560"
        );
        assert_eq!(
            normalize_endpoint("zmq://tcp://127.0.0.1:5560").unwrap(),
            "tcp://127.0.0.1:5560"
        );
        assert_eq!(
            normalize_endpoint("zmq://ipc:///tmp/ctpbuddy").unwrap(),
            "ipc:///tmp/ctpbuddy"
        );
        assert!(normalize_endpoint("127.0.0.1:5560").is_err());
        assert_eq!(
            normalize_pub_endpoint("zmq://127.0.0.1:5561").unwrap(),
            "tcp://127.0.0.1:5561"
        );
        assert_eq!(
            normalize_pub_endpoint("ipc:///tmp/ctpbuddy-pub").unwrap(),
            "ipc:///tmp/ctpbuddy-pub"
        );
    }

    #[test]
    fn router_dealer_roundtrip_preserves_cb_frame() {
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let mut router = RouterSocket::new();
            let endpoint = router.bind("tcp://127.0.0.1:0").await.unwrap().to_string();
            let mut dealer = DealerSocket::new();
            dealer.connect(&endpoint).await.unwrap();
            let request = Frame::new(0x0101, 9, b"hello".to_vec());
            dealer.send(request.encode().into()).await.unwrap();
            let incoming = router.recv().await.unwrap();
            assert_eq!(incoming.len(), 2);
            let identity = incoming.get(0).unwrap().clone();
            let payload = incoming.get(1).unwrap();
            let mut cursor = std::io::Cursor::new(payload.as_ref());
            let decoded = Frame::read_from(&mut cursor).unwrap().unwrap();
            assert_eq!(decoded.msg_type, 0x0101);
            assert_eq!(decoded.req_id, 9);
            assert_eq!(decoded.payload, b"hello");
            let response = Frame::new(0x0102, 9, b"world".to_vec());
            router
                .send(ZmqMessage::try_from(vec![identity, Bytes::from(response.encode())]).unwrap())
                .await
                .unwrap();
            let outgoing = dealer.recv().await.unwrap();
            let mut cursor = std::io::Cursor::new(outgoing.get(0).unwrap().as_ref());
            let decoded = Frame::read_from(&mut cursor).unwrap().unwrap();
            assert_eq!(decoded.msg_type, 0x0102);
            assert_eq!(decoded.payload, b"world");
        });
    }

    #[test]
    fn pub_sub_roundtrip_preserves_topic_and_cb_frame() {
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let mut publisher = PubSocket::new();
            let endpoint = publisher
                .bind("tcp://127.0.0.1:0")
                .await
                .unwrap()
                .to_string();
            let mut subscriber = SubSocket::new();
            subscriber.connect(&endpoint).await.unwrap();
            subscriber.subscribe("md/").await.unwrap();
            sleep(Duration::from_millis(50)).await;
            let frame = Frame::new(0x1022, 0, b"tick".to_vec());
            publisher
                .send(
                    ZmqMessage::try_from(vec![
                        Bytes::from("md/cu2501"),
                        Bytes::from(frame.encode()),
                    ])
                    .unwrap(),
                )
                .await
                .unwrap();
            let message = timeout(Duration::from_secs(2), subscriber.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(message.len(), 2);
            assert_eq!(message.get(0).unwrap().as_ref(), b"md/cu2501");
            let mut cursor = std::io::Cursor::new(message.get(1).unwrap().as_ref());
            let decoded = Frame::read_from(&mut cursor).unwrap().unwrap();
            assert_eq!(decoded.msg_type, 0x1022);
            assert_eq!(decoded.payload, b"tick");
        });
    }
}
