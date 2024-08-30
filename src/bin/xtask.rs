use std::env::args;
use std::process::Command;

use anyhow::{anyhow, ensure, Result};

fn main() -> Result<()> {
    match args().nth(1).as_deref() {
        None => default(),
        Some("harvester") => harvester(),
        Some("app") => app(),
        Some(name) => Err(anyhow!("Unknown task {}", name)),
    }
}

fn default() -> Result<()> {
    let status = Command::new("cargo").arg("fmt").status()?;

    ensure!(status.success(), "Rustfmt failed with status {:?}", status);

    let status = Command::new("cargo")
        .args(["clippy", "--all-targets"])
        .status()?;

    ensure!(status.success(), "Clippy failed with status {:?}", status);

    let status = Command::new("cargo").arg("test").status()?;

    ensure!(status.success(), "Tests failed with status {:?}", status);

    Ok(())
}

fn harvester() -> Result<()> {
    let status = Command::new("cargo")
        .args(["run", "--bin", "harvester"])
        .envs([("DATA_PATH", "data"), ("RUST_LOG", "info,harvester=debug")])
        .status()?;

    ensure!(
        status.success(),
        "Harvester failed with status {:?}",
        status
    );

    Ok(())
}

fn app() -> Result<()> {
    let status = Command::new("cargo")
        .args(["run", "--bin", "app"])
        .envs([
            ("DATA_PATH", "data"),
            ("BIND_ADDR", "127.0.0.1:8081"),
            ("REQUEST_LIMIT", "32"),
            ("RUST_LOG", "info,server=debug"),
        ])
        .status()?;

    ensure!(status.success(), "Server failed with status {:?}", status);

    Ok(())
}
