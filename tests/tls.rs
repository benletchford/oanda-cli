//! Keep environment-sensitive TLS tests in subprocesses: changing process-wide
//! environment variables in a multithreaded test runner is unsafe.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use oanda_cli::{Config, ErrorKind, OandaClient};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

const TIMEOUT: Duration = Duration::from_secs(10);

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tls")
        .join(name)
}

fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match listener.accept() {
            Ok((socket, _)) => {
                // Accepted sockets inherit nonblocking mode on some platforms.
                socket.set_nonblocking(false).unwrap();
                socket.set_read_timeout(Some(TIMEOUT)).unwrap();
                socket.set_write_timeout(Some(TIMEOUT)).unwrap();
                return socket;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "client did not connect");
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("accept failed: {error}"),
        }
    }
}

fn read_headers(stream: &mut impl Read) -> std::io::Result<String> {
    let mut request = Vec::new();
    while !request.ends_with(b"\r\n\r\n") {
        assert!(request.len() < 8192, "oversized test request");
        let mut byte = [0];
        stream.read_exact(&mut byte)?;
        request.push(byte[0]);
    }
    Ok(String::from_utf8(request).unwrap())
}

#[derive(Clone, Copy)]
enum ProxyMode {
    None,
    Connect,
    Bypass,
}

fn run_case(
    ca_file: Option<&str>,
    ca_dir: bool,
    wrong_hostname: bool,
    proxy: ProxyMode,
    stream: bool,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let success = (ca_file == Some("ca.pem") || ca_dir) && !wrong_hostname;
    let server = thread::spawn(move || {
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![CertificateDer::from(
                    include_bytes!("fixtures/tls/localhost.der").to_vec(),
                )],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                    include_bytes!("fixtures/tls/localhost-key.der").to_vec(),
                )),
            )
            .unwrap();
        let connection = rustls::ServerConnection::new(Arc::new(config)).unwrap();
        let mut tls = rustls::StreamOwned::new(connection, accept(&listener));
        let request = read_headers(&mut tls);
        if !success {
            assert!(request.is_err(), "untrusted TLS connection was accepted");
            return;
        }
        let request = request.unwrap();
        assert!(request.starts_with("GET /v3/accounts HTTP/1.1\r\n"));
        assert!(
            request
                .to_lowercase()
                .contains("authorization: bearer test-token\r\n")
        );
        let body = r#"{"accounts":[]}"#;
        write!(tls, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        tls.flush().unwrap();
    });

    let host = if wrong_hostname {
        "127.0.0.1"
    } else {
        "localhost"
    };
    let mut child = Command::new(std::env::current_exe().unwrap());
    child.args(["--ignored", "--exact", "tls_subprocess", "--nocapture"]);
    for key in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "REQUEST_METHOD",
    ] {
        child.env_remove(key);
    }
    // Prevent native system fallback in negative cases; these test roots are
    // never installed on the machine and bundled roots cannot trust them.
    child.env("SSL_CERT_FILE", fixture(ca_file.unwrap_or("unrelated.pem")));
    if ca_dir {
        child.env_remove("SSL_CERT_FILE");
        child.env("SSL_CERT_DIR", fixture(""));
    }
    child.env(
        "OANDA_TEST_URL",
        format!("https://{host}:{}", address.port()),
    );
    child.env("OANDA_TEST_SUCCESS", if success { "1" } else { "0" });
    child.env("OANDA_TEST_STREAM", if stream { "1" } else { "0" });

    let bypass_listener = if matches!(proxy, ProxyMode::Bypass) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        child.env(
            "HTTPS_PROXY",
            format!("http://{}", listener.local_addr().unwrap()),
        );
        child.env("NO_PROXY", "localhost");
        Some(listener)
    } else {
        None
    };
    let proxy_thread = if matches!(proxy, ProxyMode::Connect) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        child.env(
            "HTTPS_PROXY",
            format!("http://{}", listener.local_addr().unwrap()),
        );
        Some(thread::spawn(move || {
            let mut downstream = accept(&listener);
            let request = read_headers(&mut downstream).unwrap();
            assert!(request.starts_with(&format!(
                "CONNECT localhost:{} HTTP/1.1\r\n",
                address.port()
            )));
            let mut upstream = TcpStream::connect(address).unwrap();
            upstream.set_read_timeout(Some(TIMEOUT)).unwrap();
            upstream.set_write_timeout(Some(TIMEOUT)).unwrap();
            downstream
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut upstream_write = upstream.try_clone().unwrap();
            let mut downstream_read = downstream.try_clone().unwrap();
            let copy = thread::spawn(move || {
                let _ = std::io::copy(&mut downstream_read, &mut upstream_write);
                let _ = upstream_write.shutdown(std::net::Shutdown::Write);
            });
            let _ = std::io::copy(&mut upstream, &mut downstream);
            let _ = downstream.shutdown(std::net::Shutdown::Write);
            copy.join().unwrap();
        }))
    } else {
        None
    };
    let output = child.output().unwrap();
    server.join().unwrap();
    if let Some(listener) = bypass_listener {
        listener.set_nonblocking(true).unwrap();
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    if let Some(proxy) = proxy_thread {
        proxy.join().unwrap();
    }
    assert!(
        output.status.success(),
        "TLS subprocess failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn custom_ca_file_is_trusted() {
    run_case(Some("ca.pem"), false, false, ProxyMode::None, false);
}

#[test]
fn custom_ca_directory_is_trusted() {
    run_case(None, true, false, ProxyMode::None, false);
}

#[test]
fn unrelated_ca_is_rejected() {
    run_case(None, false, false, ProxyMode::None, false);
}

#[test]
fn hostname_verification_is_preserved() {
    run_case(Some("ca.pem"), false, true, ProxyMode::None, false);
}

#[test]
fn https_proxy_and_custom_ca_work_together() {
    run_case(Some("ca.pem"), false, false, ProxyMode::Connect, false);
}

#[test]
fn no_proxy_bypasses_https_proxy() {
    run_case(Some("ca.pem"), false, false, ProxyMode::Bypass, false);
}

#[test]
fn streaming_uses_custom_ca() {
    run_case(Some("ca.pem"), false, false, ProxyMode::None, true);
}

#[test]
#[ignore = "helper executed in a subprocess with isolated environment"]
fn tls_subprocess() {
    let url = std::env::var("OANDA_TEST_URL").unwrap();
    let config = Config::new("test-token", "101-001-123-001")
        .with_timeouts(Some(3), Some(3))
        .unwrap();
    let client = OandaClient::with_base_urls(&config, &url, &url).unwrap();
    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        if std::env::var("OANDA_TEST_STREAM").unwrap() == "1" {
            let response = client.stream_response("/v3/accounts", &[]).await?;
            Ok(response.json::<serde_json::Value>().await.unwrap())
        } else {
            client.accounts().list().await
        }
    });
    if std::env::var("OANDA_TEST_SUCCESS").unwrap() == "1" {
        assert_eq!(result.unwrap(), serde_json::json!({"accounts": []}));
    } else {
        let error = result.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Network);
        assert_eq!(error.exit_code(), 4);
        let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(&error);
        let mut certificate_error = false;
        while let Some(error) = cause {
            if matches!(
                error.downcast_ref::<rustls::Error>(),
                Some(rustls::Error::InvalidCertificate(_))
            ) {
                certificate_error = true;
                break;
            }
            cause = if let Some(io) = error.downcast_ref::<std::io::Error>() {
                io.get_ref()
                    .map(|inner| inner as &(dyn std::error::Error + 'static))
            } else {
                error.source()
            };
        }
        assert!(
            certificate_error,
            "expected a TLS certificate verification failure: {error:?}"
        );
    }
}
