extern crate core;


use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;

use anyhow::Context;
use enet::*;
use rmp_serde::{decode, from_slice, to_vec};
use simple_tables::{IdTable, Table};
use uuid::{uuid, Uuid};
use commonlib::{PacketType, SpawnPacket, PossessionTable, PossessionTableRow};


pub fn main() -> anyhow::Result<()> {
    let enet = Enet::new().context("could not initialize ENet")?;

    let local_addr = Address::new(Ipv4Addr::LOCALHOST, 9001);

    let mut host: Host<Uuid> = enet
        .create_host::<Uuid>(
            Some(&local_addr),
            10,
            ChannelLimit::Maximum,
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
        )
        .context("could not create host")?;

    let mut possession_table = PossessionTable::new();
    loop {
        let mut packet_to_send = None;

        match &mut host.service(1000).context("service failed")? {
            Some(Event::Connect(ref p)) => {
                let uuid = Uuid::new_v4();
                println!("new connection! {}", uuid);
                let mut p = p.clone();
                p.set_data(Some(uuid));
                p.send_packet(
                    Packet::new(to_vec(&uuid).unwrap().as_slice(), PacketMode::ReliableSequenced).unwrap(),
                    2,
                ).context("sending packet failed")?;
            },
            Some(Event::Disconnect(..)) => println!("disconnect!"),
            Some(Event::Receive {
                     sender,
                     channel_id,
                     ref packet,
                 }) => {
                let packet_obj: PacketType = from_slice(packet.data()).unwrap();
                println!("got packet on channel {}", channel_id);
                match packet_obj {
                    PacketType::Spawn(spawn_packet) => {
                        println!("spawn packet which was: {:?}", spawn_packet);
                        possession_table.push(PossessionTableRow {
                            id: spawn_packet.id,
                            owner: spawn_packet.owner,
                            master: spawn_packet.master
                        });
                        packet_to_send = Some(packet.data().to_vec());
                    }
                    PacketType::Delete(delete_packet) => {
                        possession_table.rm_row(delete_packet.id).unwrap();
                    }
                    PacketType::RequestOwnership(_) => {}
                    PacketType::SendData(data_packet) => {
                        let row = possession_table.get_row(data_packet.id).unwrap();
                        if row.owner == *sender.data().unwrap() {
                            println!("gonna send this data");
                            packet_to_send = Some(packet.data().to_vec());
                        } else {
                            println!("not owner so won't send data");
                        }
                    }
                }
            },
            _ => (),
        }
        match packet_to_send {
            None => {}
            Some(packet) => {
                let mut i = 0;
                host.peers().for_each(|mut peer| {
                    match peer.state() {
                        PeerState::Connected => {
                            peer.send_packet(Packet::new(packet.as_slice(), PacketMode::ReliableSequenced)
                                                 .unwrap(), 2).unwrap();
                            i += 1;
                        }
                        _ => {}
                    }
                });
            }
        }
    }
}
