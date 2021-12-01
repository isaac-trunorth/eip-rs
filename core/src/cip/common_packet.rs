// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use core::{
    convert::TryFrom,
    ops::{Deref, DerefMut},
};
use smallvec::SmallVec;
use std::io::{self, Error, ErrorKind};

type Array = [CommonPacketItem; 4];

/// common packet format
#[derive(Default, Debug)]
pub struct CommonPacket(SmallVec<Array>);

impl CommonPacket {
    /// new object
    #[inline]
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// append an item
    #[inline]
    pub fn push(&mut self, item: CommonPacketItem) {
        self.0.push(item);
    }

    /// panic if idx is out of range
    #[inline]
    pub fn remove(&mut self, idx: usize) -> CommonPacketItem {
        self.0.remove(idx)
    }
}

impl Deref for CommonPacket {
    type Target = [CommonPacketItem];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CommonPacket {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<CommonPacketItem>> for CommonPacket {
    #[inline]
    fn from(src: Vec<CommonPacketItem>) -> Self {
        Self(SmallVec::from_vec(src))
    }
}

impl IntoIterator for CommonPacket {
    type Item = CommonPacketItem;
    type IntoIter = crate::iter::IntoIter<Array>;
    fn into_iter(self) -> Self::IntoIter {
        crate::iter::IntoIter::new(self.0)
    }
}

/// common packet format item
#[derive(Debug)]
pub struct CommonPacketItem {
    pub type_code: u16,
    pub data: Bytes,
}

impl CommonPacketItem {
    /// null address
    #[inline]
    pub fn with_null_addr() -> Self {
        Self {
            type_code: 0,
            data: Bytes::from_static(&[0x00, 0x00]),
        }
    }

    /// unconnected data item
    #[inline]
    pub fn with_unconnected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB2,
            data,
        }
    }

    /// connected data item
    #[inline]
    pub fn with_connected_data(data: Bytes) -> Self {
        Self {
            type_code: 0xB1,
            data,
        }
    }

    /// is null address
    #[inline]
    pub fn is_null_addr(&self) -> bool {
        if self.type_code != 0 {
            return false;
        }
        self.data.is_empty()
    }

    /// ensure current item matches the specified type code
    #[inline]
    pub fn ensure_type_code(&self, type_code: u16) -> io::Result<()> {
        if self.type_code != type_code {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "common packet format item: unexpected type code - {:#0x}",
                    type_code
                ),
            ));
        }
        Ok(())
    }
}

pub struct CommonPacketIter {
    buf: Bytes,
    offset: u16,
    total: u16,
}

impl CommonPacketIter {
    #[inline]
    pub fn new(mut buf: Bytes) -> io::Result<Self> {
        if buf.len() < 2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "common packet format: invalid data",
            ));
        }
        let item_count = LittleEndian::read_u16(&buf[0..2]);
        buf = buf.slice(2..);
        Ok(Self {
            buf,
            offset: 0,
            total: item_count,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.total as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }
}

impl Iterator for CommonPacketIter {
    type Item = io::Result<CommonPacketItem>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.total == 0 {
            return None;
        }
        if self.offset >= self.total {
            return None;
        }

        if self.buf.len() < 4 {
            return Some(Err(Error::new(
                ErrorKind::InvalidData,
                "common packet format: invalid data",
            )));
        }
        let type_code = LittleEndian::read_u16(&self.buf[0..2]);
        let item_length = LittleEndian::read_u16(&self.buf[2..4]) as usize;
        if self.buf.len() < 4 + item_length {
            return Some(Err(Error::new(
                ErrorKind::InvalidData,
                "common packet format: invalid data",
            )));
        }
        let _ = self.buf.split_to(4);
        let item_data = self.buf.split_to(item_length as usize);
        let item = CommonPacketItem {
            type_code,
            data: item_data,
        };
        self.offset += 1;
        Some(Ok(item))
    }
}

impl TryFrom<Bytes> for CommonPacket {
    type Error = Error;
    #[inline]
    fn try_from(mut buf: Bytes) -> Result<Self, Self::Error> {
        if buf.len() < 2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "common packet format: invalid data",
            ));
        }
        let item_count = LittleEndian::read_u16(&buf[0..2]);
        buf = buf.slice(2..);
        let mut cpf = CommonPacket::new();
        for _ in 0..item_count {
            if buf.len() < 4 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "common packet format: invalid data",
                ));
            }
            let type_code = LittleEndian::read_u16(&buf[0..2]);
            let item_length = LittleEndian::read_u16(&buf[2..4]) as usize;
            if buf.len() < 4 + item_length {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "common packet format: invalid data",
                ));
            }
            let item_data = buf.slice(4..4 + item_length);
            cpf.push(CommonPacketItem {
                type_code,
                data: item_data,
            });
            buf = buf.slice(4 + item_length..);
        }

        // should no remaining left
        if !buf.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "common packet format: invalid data",
            ));
        }
        Ok(cpf)
    }
}
