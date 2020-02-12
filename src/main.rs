extern crate tokio;

use crate::tokio::prelude::*;
use std::env;
use std::net::SocketAddr;
use tokio::io::{copy, shutdown};
use tokio::net::{TcpListener, TcpStream};

fn main() {
    let listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8081".to_string());
    let listen_addr = listen_addr.parse::<SocketAddr>().unwrap();

    let server_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let server_addr = server_addr.parse::<SocketAddr>().unwrap();

    let socket = TcpListener::bind(&listen_addr).unwrap();

    let s = socket
        .incoming()
        .for_each(move |client| {
            let server = TcpStream::connect(&server_addr);

            server.and_then(|socket| {
                let (client_reader, client_writer) = client.split();
                let (server_reader, server_writer) = socket.split();

                let client_to_server = copy(client_reader, server_writer)
                    .and_then(|(n, _, w)| shutdown(w).map(move |_| n));

                let server_to_client = copy(server_reader, client_writer)
                    .and_then(|(n, _, writer)| shutdown(writer).map(move |_| n));

                let msg = client_to_server
                    .join(server_to_client)
                    .map(move |(from_client, from_server)| {
                        println!("From client: {}, from server: {}", from_client, from_server);
                    })
                    .map_err(|e| {
                        println!("{:?}", e);
                    });

                tokio::spawn(msg);
                Ok(())
            })
        })
        .map_err(|e| println!("{}", e));

    println!("Proxying {} -> {}", listen_addr, server_addr);

    tokio::run(s);
}
