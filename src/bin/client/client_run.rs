extern crate core;

use std::net::Ipv4Addr;

use anyhow::Context;
use enet::*;
use rmp_serde::{from_slice, to_vec};
use uuid::Uuid;
use commonlib::{DataPacket, PacketType, SpawnPacket};


pub fn main() -> anyhow::Result<()> {
    let enet = Enet::new().context("could not initialize ENet")?;
    let local_addr = Address::new(Ipv4Addr::LOCALHOST, 9003);
    let mut host = enet
        .create_host::<Uuid>(
            None,
            10,
            ChannelLimit::Maximum,
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
        )
        .context("could not create host")?;

    host.connect(&Address::new(Ipv4Addr::LOCALHOST, 9001), 10, 0)
        .context("connect failed")?;

    let mut peer = loop {
        let e = host.service(1000).context("service failed")?;

        let e = match e {
            Some(ev) => ev,
            _ => continue,
        };

        println!("[client_run] event: {:#?}", e);

        match e {
            Event::Connect(_) => {
            }
            Event::Disconnect(ref p, r) => {
                println!("connection NOT successful, peer: {:?}, reason: {}", p, r);
                std::process::exit(0);
            }
            Event::Receive { ref sender, ref packet, channel_id } => {
                let mut p = sender.clone();
                p.set_data(Some(from_slice(packet.data()).unwrap()));
                break p;
            }
        };
    };



    // send a "hello"-like packet
    // peer.send_packet(
    //     Packet::new(b"harro", PacketMode::ReliableSequenced).unwrap(),
    //     1,
    // )
    //     .context("sending packet failed")?;
    let id_of_obj = Uuid::new_v4();
    let bytes = to_vec(&PacketType::Spawn(SpawnPacket{
        id: id_of_obj,
        owner: /*Uuid::from_u128(69420)*/peer.data().unwrap().clone(),
        master: peer.data().unwrap().clone(),
    })).unwrap();

    peer.send_packet(
        Packet::new(bytes.as_slice(), PacketMode::ReliableSequenced).unwrap(),
        1,
    ).context("sending packet failed")?;

    let bytes = to_vec(&PacketType::SendData(DataPacket{
        id: id_of_obj,
        data: vec![1, 2, 3, 69, 42, 0]
    })).unwrap();
    peer.send_packet(
        Packet::new(bytes.as_slice(), PacketMode::ReliableSequenced).unwrap(),
        1,
    ).context("sending packet failed")?;
    // disconnect after all outgoing packets have been sent.
    //peer.disconnect_later(5);

    loop {
        let e = host.service(1000).context("service failed")?;
        match e {
            None => {}
            Some(event) => {
                match event {
                    Event::Connect(_) => {}
                    Event::Disconnect(_, _) => {}
                    Event::Receive { ref packet, .. } => {
                        let packet_obj: PacketType = from_slice(packet.data()).unwrap();
                        println!("received packet: {:?}", packet_obj);
                    }
                }
            }
        }
    }
    Ok(())
}