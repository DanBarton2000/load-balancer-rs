use std::collections::HashSet;
use tokio::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

type Servers = Arc<RwLock<HashSet<String>>>;

pub struct HealthCheck{
    servers: HashSet<String>,
    healthy_servers: Servers
}

impl HealthCheck  {
    pub fn new(servers: HashSet<String>, healthy_servers: Servers) -> HealthCheck {
        HealthCheck {
            servers,
            healthy_servers
        }
    }

    pub async fn health_check(&mut self) {
        for server_addr in &self.servers {
            let server_addr = server_addr.clone();
            let healthy_servers = self.healthy_servers.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    if let Err(_) = TcpStream::connect(&server_addr).await {
                        eprintln!("Failed to create stream for {server_addr}");
                        healthy_servers.write().unwrap().retain(|x| *x != server_addr);
                    } else {
                        healthy_servers.write().unwrap().insert(server_addr.clone());
                    }
                }
            });
        }
    }
}

pub struct RoundRobin {
    servers: Servers,
    current: Mutex<usize>
}

impl RoundRobin  {
    pub fn new(servers: Servers) -> RoundRobin {
        RoundRobin {
            servers,
            current: Mutex::new(0)
        }
    }

    pub fn next_server(&self) -> Option<String> {
        let mut current = self.current.lock().unwrap();
        if self.servers.read().unwrap().is_empty() {
            None
        } else {
            let backend = self.servers.read().unwrap().iter().nth(*current)?.clone();
            *current = (*current + 1) % self.servers.read().unwrap().len();
            Some(backend)
        }
    }
}

pub async fn process(mut inbound: TcpStream, server_addr: String) -> std::io::Result<()> {
    let mut outbound = TcpStream::connect(server_addr).await?;
    let (mut read_in, mut write_in) = inbound.split();
    let (mut read_out, mut write_out) = outbound.split();

    tokio::try_join!(
        tokio::io::copy(&mut read_in, &mut write_out),
        tokio::io::copy(&mut read_out, &mut write_in)
    )?;

    Ok(())
}