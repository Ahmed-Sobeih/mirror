//! TCP transport for the Mirror peer-to-peer network.
//!
//! `mirror-protocol` defines what bytes mean.
//! This crate is responsible for safely moving those bytes between peers.

use std::{
    fmt,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use mirror_protocol::{
    FRAME_HEADER_LEN, MAX_FRAME_PAYLOAD_LEN, ProtocolError, WireMessage, decode_message,
    encode_message,
};

/// Default timeout for establishing an outbound peer connection.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for reading one peer message.
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Default timeout for writing one peer message.
pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// One active TCP connection to another Mirror peer.
#[derive(Debug)]
pub struct PeerConnection {
    stream: TcpStream,
}

impl PeerConnection {
    /// Connect to another Mirror node.
    pub fn connect(address: SocketAddr) -> Result<Self, NetworkError> {
        let stream = TcpStream::connect_timeout(&address, DEFAULT_CONNECT_TIMEOUT)?;

        Self::from_stream(stream)
    }

    /// Wrap an already accepted TCP connection.
    pub fn from_stream(stream: TcpStream) -> Result<Self, NetworkError> {
        stream.set_nodelay(true)?;

        stream.set_read_timeout(Some(DEFAULT_READ_TIMEOUT))?;

        stream.set_write_timeout(Some(DEFAULT_WRITE_TIMEOUT))?;

        Ok(Self { stream })
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.stream.peer_addr()?)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.stream.local_addr()?)
    }

    /// Encode and send one complete Mirror wire message.
    pub fn send_message(&mut self, message: &WireMessage) -> Result<(), NetworkError> {
        let bytes = encode_message(message)?;

        self.stream.write_all(&bytes)?;
        self.stream.flush()?;

        Ok(())
    }

    /// Receive exactly one complete Mirror wire frame.
    ///
    /// We first read the fixed 12-byte header so we can inspect the
    /// declared payload size before allocating memory for the payload.
    pub fn receive_message(&mut self) -> Result<WireMessage, NetworkError> {
        let mut header = [0u8; FRAME_HEADER_LEN];

        read_exact_frame(&mut self.stream, &mut header)?;

        let payload_len = u32::from_le_bytes(
            header[8..12]
                .try_into()
                .expect("frame payload-length field is always four bytes"),
        ) as usize;

        if payload_len > MAX_FRAME_PAYLOAD_LEN {
            return Err(NetworkError::Protocol(ProtocolError::PayloadTooLarge {
                len: payload_len,
                max: MAX_FRAME_PAYLOAD_LEN,
            }));
        }

        let total_len = FRAME_HEADER_LEN
            .checked_add(payload_len)
            .ok_or(NetworkError::FrameLengthOverflow)?;

        let mut frame = Vec::with_capacity(total_len);

        frame.extend_from_slice(&header);

        if payload_len != 0 {
            let mut payload = vec![0u8; payload_len];

            read_exact_frame(&mut self.stream, &mut payload)?;

            frame.extend_from_slice(&payload);
        }

        Ok(decode_message(&frame)?)
    }
}

/// Listening socket for inbound Mirror peers.
#[derive(Debug)]
pub struct PeerListener {
    listener: TcpListener,
}

impl PeerListener {
    pub fn bind(address: SocketAddr) -> Result<Self, NetworkError> {
        let listener = TcpListener::bind(address)?;

        Ok(Self { listener })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.listener.local_addr()?)
    }

    /// Accept one inbound peer.
    pub fn accept(&self) -> Result<PeerConnection, NetworkError> {
        let (stream, _) = self.listener.accept()?;

        PeerConnection::from_stream(stream)
    }
}

fn read_exact_frame(stream: &mut TcpStream, bytes: &mut [u8]) -> Result<(), NetworkError> {
    match stream.read_exact(bytes) {
        Ok(()) => Ok(()),

        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
            Err(NetworkError::ConnectionClosed)
        }

        Err(error) => Err(NetworkError::Io(error)),
    }
}

#[derive(Debug)]
pub enum NetworkError {
    Io(std::io::Error),
    Protocol(ProtocolError),
    ConnectionClosed,
    FrameLengthOverflow,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => {
                write!(f, "network I/O error: {error}")
            }

            Self::Protocol(error) => {
                write!(f, "Mirror protocol error: {error}")
            }

            Self::ConnectionClosed => {
                write!(f, "peer closed the connection before completing a message")
            }

            Self::FrameLengthOverflow => {
                write!(f, "network frame length overflow")
            }
        }
    }
}

impl std::error::Error for NetworkError {}

impl From<std::io::Error> for NetworkError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ProtocolError> for NetworkError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        net::{IpAddr, Ipv4Addr},
        thread,
    };

    use mirror_crypto::Hash256;

    use mirror_protocol::{HelloMessage, WireMessage};

    fn localhost() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
    }

    fn hello(height: u64, tip_byte: u8) -> WireMessage {
        WireMessage::Hello(HelloMessage::new(
            1,
            Hash256::from_bytes([0x11; 32]),
            height,
            Hash256::from_bytes([tip_byte; 32]),
        ))
    }

    #[test]
    fn tcp_peer_sends_and_receives_hello() {
        let listener = PeerListener::bind(localhost()).unwrap();

        let address = listener.local_addr().unwrap();

        let server = thread::spawn(move || {
            let mut peer = listener.accept().unwrap();

            let received = peer.receive_message().unwrap();

            assert_eq!(received, hello(7, 0x22));
        });

        let mut client = PeerConnection::connect(address).unwrap();

        client.send_message(&hello(7, 0x22)).unwrap();

        server.join().unwrap();
    }

    #[test]
    fn two_peers_exchange_hello_messages() {
        let listener = PeerListener::bind(localhost()).unwrap();

        let address = listener.local_addr().unwrap();

        let server = thread::spawn(move || {
            let mut inbound = listener.accept().unwrap();

            let client_hello = inbound.receive_message().unwrap();

            assert_eq!(client_hello, hello(3, 0x33));

            inbound.send_message(&hello(9, 0x99)).unwrap();
        });

        let mut outbound = PeerConnection::connect(address).unwrap();

        outbound.send_message(&hello(3, 0x33)).unwrap();

        let server_hello = outbound.receive_message().unwrap();

        assert_eq!(server_hello, hello(9, 0x99));

        server.join().unwrap();
    }

    #[test]
    fn prematurely_closed_connection_is_rejected() {
        let listener = TcpListener::bind(localhost()).unwrap();

        let address = listener.local_addr().unwrap();

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();

            stream.write_all(b"MIRR").unwrap();

            // Connection closes before
            // the 12-byte header completes.
        });

        let stream = TcpStream::connect(address).unwrap();

        let mut peer = PeerConnection::from_stream(stream).unwrap();

        assert!(matches!(
            peer.receive_message(),
            Err(NetworkError::ConnectionClosed)
        ));

        server.join().unwrap();
    }

    #[test]
    fn oversized_frame_is_rejected_before_payload_allocation() {
        let listener = TcpListener::bind(localhost()).unwrap();

        let address = listener.local_addr().unwrap();

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();

            let mut header = [0u8; FRAME_HEADER_LEN];

            header[0..4].copy_from_slice(b"MIRR");

            header[4..6].copy_from_slice(&1u16.to_le_bytes());

            header[6..8].copy_from_slice(&1u16.to_le_bytes());

            let oversized = (MAX_FRAME_PAYLOAD_LEN as u32) + 1;

            header[8..12].copy_from_slice(&oversized.to_le_bytes());

            stream.write_all(&header).unwrap();
        });

        let stream = TcpStream::connect(address).unwrap();

        let mut peer = PeerConnection::from_stream(stream).unwrap();

        assert!(matches!(
            peer.receive_message(),
            Err(NetworkError::Protocol(
                ProtocolError::PayloadTooLarge { .. }
            ))
        ));

        server.join().unwrap();
    }
}
