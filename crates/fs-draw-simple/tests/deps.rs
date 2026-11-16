//! The frontend depends on the client and on fs-components only, never on the core or the server (ADR 0094 draft, item 6;
//! engine and frontend are separate, ADR 0021).
//!
//! Tests: reads the lock of the workspace and walks the dependencies of `fs-draw-simple` (what `cargo tree -p
//! fs-draw-simple -e normal` shows), without starting cargo.

use std::collections::{BTreeMap, BTreeSet};

/// `name -> its dependency names` from a `Cargo.lock`.
fn graph(lock: &str) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    let (mut name, mut deps, mut in_deps) = (String::new(), Vec::new(), false);
    for line in lock.lines().chain(std::iter::once("[[package]]")) {
        if line == "[[package]]" {
            if !name.is_empty() {
                out.entry(std::mem::take(&mut name))
                    .or_insert_with(Vec::new)
                    .append(&mut deps);
            }
            in_deps = false;
        } else if let Some(v) = line.strip_prefix("name = ") {
            v.trim_matches('"').clone_into(&mut name);
        } else if line.starts_with("dependencies = [") {
            in_deps = true;
        } else if in_deps && line == "]" {
            in_deps = false;
        } else if in_deps {
            let dep = line.trim().trim_matches(',').trim_matches('"');
            deps.push(dep.split(' ').next().unwrap_or(dep).to_owned());
        }
    }
    out
}

fn closure(graph: &BTreeMap<String, Vec<String>>, root: &str) -> BTreeSet<String> {
    let (mut seen, mut todo) = (BTreeSet::new(), vec![root.to_owned()]);
    while let Some(n) = todo.pop() {
        if seen.insert(n.clone()) {
            todo.extend(graph.get(&n).cloned().unwrap_or_default());
        }
    }
    seen
}

#[test]
fn the_frontend_never_reaches_the_core_or_the_server() {
    let lock =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock")).unwrap();
    let all = closure(&graph(&lock), "fs-draw-simple");
    assert!(
        all.contains("fs-draw-client"),
        "it speaks through the client"
    );
    assert!(all.contains("fs-components-app"));
    for forbidden in [
        "fs-draw-core",
        "fs-draw-server",
        "fs-draw-bin",
        "fs-objects",
    ] {
        assert!(
            !all.contains(forbidden),
            "{forbidden} must not be below the frontend"
        );
    }
}

#[test]
fn the_frontend_names_only_the_client_and_fs_components_as_direct_dependencies() {
    let manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let section = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap();
    for line in section
        .lines()
        .filter(|l| l.contains('=') && !l.trim_start().starts_with('#'))
    {
        let name = line.split('=').next().unwrap().trim();
        let name = name.trim_end_matches(".workspace").trim();
        assert!(
            name == "fs-draw-client" || name.starts_with("fs-components"),
            "unexpected dependency {name}"
        );
    }
}
