//! The handshake with a contract that does not fit: the frontend `simple` exits with 3 and a message and opens no window.
//!
//! Tests: a real server (`fs-draw serve`, own socket and data folder under a temp folder) and the frontend as a library
//! call; the window is never reached with these words (a contract that fits would open one, so no test does that).

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use fs_draw_client::{CONTRACT, probe};
use fs_draw_simple::{EXIT_CONTRACT, run_code};

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn a_frontend_that_needs_another_contract_ends_with_3_instead_of_opening_a_window() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("draw.sock");
    let child = Command::new(env!("CARGO_BIN_EXE_fs-draw"))
        .arg("serve")
        .args(["--socket"])
        .arg(&socket)
        .args(["--data"])
        .arg(dir.path().join("data"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _server = Server(child);
    let until = Instant::now() + Duration::from_secs(20);
    while probe(&socket, CONTRACT).is_err() {
        assert!(Instant::now() < until, "the server did not come up");
        std::thread::sleep(Duration::from_millis(50));
    }
    let words: Vec<String> = [
        "--socket",
        socket.to_str().unwrap(),
        "--needs-contract",
        "whiteboard 9.0",
    ]
    .map(str::to_owned)
    .to_vec();
    assert_eq!(run_code(&words), EXIT_CONTRACT);
    assert_eq!(EXIT_CONTRACT, 3, "the number the starter reads");
}
