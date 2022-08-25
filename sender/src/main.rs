use clap::Parser;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io;
use tokio::net::TcpStream;

static STOP: AtomicBool = AtomicBool::new(false);
const DEFAULT_ADDR: &str = "127.0.0.1:6142";

/// TCP sender
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// listner ip:port
    #[clap(short, long, value_parser)]
    addr: Option<String>,

    #[clap(short, long, value_parser, default_value_t = false)]
    load: bool,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let addr = {
        if let Some(addr) = args.addr {
            Arc::new(addr)
        } else {
            Arc::new(DEFAULT_ADDR.to_string())
        }
    };

    let mut _stream = TcpStream::connect(&*addr.clone()).await?;

    if args.load {
        while !STOP.load(Ordering::Relaxed) {
            let daddr = addr.clone();
            tokio::spawn(async move {
                let stream = TcpStream::connect(&*daddr).await;
                if let Err(e) = stream {
                    println!("Failed to connect {:?}\n", e);
                }
            });
        }
    }
    Ok(())
}
