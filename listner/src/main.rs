use clap::Parser;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::Duration;
use tokio::io::AsyncWriteExt;

static STOP: AtomicBool = AtomicBool::new(false);
#[cfg(not(target_arch = "x86_64"))]
const DEFAULT_ADDR: &str = "127.0.0.1:6142";
#[cfg(target_arch = "x86_64")]
const DEFAULT_ADDR: &str = "0.0.0.0:6181";

/// TCP listner
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// listner ip:port
    #[clap(short, long, value_parser)]
    addr: Option<String>,
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
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
            tokio::time::sleep(Duration::from_millis(1000)).await;
            let conn = counter.load(Ordering::Relaxed);

            if conn > 0 {
                println!("{conn} connections in last epoch\n");
                counter.store(0, Ordering::SeqCst);
            }
        }
    });

    while !STOP.load(Ordering::Relaxed) {
        let (mut socket, _) = listner.accept().await?;

        let load = inner_counter.fetch_add(1, Ordering::Relaxed);

        if load < 8 {
            //let printer = inner_counter.clone();
            tokio::spawn(async move {
                    socket.write_all(b"Hello World!\n").await?;

                   println!(
                       "connection from {:?} - {}",
                   socket, load,
                   //printer.load(Ordering::Relaxed)
                   );
                    Ok::<_, tokio::io::Error>(())
            });
            //let the reply run
            tokio::task::yield_now().await;
        }
    }
    Ok(())
}
