use clap::Parser;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io;
use tokio::net::TcpStream;
use tokio::time::Duration;
use num_format::{Buffer, Locale};
//use tokio::io::AsyncReadExt;
//use quanta::Clock;

static STOP: AtomicBool = AtomicBool::new(false);
#[cfg(not(target_arch = "x86_64"))]
const DEFAULT_ADDR: &str = "127.0.0.1:6142";
#[cfg(target_arch = "x86_64")]
const DEFAULT_ADDR: &str = "10.100.62.151:6180";

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

    /*
    let mut buf = vec![0;256];
    let n = stream.read(&mut buf).await?;

    if n > 0 {
        //this is fucking ugly but I'm lazy...
        println!("Got: {:?}", String::from_utf8((&buf[..n]).to_vec()).unwrap());
    } else {
        println!("Huston...");
    }
    */

    let inner_counter = Arc::new(AtomicU64::new(0));
    let inner_err_counter = Arc::new(AtomicU64::new(0));
    let inner_conn_counter = Arc::new(AtomicU64::new(0));

    let counter = inner_counter.clone();
    let con_at = inner_conn_counter.clone();
    let con_err = inner_err_counter.clone();

    if args.load {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(1000)).await;
                let conn = counter.load(Ordering::Relaxed);
                let mut conn_buffer = Buffer::default();
                let attempt = con_at.load(Ordering::Relaxed);
                let err = con_err.load(Ordering::Relaxed);

                conn_buffer.write_formatted(&conn, &Locale::en);
                println!("Approx: {attempt}/{} connections in last sec, {err} errors", conn_buffer.as_str());
                counter.fetch_sub(conn, Ordering::SeqCst);
                con_at.fetch_sub(attempt, Ordering::SeqCst);
                con_err.fetch_sub(err, Ordering::SeqCst);
            }
        });

        while !STOP.load(Ordering::Relaxed) {
            let daddr = addr.clone();
            let con_at = inner_conn_counter.clone();
            let con_err = inner_err_counter.clone();


            tokio::time::sleep(Duration::from_millis(1)).await; // Limit to 500 connections per sec
            inner_counter.fetch_add(1, Ordering::Relaxed);
            tokio::spawn(async move {
                let stream = TcpStream::connect(&*daddr).await;
                if let Err(_e) = stream {
                    con_err.fetch_add(1, Ordering::Relaxed);
                } else {
                    con_at.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    }
    Ok(())
}
