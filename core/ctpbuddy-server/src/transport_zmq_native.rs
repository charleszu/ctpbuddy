//! Optional native libzmq adapter for real CURVE encryption. All native handles
//! belong to one transport thread; loading/options/bind failures abort startup.
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_long, c_void, CStr, CString};
use std::io::{self, Cursor};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Instant;

use crate::{allocate_conn_id, flow::WindowBudget, Broadcast, WorldMsg};
use ctpbuddy_wire::Frame;
use libloading::Library;
use tokio::sync::mpsc as queue;

const DONTWAIT: c_int = 1;
const SNDMORE: c_int = 2;
const POLLIN: i16 = 1;

#[repr(C, align(8))]
struct Message([u8; 64]); // libzmq stable zmq_msg_t ABI

#[repr(C)]
struct PollItem {
    socket: *mut c_void,
    #[cfg(windows)]
    fd: usize, // SOCKET is pointer-sized on Windows
    #[cfg(not(windows))]
    fd: c_int,
    events: i16,
    revents: i16,
}

struct Native {
    _library: Library,
    ctx_new: unsafe extern "C" fn() -> *mut c_void,
    ctx_term: unsafe extern "C" fn(*mut c_void) -> c_int,
    socket: unsafe extern "C" fn(*mut c_void, c_int) -> *mut c_void,
    close: unsafe extern "C" fn(*mut c_void) -> c_int,
    bind: unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int,
    connect: unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int,
    option: unsafe extern "C" fn(*mut c_void, c_int, *const c_void, usize) -> c_int,
    monitor: unsafe extern "C" fn(*mut c_void, *const c_char, c_int) -> c_int,
    send: unsafe extern "C" fn(*mut c_void, *const c_void, usize, c_int) -> c_int,
    poll: unsafe extern "C" fn(*mut PollItem, c_int, c_long) -> c_int,
    msg_init: unsafe extern "C" fn(*mut Message) -> c_int,
    msg_close: unsafe extern "C" fn(*mut Message) -> c_int,
    msg_recv: unsafe extern "C" fn(*mut Message, *mut c_void, c_int) -> c_int,
    msg_data: unsafe extern "C" fn(*mut Message) -> *mut c_void,
    msg_size: unsafe extern "C" fn(*const Message) -> usize,
    msg_more: unsafe extern "C" fn(*const Message) -> c_int,
    msg_get: unsafe extern "C" fn(*const Message, c_int) -> c_int,
    errno: unsafe extern "C" fn() -> c_int,
    strerror: unsafe extern "C" fn(c_int) -> *const c_char,
    has: unsafe extern "C" fn(*const c_char) -> c_int,
}

impl Native {
    fn load() -> io::Result<Self> {
        #[cfg(windows)]
        let defaults = ["libzmq.dll", "zmq.dll"];
        #[cfg(not(windows))]
        let defaults = ["libzmq.so.5", "libzmq.so", "libzmq.dylib"];
        let configured = std::env::var("CTPBUDDY_ZMQ_LIBRARY").ok();
        let names: Vec<&str> = configured
            .as_deref()
            .map(|v| vec![v])
            .unwrap_or_else(|| defaults.to_vec());
        let library = names
            .iter()
            .find_map(|name| {
                // SAFETY: caller supplies a trusted native libzmq library. Function
                // signatures below follow the public libzmq 4.x C ABI.
                unsafe { Library::new(name).ok() }
            })
            .ok_or_else(|| {
                io::Error::other("CURVE requires native libzmq; set CTPBUDDY_ZMQ_LIBRARY")
            })?;
        macro_rules! symbol {
            ($name:ident) => {
                unsafe {
                    *library
                        .get(concat!("zmq_", stringify!($name), "\0").as_bytes())
                        .map_err(|_| {
                            io::Error::other(concat!("libzmq missing ", stringify!($name)))
                        })?
                }
            };
        }
        let native = Self {
            ctx_new: symbol!(ctx_new),
            ctx_term: symbol!(ctx_term),
            socket: symbol!(socket),
            close: symbol!(close),
            bind: symbol!(bind),
            connect: symbol!(connect),
            option: symbol!(setsockopt),
            monitor: symbol!(socket_monitor),
            send: symbol!(send),
            poll: symbol!(poll),
            msg_init: symbol!(msg_init),
            msg_close: symbol!(msg_close),
            msg_recv: symbol!(msg_recv),
            msg_data: symbol!(msg_data),
            msg_size: symbol!(msg_size),
            msg_more: symbol!(msg_more),
            msg_get: symbol!(msg_get),
            errno: symbol!(errno),
            strerror: symbol!(strerror),
            has: symbol!(has),
            _library: library,
        };
        if unsafe { (native.has)(c"curve".as_ptr()) } != 1 {
            return Err(io::Error::other("native libzmq was built without CURVE"));
        }
        Ok(native)
    }

    fn error(&self, action: &str) -> io::Error {
        // SAFETY: strerror returns a static NUL-terminated string for errno.
        let detail = unsafe { CStr::from_ptr((self.strerror)((self.errno)())) }.to_string_lossy();
        io::Error::other(format!("libzmq {action}: {detail}"))
    }

    fn option<T>(&self, socket: *mut c_void, key: c_int, value: &T) -> io::Result<()> {
        if unsafe {
            (self.option)(
                socket,
                key,
                (value as *const T).cast(),
                std::mem::size_of::<T>(),
            )
        } != 0
        {
            return Err(self.error("socket option"));
        }
        Ok(())
    }

    fn recv(&self, socket: *mut c_void) -> io::Result<Option<(Vec<Vec<u8>>, c_int)>> {
        let mut parts = Vec::new();
        let mut fd = -1;
        let mut invalid = false;
        loop {
            let mut msg = Message([0; 64]);
            if unsafe { (self.msg_init)(&mut msg) } != 0 {
                return Err(self.error("message init"));
            }
            let received = unsafe { (self.msg_recv)(&mut msg, socket, DONTWAIT) };
            if received < 0 {
                let error = unsafe { (self.errno)() };
                unsafe {
                    (self.msg_close)(&mut msg);
                }
                if error == 11 && parts.is_empty() {
                    return Ok(None);
                } // EAGAIN
                return Err(self.error("receive"));
            }
            let len = unsafe { (self.msg_size)(&msg) };
            let more = unsafe { (self.msg_more)(&msg) } != 0;
            let source_fd = unsafe { (self.msg_get)(&msg, 2) }; // ZMQ_SRCFD
            if source_fd >= 0 {
                fd = source_fd;
            }
            if len > ctpbuddy_wire::frame::MAX_PAYLOAD + ctpbuddy_wire::frame::HEADER_LEN
                || parts.len() >= 8
            {
                invalid = true;
            } else if !invalid {
                // SAFETY: msg_data is valid for exactly msg_size bytes until
                // msg_close. Copy before closing the native message.
                parts.push(
                    unsafe {
                        std::slice::from_raw_parts((self.msg_data)(&mut msg).cast::<u8>(), len)
                    }
                    .to_vec(),
                );
            }
            unsafe {
                (self.msg_close)(&mut msg);
            }
            if !more {
                break;
            }
        }
        if invalid {
            parts.clear();
        }
        Ok(Some((parts, fd)))
    }

    fn send(&self, socket: *mut c_void, parts: &[&[u8]]) -> io::Result<()> {
        for (index, part) in parts.iter().enumerate() {
            let flags = if index + 1 < parts.len() { SNDMORE } else { 0 };
            if unsafe { (self.send)(socket, part.as_ptr().cast(), part.len(), flags) } < 0 {
                return Err(self.error("send"));
            }
        }
        Ok(())
    }
}

struct Sockets {
    native: Native,
    ctx: *mut c_void,
    handles: Vec<*mut c_void>,
}

impl Drop for Sockets {
    fn drop(&mut self) {
        for socket in &self.handles {
            unsafe {
                (self.native.close)(*socket);
            }
        }
        unsafe {
            (self.native.ctx_term)(self.ctx);
        }
    }
}

impl Sockets {
    fn new() -> io::Result<Self> {
        let native = Native::load()?;
        let ctx = unsafe { (native.ctx_new)() };
        if ctx.is_null() {
            return Err(native.error("context"));
        }
        Ok(Self {
            native,
            ctx,
            handles: Vec::new(),
        })
    }

    fn socket(&mut self, kind: c_int, secret: Option<&str>) -> io::Result<*mut c_void> {
        let socket = unsafe { (self.native.socket)(self.ctx, kind) };
        if socket.is_null() {
            return Err(self.native.error("socket"));
        }
        self.handles.push(socket);
        self.native.option(socket, 17, &0i32)?; // LINGER
        self.native.option(socket, 23, &1024i32)?; // SNDHWM
        self.native.option(socket, 24, &1024i32)?; // RCVHWM
        self.native.option(socket, 28, &100i32)?; // SNDTIMEO (don't starve monitor)
        self.native.option(
            socket,
            22,
            &((ctpbuddy_wire::frame::MAX_PAYLOAD + 13) as i64),
        )?;
        if let Some(secret) = secret {
            let domain: &[u8] = if kind == 6 {
                b"ctpbuddy-td"
            } else {
                b"ctpbuddy-pub"
            };
            if unsafe { (self.native.option)(socket, 55, domain.as_ptr().cast(), domain.len()) }
                != 0
            {
                return Err(self.native.error("ZAP domain"));
            }
            self.native.option(socket, 47, &1i32)?; // CURVE_SERVER
            let rc =
                unsafe { (self.native.option)(socket, 49, secret.as_ptr().cast(), secret.len()) };
            if rc != 0 {
                return Err(self.native.error("CURVE secret key"));
            }
        }
        Ok(socket)
    }

    fn bind(&self, socket: *mut c_void, endpoint: &str) -> io::Result<()> {
        let endpoint = CString::new(endpoint).map_err(io::Error::other)?;
        if unsafe { (self.native.bind)(socket, endpoint.as_ptr()) } != 0 {
            return Err(self.native.error("bind"));
        }
        Ok(())
    }
}

struct Peer {
    id: u64,
    fd: c_int,
}
struct Outbound {
    identity: Vec<u8>,
    frame: Frame,
}

pub(crate) fn start(
    endpoint: String,
    pub_endpoint: Option<String>,
    secret: String,
    clients: Vec<[u8; 32]>,
    connect_freq: u32,
    world_tx: Sender<WorldMsg>,
) -> io::Result<Option<queue::Sender<Broadcast>>> {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (broadcast_tx, mut broadcasts) = queue::channel::<Broadcast>(crate::ZMQ_PUB_QUEUE_CAPACITY);
    let result = pub_endpoint.as_ref().map(|_| broadcast_tx);
    thread::spawn(move || {
        let prepared = (|| {
            let mut sockets = Sockets::new()?;
            let zap = sockets.socket(4, None)?; // REP, ZAP 1.0 client-key authorization
            sockets.bind(zap, "inproc://zeromq.zap.01")?;
            let router = sockets.socket(6, Some(&secret))?; // ROUTER
            sockets.native.option(router, 33, &1i32)?; // ROUTER_MANDATORY
            let monitor = sockets.socket(0, None)?; // PAIR
            let monitor_addr = c"inproc://ctpbuddy-curve-monitor";
            if unsafe { (sockets.native.monitor)(router, monitor_addr.as_ptr(), 0x200) } != 0 {
                return Err(sockets.native.error("monitor"));
            }
            if unsafe { (sockets.native.connect)(monitor, monitor_addr.as_ptr()) } != 0 {
                return Err(sockets.native.error("monitor connect"));
            }
            let publisher = pub_endpoint
                .as_ref()
                .map(|endpoint| {
                    let socket = sockets.socket(1, Some(&secret))?;
                    sockets.bind(socket, endpoint)?;
                    Ok::<_, io::Error>(socket)
                })
                .transpose()?;
            sockets.bind(router, &endpoint)?;
            Ok::<_, io::Error>((sockets, router, monitor, publisher, zap))
        })();
        let (sockets, router, monitor, publisher, zap) = match prepared {
            Ok(prepared) => {
                let _ = ready_tx.send(Ok(()));
                prepared
            }
            Err(error) => {
                let _ = ready_tx.send(Err(error));
                return;
            }
        };
        let live = crate::shared_connection_counter();
        let mut peers: HashMap<Vec<u8>, Peer> = HashMap::new();
        let mut connections = WindowBudget::default();
        let (out_tx, mut outgoing) = queue::channel::<Outbound>(1024);
        let result = (|| -> io::Result<()> {
            loop {
                let mut items = [
                    PollItem {
                        socket: zap,
                        fd: 0,
                        events: POLLIN,
                        revents: 0,
                    },
                    PollItem {
                        socket: router,
                        fd: 0,
                        events: POLLIN,
                        revents: 0,
                    },
                    PollItem {
                        socket: monitor,
                        fd: 0,
                        events: POLLIN,
                        revents: 0,
                    },
                ];
                if unsafe { (sockets.native.poll)(items.as_mut_ptr(), 3, 10) } < 0 {
                    return Err(sockets.native.error("poll"));
                }
                if items[0].revents & POLLIN != 0 {
                    if let Some((parts, _)) = sockets.native.recv(zap)? {
                        if parts.len() >= 2 {
                            let allowed = parts.len() == 7
                                && parts[0] == b"1.0"
                                && (parts[2] == b"ctpbuddy-td" || parts[2] == b"ctpbuddy-pub")
                                && parts[5] == b"CURVE"
                                && clients.iter().any(|key| parts[6].as_slice() == key)
                                && (parts[2] != b"ctpbuddy-td"
                                    || crate::flow::connect_gate(
                                        &mut connections,
                                        Instant::now(),
                                        connect_freq,
                                        &world_tx,
                                        "curve",
                                    ));
                            let (code, text): (&[u8], &[u8]) = if allowed {
                                (b"200", b"OK")
                            } else {
                                (b"400", b"Unauthorized")
                            };
                            sockets.native.send(
                                zap,
                                &[
                                    b"1.0",
                                    &parts[1],
                                    code,
                                    text,
                                    if allowed { b"authorized" } else { b"" },
                                    b"",
                                ],
                            )?;
                        }
                    }
                }
                if items[2].revents & POLLIN != 0 {
                    if let Some((parts, _)) = sockets.native.recv(monitor)? {
                        if parts.len() == 2 && parts[0].len() == 6 {
                            let fd = i32::from_le_bytes(parts[0][2..6].try_into().unwrap());
                            let removed: Vec<_> = peers
                                .iter()
                                .filter(|(_, peer)| peer.fd == fd)
                                .map(|(key, _)| key.clone())
                                .collect();
                            for identity in removed {
                                if let Some(peer) = peers.remove(&identity) {
                                    live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                                    let _ = world_tx.send(WorldMsg::Closed { id: peer.id });
                                }
                            }
                        }
                    }
                }
                if items[1].revents & POLLIN != 0 {
                    if let Some((parts, fd)) = sockets.native.recv(router)? {
                        if parts.len() != 2 {
                            continue;
                        }
                        let identity = &parts[0];
                        let mut cursor = Cursor::new(&parts[1]);
                        let frame = match Frame::read_from(&mut cursor) {
                            Ok(Some(frame)) if cursor.position() as usize == parts[1].len() => {
                                frame
                            }
                            _ => continue,
                        };
                        if !peers.contains_key(identity) {
                            if live.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                                >= crate::configured_max_conns()
                            {
                                live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                                continue;
                            }
                            let id = allocate_conn_id();
                            let (writer, receiver) = mpsc::channel();
                            if world_tx
                                .send(WorldMsg::Opened {
                                    id,
                                    writer,
                                    is_admin: false,
                                })
                                .is_err()
                            {
                                live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                                return Ok(());
                            }
                            let out_tx = out_tx.clone();
                            let identity = identity.clone();
                            thread::spawn(move || {
                                for frame in receiver {
                                    if out_tx
                                        .blocking_send(Outbound {
                                            identity: identity.clone(),
                                            frame,
                                        })
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            });
                            peers.insert(parts[0].clone(), Peer { id, fd });
                        }
                        let id = peers[identity].id;
                        if world_tx.send(WorldMsg::Inbound { id, frame }).is_err() {
                            return Ok(());
                        }
                    }
                }
                for _ in 0..64 {
                    let Ok(out) = outgoing.try_recv() else {
                        break;
                    };
                    let bytes = out.frame.encode();
                    // ROUTER_MANDATORY reports disconnected/HWM peers; don't
                    // turn one peer's send failure into global transport loss.
                    if sockets
                        .native
                        .send(router, &[&out.identity, &bytes])
                        .is_err()
                    {
                        if let Some(peer) = peers.remove(&out.identity) {
                            live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                            let _ = world_tx.send(WorldMsg::Closed { id: peer.id });
                        }
                    }
                }
                if let Some(publisher) = publisher {
                    for _ in 0..64 {
                        let Ok(event) = broadcasts.try_recv() else {
                            break;
                        };
                        let _ = sockets
                            .native
                            .send(publisher, &[event.topic.as_bytes(), &event.frame.encode()]);
                    }
                }
            }
        })();
        for (_, peer) in peers {
            live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            let _ = world_tx.send(WorldMsg::Closed { id: peer.id });
        }
        if let Err(error) = result {
            eprintln!("[ctpbuddy] CURVE transport stopped: {error}");
        }
        let _ = world_tx.send(WorldMsg::TransportClosed);
    });
    ready_rx
        .recv()
        .map_err(|_| io::Error::other("CURVE transport stopped before startup"))??;
    Ok(result)
}
