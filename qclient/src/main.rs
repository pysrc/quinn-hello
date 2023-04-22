use quinn::{ClientConfig, Endpoint};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cert = "cert.pem";
    let file = File::open(Path::new(cert)).expect(format!("cannot open {}", cert).as_str());
    let mut br = BufReader::new(file);
    let cetrs = rustls_pemfile::certs(&mut br).unwrap();

    let certificate = rustls::Certificate(cetrs[0].clone());
    let mut certs = rustls::RootCertStore::empty();
    certs.add(&certificate)?;

    let client_config = ClientConfig::with_root_certificates(certs);

    let endpoint = {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let mut endpoint = Endpoint::client(bind_addr)?;
        endpoint.set_default_client_config(client_config);
        endpoint
    };

    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1234);

    let new_conn = endpoint
        .connect(server_addr, "hello.world.example")?
        .await?;

    let (mut w, mut r) = new_conn.open_bi().await?;

    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        let _ = tokio::io::copy(&mut r, &mut stdout).await;
    });
    let mut stdin = tokio::io::stdin();
    tokio::io::copy(&mut stdin, &mut w).await?;

    w.finish().await?;

    new_conn.close(0u32.into(), b"done");

    endpoint.wait_idle().await;

    Ok(())
}
