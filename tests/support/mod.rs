use bili_live::api::Api;
use bili_live::model::Credentials;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub struct Response {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Response {
    pub fn json(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}

pub struct Fixture {
    pub api: Api,
    pub origin: String,
    pub cookie_path: PathBuf,
    directory: PathBuf,
    stop: Arc<AtomicBool>,
    server: Option<JoinHandle<Vec<String>>>,
}

impl Fixture {
    pub fn new(bodies: &[&str]) -> Self {
        Self::responses(bodies.iter().map(|body| Response::json(*body)).collect())
    }

    pub fn responses(responses: Vec<Response>) -> Self {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "bili-live-test-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        let cookie_path = directory.join("cookies.json");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let api = Api::with_origins(&origin, &origin, &origin, &cookie_path).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let server = thread::spawn(move || {
            let mut requests = Vec::new();
            for response in responses {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut socket = loop {
                    if stopped.load(Ordering::Relaxed) {
                        return requests;
                    }
                    match listener.accept() {
                        Ok((socket, _)) => break socket,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "fixture request timed out");
                            thread::sleep(Duration::from_millis(2));
                        }
                        Err(error) => panic!("fixture accept failed: {error}"),
                    }
                };
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                requests.push(read_request(&mut socket));
                write!(socket, "HTTP/1.1 {} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n", response.status, response.body.len()).unwrap();
                for (name, value) in response.headers {
                    write!(socket, "{name}: {value}\r\n").unwrap();
                }
                write!(socket, "\r\n{}", response.body).unwrap();
            }
            requests
        });
        Self {
            api,
            origin,
            cookie_path,
            directory,
            stop,
            server: Some(server),
        }
    }

    pub fn save_credentials(&self) {
        std::fs::write(
            &self.cookie_path,
            serde_json::to_vec(&credentials()).unwrap(),
        )
        .unwrap();
    }

    pub fn requests(mut self) -> Vec<String> {
        self.server.take().unwrap().join().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(server) = self.server.take() {
            let _ = server.join();
        }
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

fn read_request(socket: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0; 1];
        socket.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
        assert!(
            bytes.len() <= 64 * 1024,
            "fixture request headers too large"
        );
        if bytes.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let header_end = bytes.len();
    let headers = String::from_utf8(bytes.clone()).unwrap();
    let length = headers
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    bytes.resize(header_end + length, 0);
    socket.read_exact(&mut bytes[header_end..]).unwrap();
    String::from_utf8(bytes).unwrap()
}

pub fn credentials() -> Credentials {
    Credentials {
        room_id: 123,
        sessdata: "session-safe".into(),
        csrf_token: "csrf+value".into(),
    }
}
