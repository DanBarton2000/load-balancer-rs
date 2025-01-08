use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use clap::Parser;
use load_balancer_rs::{process, HealthCheck, RoundRobin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let listener = TcpListener::bind(args.bind_address).await?;
    let healthy_servers = Arc::new(RwLock::new(HashSet::new()));

    let round_robin = Arc::new(RoundRobin::new(healthy_servers.clone()));

    let mut health_check = HealthCheck::new(HashSet::from_iter(args.servers), healthy_servers.clone());
    health_check.health_check().await;

    loop {
        let (stream, _) = listener.accept().await?;
        let round_robin = round_robin.clone();

        tokio::spawn(async move {
            if let Some(server_addr) = round_robin.next_server() {
                println!("Request sent to {server_addr}");
                if let Err(e) = process(stream, server_addr).await {
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