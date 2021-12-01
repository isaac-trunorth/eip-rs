// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

mod path;
mod service;
mod symbol;
mod tag_value;
pub mod template;

use super::*;
use futures_util::future::BoxFuture;
pub use path::{PathError, PathParser};
use rseip_cip::Status;
pub use rseip_eip::{EipContext, EipDiscovery};
pub use service::*;
use std::net::SocketAddrV4;
pub use symbol::{GetInstanceAttributeList, SymbolInstance};
pub use tag_value::TagValue;
pub use template::TemplateService;
use tokio::net::TcpStream;

pub const CLASS_SYMBOL: u16 = 0x6B;
pub const CLASS_TEMPLATE: u16 = 0x6C;

pub const SERVICE_READ_TAG: u8 = 0x4C;
pub const SERVICE_WRITE_TAG: u8 = 0x4D;
pub const SERVICE_READ_TAG_FRAGMENTED: u8 = 0x52;
pub const SERVICE_WRITE_TAG_FRAGMENTED: u8 = 0x53;
pub const SERVICE_READ_MODIFY_WRITE_TAG: u8 = 0x4E;
pub const SERVICE_TEMPLATE_READ: u8 = 0x4C;

pub const REPLY_MASK: u8 = 0x80;

/// AB EIP Client
pub type AbEipClient = Client<AbEipDriver>;

/// AB EIP Connection
pub type AbEipConnection = Connection<AbEipDriver>;

/// AB EIP driver
pub struct AbEipDriver;

impl Driver for AbEipDriver {
    type Endpoint = SocketAddrV4;
    type Service = EipContext<TcpStream>;

    #[inline]
    fn build_service(addr: &Self::Endpoint) -> BoxFuture<Result<Self::Service>> {
        EipDriver::build_service(addr)
    }
}

/// has more data
pub trait HasMore {
    /// true: has more data to retrieve
    fn has_more(&self) -> bool;
}

impl HasMore for Status {
    /// true: has more data to retrieve
    #[inline]
    fn has_more(&self) -> bool {
        self.general == 0x06
    }
}

impl<D> HasMore for MessageReply<D> {
    #[inline]
    fn has_more(&self) -> bool {
        self.status.has_more()
    }
}
