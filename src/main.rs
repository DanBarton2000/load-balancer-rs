use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:80").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        process(stream).await?;
    }
}

async fn process(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).await?;

    let mut backend_stream = TcpStream::connect("127.0.0.1:8080").await?;
    backend_stream.write(&buffer).await?;

    let mut buffer = Vec::new();
    backend_stream.read_to_end(&mut buffer).await?;

    stream.write(&*buffer).await?;

    Ok(())
}