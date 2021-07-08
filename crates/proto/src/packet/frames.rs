use std::io::{self, ErrorKind};

use crate::cursor::BytesMut;

#[derive(Copy, Clone, Debug)]
pub enum PacketType {
    Handshake,
    Data,
}

#[derive(Copy, Clone, Debug)]
pub enum Header {
    Long {
        packet_number: u64,
        packet_type: PacketType,
        // TODO: Add version checksum.
        src_id: u64,
        dst_id: u64,
    },
    Short {
        packet_number: u64,
        packet_type: PacketType,
        dst_id: u64,
    },
}

impl Header {
    pub fn read(buf: &mut BytesMut) -> io::Result<Self> {
        let packet_number = buf.read::<u64>()?;
        let packet_type = buf.read::<u8>()?;
        let header = match packet_type {
            0x01 => {
                let src_id = buf.read::<u64>()?;
                let dst_id = buf.read::<u64>()?;

                Header::Long {
                    packet_number,
                    packet_type: PacketType::Handshake,
                    src_id,
                    dst_id,
                }
            },
            0x10 => {
                let dst_id = buf.read::<u64>()?;

                Header::Short {
                    packet_number,
                    packet_type: PacketType::Data,
                    dst_id,
                }
            },
        };

        Ok(header)
    }

    pub fn write(&self, buf: &mut BytesMut) -> io::Result<()> {
        match self {
            Header::Long {
                packet_number,
                packet_type,
                src_id,
                dst_id,
            } => {
                buf.write::<u64>(packet_number);
                buf.write::<u8>(0x01);
                buf.write::<u64>(src_id);
                buf.write::<u64>(dst_id);
            },
            Header::Short {
                packet_number,
                packet_type,
                dst_id,
            } => {
                buf.write::<u64>(packet_number);
                buf.write::<u8>(0x10);
                buf.write::<u64>(dst_id);
            },
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Frame {
    Padding {
        len: u16,
    },
    Ping,
    Ack {
        ack_sequence: u64,
        ack_mask: u64,
    },
    Data {
        channel_id: u64,
        channel_sequence: u64,
        fragment_index: u8,
        fragment_count: u8,
        len: u16,
    },
}

impl Frame {
    pub fn read(buf: &mut BytesMut) -> io::Result<Self> {
        let frame_type = buf.read::<u8>()?;
        let frame = match frame_type {
            0x00 => {
                let mut len = 1;
                while buf.peek::<u8>() == Ok(0x00) {
                    buf.read::<u8>()?;
                    len += 1;
                }

                Frame::Padding { len }
            },
            0x10 => Frame::Ping,
            0x20 => {
                let ack_sequence = buf.read::<u64>()?;
                let ack_mask = buf.read::<u64>()?;

                Frame::Ack {
                    ack_sequence,
                    ack_mask,
                }
            },
            0x31 => {
                let channel_id = buf.read::<u64>()?;
                let channel_sequence = buf.read::<u64>()?;
                let fragment_index = buf.read::<u8>()?;
                let fragment_count = buf.read::<u8>()?;
                let len = buf.read::<u16>()?;

                Frame::Data {
                    channel_id,
                    channel_sequence,
                    fragment_index,
                    fragment_count,
                    len,
                }
            },
            _ => return Err(ErrorKind::InvalidData),
        };

        Ok(frame)
    }

    pub fn write(&self, buf: &mut BytesMut) -> io::Result<()> {
        match self {
            Frame::Padding { len } => {
                buf.write_bytes(0x00, len as usize)?;
            },
            Frame::Ping => {
                buf.write::<u8>(0x10)?;
            },
            Frame::Ack {
                ack_sequence,
                ack_mask,
            } => {
                buf.write::<u8>(0x20)?;
                buf.write::<u64>(ack_sequence)?;
                buf.write::<u64>(ack_mask)?;
            },
            Frame::Data {
                channel_id,
                channel_sequence,
                fragment_index,
                fragment_count,
                len,
            } => {
                buf.write::<u8>(0x31)?;
                buf.write::<u64>(channel_id)?;
                buf.write::<u64>(channel_sequence)?;
                buf.write::<u8>(fragment_index)?;
                buf.write::<u8>(fragment_count)?;
                buf.write::<u16>(len)?;
            },
        }

        Ok(())
    }
}
