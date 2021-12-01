// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use anyhow::Result;
use rseip::{
    cip::{
        epath::{EPath, PortSegment},
        service::MessageService,
    },
    client::{
        ab_eip::{TagValue, CLASS_SYMBOL},
        AbEipClient, AbService,
    },
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut client = AbEipClient::new_host_lookup("192.168.0.83")
        .await?
        .with_connection_path(PortSegment::default());
    // test_car1_x, its instance id is 0x66b9
    // see example ab-list-instances for how to fetch symbol instances
    let tag = EPath::default()
        .with_class(CLASS_SYMBOL)
        .with_instance(0x66b9);
    println!("read tag...");
    let value: TagValue = client.read_tag(tag.clone()).await?;
    println!("tag value: {:?}", value);
    client.write_tag(tag, value).await?;
    println!("write tag - done");
    client.close().await?;
    Ok(())
}
