//! The real server as a process, spoken to through the client: the handshake and the contract, the boards, the lease.
//!
//! Tests: the process is started with its own socket and data folder under a temp folder (never the person's own), and is
//! killed when the test ends, also when it fails.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use fs_draw_client::{
    CONTRACT, CommandStarter, ConnectError, ContractError, create_board, delete_board, handshake,
    hold_lease, list_boards, probe, rename_board,
};

/// A running `fs-draw-server`; killed and waited for on drop.
struct Server {
    child: Child,
    socket: PathBuf,
    dir: tempfile::TempDir,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start(extra: &[&str], poll_ms: Option<&str>) -> Server {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("run").join("draw.sock");
    std::fs::create_dir_all(socket.parent().unwrap()).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_fs-draw-server"));
    command
        .args(["--socket"])
        .arg(&socket)
        .args(["--data"])
        .arg(dir.path().join("data"))
        .args(extra)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(ms) = poll_ms {
        command.env("FS_LEASE_POLL_MS", ms);
    }
    let child = command.spawn().unwrap();
    let server = Server { child, socket, dir };
    wait_for(&server.socket);
    server
}

fn wait_for(socket: &Path) {
    let until = Instant::now() + Duration::from_secs(20);
    while Instant::now() < until {
        if probe(socket, CONTRACT).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("the server did not come up");
}

#[test]
fn the_report_names_what_works_and_what_is_not_possible_yet() {
    let server = start(&[], None);
    let info = probe(&server.socket, CONTRACT).unwrap();
    assert_eq!(info.contract, "whiteboard 0.1 (draft)");
    for id in ["board.create", "board.delete", "board.list"] {
        assert!(info.supported.iter().any(|s| s == id), "{id}");
    }
    let open = info
        .unsupported
        .iter()
        .find(|(id, _)| id == "board.open")
        .unwrap();
    assert!(
        open.1.contains("not in the basics"),
        "a reason, not silence"
    );
    assert!(!info.supported.iter().any(|s| s == "board.open"));
}

#[test]
fn a_contract_that_does_not_fit_is_refused_with_words() {
    let server = start(&[], None);
    for needed in ["whiteboard 9.0", "whiteboard 0.9", "texts 0.1"] {
        match probe(&server.socket, needed) {
            Err(ConnectError::Contract(
                ContractError::Major { .. } | ContractError::TooOld { .. },
            )) => {}
            other => panic!("{needed}: {other:?}"),
        }
    }
}

#[test]
fn boards_are_made_listed_renamed_and_deleted_and_kept_across_a_restart() {
    let mut server = start(&[], None);
    assert_eq!(list_boards(&server.socket).unwrap(), vec![]);
    let a = create_board(&server.socket, "Plan").unwrap();
    assert_eq!((a.name.as_str(), a.pages), ("Plan", 1));
    create_board(&server.socket, "Ideas").unwrap();
    assert_eq!(list_boards(&server.socket).unwrap().len(), 2);
    assert_eq!(
        rename_board(&server.socket, &a.id, "Plan 2").unwrap().name,
        "Plan 2"
    );
    assert!(
        list_boards(&server.socket)
            .unwrap()
            .iter()
            .any(|b| b.name == "Plan 2")
    );
    delete_board(&server.socket, &a.id).unwrap();
    let left = list_boards(&server.socket).unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].name, "Ideas");
    // A restart on the same data folder: the board is still there (the store is a file).
    let data = server.dir.path().join("data");
    let _ = server.child.kill();
    let _ = server.child.wait();
    let _ = std::fs::remove_file(&server.socket);
    let child = Command::new(env!("CARGO_BIN_EXE_fs-draw-server"))
        .args(["--socket"])
        .arg(&server.socket)
        .args(["--data"])
        .arg(&data)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    server.child = child;
    wait_for(&server.socket);
    assert_eq!(list_boards(&server.socket).unwrap()[0].name, "Ideas");
}

#[test]
fn a_server_of_this_version_is_used_as_it_is_by_the_handshake() {
    let server = start(&[], None);
    // The starter would fail if it were called: the server must not be restarted.
    let starter = CommandStarter::new("/nonexistent/fs-draw", ["serve"]);
    let info = handshake(&server.socket, CONTRACT, &starter).unwrap();
    assert_eq!(info.server_version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn a_started_server_ends_when_no_frontend_holds_the_lease_any_more() {
    let dir = tempfile::tempdir().unwrap();
    let lease = dir.path().join("x.lease");
    let held = hold_lease(&lease).unwrap();
    let mut server = start(&["--lease", lease.to_str().unwrap()], Some("50"));
    std::thread::sleep(Duration::from_millis(800));
    assert!(
        server.child.try_wait().unwrap().is_none(),
        "held lease: the server stays"
    );
    drop(held);
    let until = Instant::now() + Duration::from_secs(15);
    while Instant::now() < until {
        if server.child.try_wait().unwrap().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the server did not end after the lease was let go");
}

#[test]
fn bad_words_end_the_server_with_2_and_help_with_0() {
    let bin = env!("CARGO_BIN_EXE_fs-draw-server");
    let bad = Command::new(bin).arg("--bogus").output().unwrap();
    assert_eq!(bad.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("unknown argument"));
    let help = Command::new(bin).arg("--help").output().unwrap();
    assert_eq!(help.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&help.stdout).contains("--lease"));
}
