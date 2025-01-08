use tokio::net::TcpStream;
use std::sync::Mutex;

pub struct RoundRobin {
    backends: Vec<String>,
    current: Mutex<usize>
}

impl RoundRobin {
    pub fn new(backends: Vec<String>) -> RoundRobin {
        RoundRobin {
            backends,
            current: Mutex::new(0)
        }
    }

    pub fn next_backend(&self) -> Option<String> {
        let mut current = self.current.lock().unwrap();
        if self.backends.is_empty() {
            None
        } else {
            let backend = self.backends[*current].clone();
            *current = (*current + 1) % self.backends.len();
            Some(backend)
        }
    }
}

pub async fn process(mut inbound: TcpStream, backend_addr: String) -> std::io::Result<()> {
    let mut outbound = TcpStream::connect(backend_addr).await?;
    let (mut read_in, mut write_in) = inbound.split();
    let (mut read_out, mut write_out) = outbound.split();

    tokio::try_join!(
        tokio::io::copy(&mut read_in, &mut write_out),
        tokio::io::copy(&mut read_out, &mut write_in)
    )?;

    Ok(())
}