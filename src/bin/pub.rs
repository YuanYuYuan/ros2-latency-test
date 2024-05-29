use futures::stream::StreamExt;
use r2r::std_msgs::msg::MultiArrayLayout;
use r2r::QosProfile;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst},
    Arc,
};
use anyhow::Result;

// type Message = r2r::std_msgs::msg::String;
type Message = r2r::std_msgs::msg::ByteMultiArray;

const TIMEOUT_SECS: u64 = 10;
const PAYLOAD_SIZE: usize = 8;
// static PAYLOAD_SIZE: usize = 256 * 1024;

#[tokio::main]
async fn main() -> Result<()> {
    println!("[pub] started");
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "testnode", "")?;

    // let qos = QosProfile::default().reliable().transient_local();
    let mut qos = QosProfile::default();
    qos.depth = 1;

    let publisher = node.create_publisher::<Message>("topic", qos.clone())?;

    let mut val = 0;
    loop {
        let payload = [val; PAYLOAD_SIZE];
        let msg = Message {
            layout: MultiArrayLayout::default(),
            data: payload.to_vec(),
        };
        publisher.publish(&msg)?;
        // std::thread::sleep(std::time::Duration::from_secs(1));
        val = (val + 1) % 255;
    }
}
