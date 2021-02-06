/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};

// a magic that tells you if a tls record is client hello (3 comparisons!)
fn is_hello(data: &[u8]) -> bool {
    // conent_type == handshake && handshake_type == client_hello
    data.len() > 5 && data[0] == 0x16 && data[5] == 0x01
}

// split a tls record half into fragments.
// if multiple tls records of same type are send, the server should
// identify them as 'fragmented' and reassemble them up to a single record.
//
// see [https://tools.ietf.org/html/rfc8446#section-5] for more detail.
fn fragmentate(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // a header of an TLSPlaintext is 5 bytes length. its content are
    // content type(1 byte), protocol version(2 bytes), fragment length(2 bytes)
    assert!(data.len() > 5);

    // split the payload into half
    let (left, right) = data.split_at(5 + (data.len() - 5) / 2);
    
    // we'll keep record type and protocol version as same as original,
    // but the payload length will be changed to the chunk's size.

    // left contains header; payload length is len - 5
    let size_bytes = ((left.len() - 5) as u16).to_be_bytes();
    let mut first = left.to_vec();
    first[3] = size_bytes[0];
    first[4] = size_bytes[1];

    let size_bytes = (right.len() as u16).to_be_bytes();
    let mut second = vec![
        data[0],
        data[1], data[2], 
        size_bytes[0], size_bytes[1]
    ];

    second.extend_from_slice(right);

    (first, second)
}

// first, second, send_first
type Fragment = (Vec<u8>, Vec<u8>, bool);

enum DetourState {
    Normal, // not sending a fragment; passthrough
    Sending(Fragment, usize), // currently sending a fragment
}

// a thin wrapper to bypass DPI(deep packet inspectation)
pub struct Detour<T: AsyncWrite> {
    sock: T,
    state: DetourState,
}

impl<T: AsyncWrite> Detour<T> {
    pub fn new(sock: T) -> Self {
        Self {
            sock,
            state: DetourState::Normal,
        }
    }

    // consume a pin to self into a pin to sock
    // we know it's safe since self.sock never moves
    fn sock(self: Pin<&mut Self>) -> Pin<&mut T> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().sock) }
    }
}

// to access internal socket
impl<T: AsyncWrite> Deref for Detour<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.sock
    }
}

// i'm in doubt if i should implement this... things would be
// easily broken if it's interrupted between sending fragments
impl<T: AsyncWrite> DerefMut for Detour<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sock
    }
}

// handy boilerplate for generics which require both read and write
impl<T: AsyncRead + AsyncWrite> AsyncRead for Detour<T> {
    fn poll_read(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>
    ) -> Poll<tokio::io::Result<()>> {
        self.sock().poll_read(cx, buf)
    }
}

impl<T: AsyncWrite> AsyncWrite for Detour<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8]
    ) -> Poll<tokio::io::Result<usize>> {
        // passthrough if the message isn't client hello (need not be fragmented)
        if !is_hello(buf) {
            return self.sock().poll_write(cx, buf);
        }

        // consume the pin out; we must not move self and its member from now on
        let ref_self = unsafe { self.get_unchecked_mut() };

        match &mut ref_self.state {
            // this call is the first time to be polled to send this buf
            DetourState::Normal => {
                // save fragments into buffer and set the state to Sending
                // the fragments will be send in the next poll
                let (first, second) = fragmentate(buf);

                ref_self.state = DetourState::Sending(
                    (first, second, true), 0
                );

                Poll::Pending
            },
            DetourState::Sending((first, second, send_first), size) => {
                // both ref_self and ref_self.sock won't move so it's safe to pin
                let sock = unsafe { Pin::new_unchecked(&mut ref_self.sock) };

                let to_send = if *send_first {
                    first
                } else {
                    second
                };

                match sock.poll_write(cx, to_send) {
                    Poll::Pending => { 
                        Poll::Pending
                    },
                    Poll::Ready(Ok(n)) => {
                        *size += n;

                        if *send_first {
                            // there are fragments left to be send
                            *send_first = false;
                            Poll::Pending
                        } else {
                            // all fragments are send; go back to Normal
                            let res = *size;
                            ref_self.state = DetourState::Normal;
                            Poll::Ready(Ok(res))
                        }
                    },
                    Poll::Ready(Err(e)) => {
                        println!("fragmented successfully");
                        Poll::Ready(Err(e))
                    }
                }
            },
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        self.sock().poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        self.sock().poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use hyper::body::Body;
    use hyper::client::conn::Builder;
    use tokio::net::TcpStream;
    use tokio_native_tls::{TlsConnector, native_tls};
    use hyper::Request;

    use super::Detour;

    async fn connect(host: &str) {
        let tls = native_tls::TlsConnector::new().unwrap();
        let tls = TlsConnector::from(tls);

        let sock = TcpStream::connect((host, 443)).await.unwrap();
        let sock = tls.connect(host, Detour::new(sock)).await.unwrap();
    
        let (mut sender, connection) = Builder::new()
            .handshake::<_, Body>(sock)
            .await.unwrap();

        let host_moved = host.to_owned();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("while connecting {}, {}", host_moved, e);
            }
        });

        let req = Request::builder()
            .method("GET")
            .header("host", host)
            .header("accept", "*/*")
            .body(Body::from(""))
            .unwrap();

        let res = sender.send_request(req).await.unwrap();
        println!("{}: {}", host, res.status());
    }

    #[tokio::test]
    async fn conn_unblocked() {
        let fut1 = connect("google.com");
        let fut2 = connect("example.com");
        let fut3 = connect("naver.com");

        tokio::join!(fut1, fut2, fut3);
    }

    #[tokio::test]
    async fn conn_blocked() {
        // list of 'popular' blocked websites
        let fut1 = connect("e-hentai.org");
        let fut2 = connect("hitomi.la");
        let fut3 = connect("pornhub.com");

        tokio::join!(fut1, fut2, fut3);
    }
}
