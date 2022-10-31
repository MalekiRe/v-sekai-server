use enet::{Packet, PacketMode, Peer};
use simple_tables::{IdTable, Table};
use simple_tables::macros::table;
use simple_tables::macros::table_row;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;
use rmp_serde::to_vec;

#[table_row]
pub struct PossessionTableRow {
    pub id: Uuid,
    pub owner: Uuid,
    pub master: Uuid,
}

#[table(rows = PossessionTableRow)]
pub struct PossessionTable {}

impl IdTable<Uuid, PossessionTableRow> for PossessionTable {
    fn get_id_from_row(row: &PossessionTableRow) -> Uuid {
        row.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PacketType {
    Spawn(SpawnPacket),
    Delete(DeletePacket),
    RequestOwnership(RequestOwnershipPacket),
    SendData(DataPacket),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnPacket {
    pub id: Uuid,
    pub owner: Uuid,
    pub master: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeletePacket {
    pub id: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestOwnershipPacket {
    pub id: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPacket {
    pub id: Uuid,
    pub data: Vec<u8>
}
trait SendPacket {
    fn send_packet<T>(self, peer: Peer<T>, channel: u32) -> Result<()>;
}
impl SendPacket for SpawnPacket {
    fn send_packet<T>(self, mut peer: Peer<T>, channel: u8) -> Result<()> {
        let packet = PacketType::Spawn(self);
        let bytes = to_vec(&packet)?;//TODO optimize this so it doesn't do a heap allocation.
        peer.send_packet(Packet::new(bytes.as_slice(), PacketMode::ReliableSequenced)?, channel)?;
        Ok(())
    }
}
impl SendPacket for DeletePacket {
    fn send_packet<T>(self, mut peer: Peer<T>, channel: u8) -> Result<()> {
        let packet = PacketType::Delete(self);
        let bytes = to_vec(&packet)?;//TODO optimize this so it doesn't do a heap allocation.
        peer.send_packet(Packet::new(bytes.as_slice(), PacketMode::ReliableSequenced)?, channel)?;
        Ok(())
    }
}
impl SendPacket for RequestOwnershipPacket {
    fn send_packet<T>(self, mut peer: Peer<T>, channel: u8) -> Result<()> {
        let packet = PacketType::RequestOwnership(self);
        let bytes = to_vec(&packet)?;//TODO optimize this so it doesn't do a heap allocation.
        peer.send_packet(Packet::new(bytes.as_slice(), PacketMode::ReliableSequenced)?, channel)?;
        Ok(())
    }
}
impl SendPacket for DataPacket {
    fn send_packet<T>(self, mut peer: Peer<T>, channel: u8) -> Result<()> {
        let packet = PacketType::SendData(self);
        let bytes = to_vec(&packet)?;//TODO optimize this so it doesn't do a heap allocation.
        peer.send_packet(Packet::new(bytes.as_slice(), PacketMode::ReliableSequenced)?, channel)?;
        Ok(())
    }
}