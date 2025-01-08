use std::sync::Arc;
use tokio::net::TcpListener;
use clap::Parser;
use load_balancer_rs::{process, RoundRobin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let listener = TcpListener::bind(args.bind_address).await?;
    let round_robin = Arc::new(RoundRobin::new(args.servers));

    loop {
        let (stream, _) = listener.accept().await?;
        let round_robin = round_robin.clone();

        tokio::spawn(async move {
            if let Some(backend_addr) = round_robin.next_backend() {
                println!("Request sent to {backend_addr}");
                if let Err(e) = process(stream, backend_addr).await {
                    eprintln!("Failed to process request: {}", e);
                }
            } else {
                eprintln!("No backends available");
            }
        });
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    bind_address: String,

    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    servers: Vec<String>
}