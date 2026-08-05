#![forbid(unsafe_code)]

use serde::Serialize;
use slopity_core::{NetworkScope, RuntimeKind, ServerId, ServerProfile, ServerState};
use std::{
    collections::{HashMap, VecDeque},
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use thiserror::Error;

const MAX_LOG_ENTRIES: usize = 200;
const MAX_REQUEST_BYTES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HttpLogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpServerLog {
    pub sequence: u64,
    pub level: HttpLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpServerSnapshot {
    pub server_id: ServerId,
    pub state: ServerState,
    pub bind_address: String,
    pub urls: Vec<String>,
    pub request_count: u64,
    pub logs: Vec<HttpServerLog>,
    pub last_error: Option<String>,
}

#[derive(Debug, Error)]
pub enum HttpServerError {
    #[error("profile {0} does not use the built-in HTTP runtime")]
    UnsupportedRuntime(String),
    #[error("profile {0} is disabled; enable it before starting")]
    Disabled(String),
    #[error("built-in HTTP server is already running: {0}")]
    AlreadyRunning(String),
    #[error("built-in HTTP server is not running: {0}")]
    NotRunning(String),
    #[error("failed to bind {address}: {source}")]
    Bind {
        address: String,
        #[source]
        source: io::Error,
    },
    #[error("failed to configure the HTTP listener: {0}")]
    Listener(io::Error),
    #[error("failed to spawn the HTTP server thread: {0}")]
    ThreadSpawn(io::Error),
    #[error("HTTP server thread panicked: {0}")]
    ThreadPanic(String),
}

#[derive(Debug)]
struct SnapshotState {
    server_id: ServerId,
    state: ServerState,
    bind_address: String,
    urls: Vec<String>,
    request_count: u64,
    logs: VecDeque<HttpServerLog>,
    next_sequence: u64,
    last_error: Option<String>,
}

impl SnapshotState {
    fn snapshot(&self) -> HttpServerSnapshot {
        HttpServerSnapshot {
            server_id: self.server_id.clone(),
            state: self.state,
            bind_address: self.bind_address.clone(),
            urls: self.urls.clone(),
            request_count: self.request_count,
            logs: self.logs.iter().cloned().collect(),
            last_error: self.last_error.clone(),
        }
    }

    fn log(&mut self, level: HttpLogLevel, message: impl Into<String>) {
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.logs.push_back(HttpServerLog {
            sequence: self.next_sequence,
            level,
            message: message.into(),
        });
        while self.logs.len() > MAX_LOG_ENTRIES {
            self.logs.pop_front();
        }
    }
}

#[derive(Debug)]
struct RunningServer {
    stop_requested: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    wake_address: SocketAddr,
}

#[derive(Debug, Default)]
pub struct HttpServerManager {
    running: HashMap<ServerId, RunningServer>,
    snapshots: HashMap<ServerId, Arc<Mutex<SnapshotState>>>,
}

impl HttpServerManager {
    pub fn start(
        &mut self,
        profile: &ServerProfile,
    ) -> Result<HttpServerSnapshot, HttpServerError> {
        self.reap_finished();
        if profile.runtime != RuntimeKind::BuiltInHttp {
            return Err(HttpServerError::UnsupportedRuntime(profile.id.0.clone()));
        }
        if !profile.enabled {
            return Err(HttpServerError::Disabled(profile.id.0.clone()));
        }
        if self.running.contains_key(&profile.id) {
            return Err(HttpServerError::AlreadyRunning(profile.id.0.clone()));
        }

        let bind_ip = match profile.network_scope {
            NetworkScope::Loopback => IpAddr::V4(Ipv4Addr::LOCALHOST),
            NetworkScope::Lan => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        };
        let requested_address = SocketAddr::new(bind_ip, profile.port);
        let listener =
            TcpListener::bind(requested_address).map_err(|source| HttpServerError::Bind {
                address: requested_address.to_string(),
                source,
            })?;
        listener
            .set_nonblocking(true)
            .map_err(HttpServerError::Listener)?;
        let bound_address = listener.local_addr().map_err(HttpServerError::Listener)?;
        let wake_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), bound_address.port());
        let urls = server_urls(profile.network_scope, bound_address.port());
        let snapshot = Arc::new(Mutex::new(SnapshotState {
            server_id: profile.id.clone(),
            state: ServerState::Starting,
            bind_address: bound_address.to_string(),
            urls,
            request_count: 0,
            logs: VecDeque::new(),
            next_sequence: 0,
            last_error: None,
        }));
        {
            let mut state = lock_state(&snapshot);
            state.log(
                HttpLogLevel::Info,
                format!("Binding built-in HTTP server on {bound_address}."),
            );
        }

        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_snapshot = Arc::clone(&snapshot);
        let thread_stop = Arc::clone(&stop_requested);
        let thread_name = format!("slopity-http-{}", profile.id.0);
        let thread = thread::Builder::new()
            .name(thread_name)
            .spawn(move || serve(listener, thread_stop, thread_snapshot))
            .map_err(HttpServerError::ThreadSpawn)?;

        {
            let mut state = lock_state(&snapshot);
            state.state = ServerState::Running;
            state.log(
                HttpLogLevel::Info,
                "Built-in HTTP server is accepting requests.",
            );
        }
        self.snapshots
            .insert(profile.id.clone(), Arc::clone(&snapshot));
        self.running.insert(
            profile.id.clone(),
            RunningServer {
                stop_requested,
                thread,
                wake_address,
            },
        );
        let result = lock_state(&snapshot).snapshot();
        Ok(result)
    }

    pub fn stop(&mut self, server_id: &ServerId) -> Result<HttpServerSnapshot, HttpServerError> {
        self.reap_finished();
        let Some(running) = self.running.remove(server_id) else {
            return Err(HttpServerError::NotRunning(server_id.0.clone()));
        };
        let snapshot = self
            .snapshots
            .get(server_id)
            .cloned()
            .ok_or_else(|| HttpServerError::NotRunning(server_id.0.clone()))?;
        {
            let mut state = lock_state(&snapshot);
            state.state = ServerState::Stopping;
            state.log(HttpLogLevel::Info, "Stop requested by the user.");
        }
        running.stop_requested.store(true, Ordering::Release);
        let _ = TcpStream::connect_timeout(&running.wake_address, Duration::from_millis(150));
        if let Err(payload) = running.thread.join() {
            let reason = panic_message(payload);
            let mut state = lock_state(&snapshot);
            state.state = ServerState::Failed;
            state.last_error = Some(reason.clone());
            state.log(
                HttpLogLevel::Error,
                format!("Server thread panicked: {reason}"),
            );
            return Err(HttpServerError::ThreadPanic(reason));
        }
        let result = lock_state(&snapshot).snapshot();
        Ok(result)
    }

    pub fn snapshot(&mut self, server_id: &ServerId) -> Option<HttpServerSnapshot> {
        self.reap_finished();
        self.snapshots
            .get(server_id)
            .map(|snapshot| lock_state(snapshot).snapshot())
    }

    pub fn snapshots(&mut self) -> Vec<HttpServerSnapshot> {
        self.reap_finished();
        let mut snapshots = self
            .snapshots
            .values()
            .map(|snapshot| lock_state(snapshot).snapshot())
            .collect::<Vec<_>>();
        snapshots.sort_by(|left, right| left.server_id.0.cmp(&right.server_id.0));
        snapshots
    }

    pub fn is_active(&mut self, server_id: &ServerId) -> bool {
        self.reap_finished();
        self.running.contains_key(server_id)
    }

    pub fn active_count(&mut self) -> usize {
        self.reap_finished();
        self.running.len()
    }

    pub fn stop_all(&mut self) {
        let server_ids = self.running.keys().cloned().collect::<Vec<_>>();
        for server_id in server_ids {
            let _ = self.stop(&server_id);
        }
    }

    fn reap_finished(&mut self) {
        let finished = self
            .running
            .iter()
            .filter(|(_, running)| running.thread.is_finished())
            .map(|(server_id, _)| server_id.clone())
            .collect::<Vec<_>>();
        for server_id in finished {
            if let Some(running) = self.running.remove(&server_id) {
                let _ = running.thread.join();
            }
        }
    }
}

impl Drop for HttpServerManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

fn serve(
    listener: TcpListener,
    stop_requested: Arc<AtomicBool>,
    snapshot: Arc<Mutex<SnapshotState>>,
) {
    loop {
        if stop_requested.load(Ordering::Acquire) {
            finish_stopped(&snapshot);
            return;
        }
        match listener.accept() {
            Ok((stream, peer)) => {
                if stop_requested.load(Ordering::Acquire) {
                    finish_stopped(&snapshot);
                    return;
                }
                match handle_client(stream) {
                    Ok(path) => {
                        let mut state = lock_state(&snapshot);
                        state.request_count = state.request_count.saturating_add(1);
                        state.log(
                            HttpLogLevel::Info,
                            format!("GET {} from {peer}.", sanitize_log_value(&path)),
                        );
                    }
                    Err(error) => {
                        let mut state = lock_state(&snapshot);
                        state.log(
                            HttpLogLevel::Warning,
                            format!("Request from {peer} failed: {error}"),
                        );
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                let reason = error.to_string();
                let mut state = lock_state(&snapshot);
                state.state = ServerState::Failed;
                state.last_error = Some(reason.clone());
                state.log(HttpLogLevel::Error, format!("Listener failed: {reason}"));
                return;
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) -> io::Result<String> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut buffer = [0_u8; MAX_REQUEST_BYTES];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok("connection-closed".into());
    }
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut request_parts = request
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();
    let method = request_parts.next().unwrap_or_default();
    let path = request_parts.next().unwrap_or("/");

    let (status, content_type, body) = match (method, path) {
        ("GET", "/") => (
            "200 OK",
            "text/html; charset=utf-8",
            "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Slopity HTTP Probe</title></head><body><main><h1>Slopity is serving.</h1><p>This is the built-in development HTTP probe. It does not execute uploaded code or external runtimes.</p><p><a href=\"/health\">Health endpoint</a></p></main></body></html>",
        ),
        ("GET", "/health") => (
            "200 OK",
            "application/json; charset=utf-8",
            "{\"status\":\"ok\",\"runtime\":\"built-in-http\"}",
        ),
        ("GET", _) => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            "Not found.\n",
        ),
        _ => (
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            "Only GET is supported.\n",
        ),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(path.to_string())
}

fn finish_stopped(snapshot: &Arc<Mutex<SnapshotState>>) {
    let mut state = lock_state(snapshot);
    state.state = ServerState::Stopped;
    state.log(HttpLogLevel::Info, "Built-in HTTP server stopped cleanly.");
}

fn server_urls(scope: NetworkScope, port: u16) -> Vec<String> {
    let mut urls = vec![format!("http://127.0.0.1:{port}")];
    if scope == NetworkScope::Lan {
        if let Some(address) = discover_lan_ipv4() {
            let url = format!("http://{address}:{port}");
            if !urls.contains(&url) {
                urls.push(url);
            }
        }
    }
    urls
}

fn discover_lan_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(address) if !address.is_loopback() && !address.is_unspecified() => Some(address),
        _ => None,
    }
}

fn sanitize_log_value(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(120)
        .collect()
}

fn lock_state(snapshot: &Arc<Mutex<SnapshotState>>) -> MutexGuard<'_, SnapshotState> {
    match snapshot.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn free_port() -> u16 {
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .and_then(|listener| listener.local_addr())
            .map(|address| address.port())
            .expect("test should reserve an ephemeral port")
    }

    fn profile(port: u16) -> ServerProfile {
        ServerProfile {
            id: ServerId("http-test".into()),
            name: "HTTP test".into(),
            runtime: RuntimeKind::BuiltInHttp,
            executable: None,
            arguments: Vec::new(),
            working_directory: None,
            port,
            memory_mib: 128,
            network_scope: NetworkScope::Loopback,
            enabled: true,
        }
    }

    fn get_with_retry(port: u16) -> String {
        for _ in 0..40 {
            if let Ok(mut stream) = TcpStream::connect((Ipv4Addr::LOCALHOST, port)) {
                stream
                    .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n")
                    .expect("request should be written");
                let mut response = String::new();
                stream
                    .read_to_string(&mut response)
                    .expect("response should be readable");
                return response;
            }
            thread::sleep(Duration::from_millis(25));
        }
        panic!("server did not accept a connection");
    }

    #[test]
    fn start_serve_and_stop_release_the_port() {
        let port = free_port();
        let mut manager = HttpServerManager::default();
        let started = manager.start(&profile(port)).expect("server should start");
        assert_eq!(started.state, ServerState::Running);

        let response = get_with_retry(port);
        assert!(response.contains("200 OK"));
        assert!(response.contains("built-in-http"));

        let stopped = manager
            .stop(&ServerId("http-test".into()))
            .expect("server should stop");
        assert_eq!(stopped.state, ServerState::Stopped);
        TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .expect("graceful stop should release the port");
    }

    #[test]
    fn duplicate_start_is_rejected() {
        let port = free_port();
        let mut manager = HttpServerManager::default();
        let profile = profile(port);
        manager.start(&profile).expect("server should start");
        assert!(matches!(
            manager.start(&profile),
            Err(HttpServerError::AlreadyRunning(_))
        ));
        manager.stop_all();
    }

    #[test]
    fn occupied_port_is_reported() {
        let occupied =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("fixture port should bind");
        let port = occupied
            .local_addr()
            .expect("fixture address should be available")
            .port();
        let mut manager = HttpServerManager::default();
        assert!(matches!(
            manager.start(&profile(port)),
            Err(HttpServerError::Bind { .. })
        ));
    }

    #[test]
    fn logs_are_bounded() {
        let mut state = SnapshotState {
            server_id: ServerId("bounded".into()),
            state: ServerState::Running,
            bind_address: "127.0.0.1:8080".into(),
            urls: Vec::new(),
            request_count: 0,
            logs: VecDeque::new(),
            next_sequence: 0,
            last_error: None,
        };
        for index in 0..250 {
            state.log(HttpLogLevel::Info, format!("entry {index}"));
        }
        assert_eq!(state.logs.len(), MAX_LOG_ENTRIES);
        assert_eq!(state.logs.front().map(|entry| entry.sequence), Some(51));
        assert_eq!(state.logs.back().map(|entry| entry.sequence), Some(250));
    }
}
