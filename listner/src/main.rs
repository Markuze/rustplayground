use clap::Parser;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io;
use tokio::net::TcpListener;
use tokio::time::Duration;

static STOP: AtomicBool = AtomicBool::new(false);
const DEFAULT_ADDR: &str = "127.0.0.1:6142";

/// TCP listner
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// listner ip:port
    #[clap(short, long, value_parser)]
    addr: Option<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let listner = {
        if let Some(addr) = args.addr {
            println!("listening on {addr}");
            TcpListener::bind(addr).await?
        } else {
            println!("listening on: {} (default)", DEFAULT_ADDR);
            TcpListener::bind(DEFAULT_ADDR).await?
        }
    };

    let inner_counter = Arc::new(AtomicU64::new(0));
    let counter = inner_counter.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let conn = counter.load(Ordering::Relaxed);

            if conn > 1 {
                println!("{conn} connections in last epoch\n");
                counter.store(0, Ordering::SeqCst);
            }
        }
    });

    while !STOP.load(Ordering::Relaxed) {
        let (mut socket, _) = listner.accept().await?;
        let printer = inner_counter.clone();

        printer.fetch_add(1, Ordering::Relaxed);
        //tokio::spawn(async move {
        println!(
            "connection from {:?} - {}",
            socket,
            printer.load(Ordering::Relaxed)
        );
        //});
    }
    Ok(())
}
