use quinn::{Endpoint, ServerConfig};
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

    let key = "key.pem";
    let filek = File::open(Path::new(key)).expect(format!("cannot open {}", key).as_str());
    let mut brk = BufReader::new(filek);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut brk).unwrap();

    let certificate = rustls::Certificate(cetrs[0].clone());
    let private_key = rustls::PrivateKey(keys[0].clone());

    let cert_chain = vec![certificate];

    let server_config = ServerConfig::with_single_cert(cert_chain, private_key)?;

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1234);
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    let mut buf = [0u8;1024];
    while let Some(income_conn) = endpoint.accept().await {
        match income_conn.await {
            Ok(new_conn) => {
                tokio::spawn(async move {
                    match new_conn.accept_bi().await {
                        Ok((mut wstream, mut rstream)) => {
                            loop {
                                let _len = rstream.read(&mut buf).await.unwrap();
                                if let Some(_len) = _len {
                                    let recv = String::from_utf8_lossy(&buf[.._len]);
                                    let recv = format!("Recv: {}", recv);
                                    eprintln!("{}", recv);
                                    wstream.write_all(recv.as_bytes()).await.unwrap();
                                } else {
                                    break;
                                }
                            }
                            
                        }
                        Err(e) => {
                            eprintln!("Ex2 {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Ex3 {}", e);
            }
        }
    }
    endpoint.wait_idle().await;
    eprintln!("END");
    Ok(())
}
