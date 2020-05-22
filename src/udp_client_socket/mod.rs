
extern crate log;

use std::thread;
use std::net::SocketAddr;
use std::time;

use crossbeam_channel::{Sender as ChannelSender, Receiver as ChannelReceiver};
use laminar::{ Packet as LaminarPacket, Socket as LaminarSocket, SocketEvent as LaminarEvent, Config as LaminarConfig };

use super::socket_event::SocketEvent;
use super::message_sender::MessageSender;
use crate::error::GaiaClientSocketError;
use gaia_socket_shared::{find_my_ip_address, find_available_port, SERVER_HANDSHAKE_MESSAGE, CLIENT_HANDSHAKE_MESSAGE, Config};

pub struct UdpClientSocket {
    address: SocketAddr,
    sender: ChannelSender<LaminarPacket>,
    receiver: ChannelReceiver<LaminarEvent>
}

impl UdpClientSocket {
    pub fn connect(server_address: &str, config: Option<Config>) -> UdpClientSocket {

        let mut config = LaminarConfig::default();
        config.heartbeat_interval = Option::Some(time::Duration::from_millis(500));

        let client_ip_address = find_my_ip_address::get();
        let free_socket = find_available_port::get(&client_ip_address).expect("no available ports");
        let client_socket_address = client_ip_address + ":" + free_socket.to_string().as_str();

        let mut client_socket = LaminarSocket::bind_with_config(client_socket_address, config).unwrap();

        let (sender, receiver): (ChannelSender<LaminarPacket>, ChannelReceiver<LaminarEvent>) = (client_socket.get_packet_sender(), client_socket.get_event_receiver());

        let server_socket_address: SocketAddr = server_address.parse().unwrap();

        //Send initial server handshake
        let handshake_message: String = CLIENT_HANDSHAKE_MESSAGE.to_string();
        sender.send(LaminarPacket::reliable_unordered(
            server_socket_address,
            handshake_message.clone().into_bytes(),
        ))
            .expect("failure sending client handshake");

        let _thread = thread::spawn(move || client_socket.start_polling());

        UdpClientSocket {
            address: server_socket_address,
            sender,
            receiver,
        }
    }

    pub fn receive(&mut self) -> Result<SocketEvent, GaiaClientSocketError> {
        match self.receiver.recv() {
            Ok(event) => {
                match event {
                    LaminarEvent::Connect(_) => {
                        // SHOULD NOT EVER GET HERE!, get a SERVER_HANDSHAKE_MESSAGE instead!
                        return Err(GaiaClientSocketError::Message("Client Socket has received a packet from an unknown host!".to_string()));
                    }
                    LaminarEvent::Packet(packet) => {
                        if packet.addr() == self.address {
                            let msg = String::from_utf8_lossy(packet.payload());

                            if msg.eq(SERVER_HANDSHAKE_MESSAGE) {
                                return Ok(SocketEvent::Connection);
                            }
                            else {
                                return Ok(SocketEvent::Message(msg.to_string()));
                            }
                        } else {
                            return Err(GaiaClientSocketError::Message("Unknown sender.".to_string()));
                        }
                    }
                    LaminarEvent::Timeout(_) => {

                        return Ok(SocketEvent::Disconnection);
                    }
                }
            }
            Err(error) => {
                return Err(GaiaClientSocketError::Wrapped(Box::new(error)));
            }
        }
    }

    pub fn get_sender(&mut self) -> MessageSender {
        return MessageSender::new(self.address, self.sender.clone());
    }

    pub fn server_address(&self) -> SocketAddr {
        return self.address;
    }
}