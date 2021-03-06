#[macro_use]
extern crate log;

use simple_logger;
use std::net::SocketAddr;

use naia_server_socket::{find_my_ip_address, Config, Packet, ServerSocket, SocketEvent};

const SERVER_PORT: u16 = 14191;
const PING_MSG: &str = "ping";
const PONG_MSG: &str = "pong";

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("A logger was already initialized");

    info!("Naia Server Socket Example Started");

    let current_ip_address = find_my_ip_address().expect("can't find ip address");
    let current_socket_address = SocketAddr::new(current_ip_address, SERVER_PORT);

    let config = Config::default();

    let mut server_socket = ServerSocket::listen(current_socket_address, Some(config)).await;

    let mut sender = server_socket.get_sender();

    loop {
        match server_socket.receive().await {
            Ok(event) => {
                match event {
                    SocketEvent::Packet(packet) => {
                        let address = packet.address();
                        let message = String::from_utf8_lossy(packet.payload());
                        info!("Server recv <- {}: {}", address, message);

                        if message.eq(PING_MSG) {
                            let to_client_message: String = PONG_MSG.to_string();
                            info!("Server send -> {}: {}", address, to_client_message);
                            sender
                                .send(Packet::new(address, to_client_message.into_bytes()))
                                .await
                                .expect("send error");
                        }
                    }
                    SocketEvent::Tick => {
                        // This could be used for your non-network logic (game
                        // loop?)
                        // info!("Server Tick");
                    }
                }
            }
            Err(error) => {
                info!("Server Error: {}", error);
            }
        }
    }
}
