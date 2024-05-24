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
const PAYLOAD_SIZE: usize = 32;
// static PAYLOAD_SIZE: usize = 256 * 1024;

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "testnode", "")?;

    // let qos = QosProfile::default().reliable().transient_local();
    let mut qos = QosProfile::default();
    qos.depth = 1;

    let publisher = node.create_publisher::<Message>("pong", qos.clone())?;
    let mut subscriber = node.subscribe::<Message>("ping", qos)?;

    let is_running = Arc::new(AtomicBool::new(true));

    let handle = tokio::task::spawn_blocking({
        let is_running = is_running.clone();
        move || {
            while is_running.load(SeqCst) {
                node.spin_once(std::time::Duration::from_millis(100));
            }
        }
    });

    let counter = Arc::new(AtomicUsize::new(0));
    tokio::task::spawn({
        let is_running = is_running.clone();
        let counter = counter.clone();
        async move {
            while is_running.load(SeqCst) {
                let start = chrono::offset::Utc::now().timestamp_nanos_opt().unwrap();
                let payload = &mut [1u8; PAYLOAD_SIZE];
                payload[..8].clone_from_slice(&start.to_le_bytes());

                let msg = Message {
                    layout: MultiArrayLayout::default(),
                    data: payload.to_vec(),
                };

                publisher.publish(&msg)?;
                counter.fetch_add(1, SeqCst);
            }
            anyhow::Ok(())
        }
    });


    println!("[ping] Started measuring for {TIMEOUT_SECS}(s)");
    let mut samples = Vec::with_capacity((TIMEOUT_SECS as f64 / 0.000002) as usize);
    let timer = std::time::Instant::now();
    while let Some(msg) = subscriber.next().await {
        let payload = msg.data;
        let start = i64::from_le_bytes(payload[..8].try_into().unwrap());
        let end = chrono::offset::Utc::now().timestamp_nanos_opt().unwrap();
        let elpased = (end - start) / 1000;
        samples.push(elpased);
        if timer.elapsed() > std::time::Duration::from_secs(TIMEOUT_SECS) {
            break;
        }
    }

    is_running.swap(false, SeqCst);
    samples.sort();
    let sum: i64 = samples.iter().sum();
    println!(
        "[ping] RTT(us) p05: {}, p50: {}, p95: {}, avg: {:.02}",
        samples[(samples.len() as f64 * 0.05) as usize],
        samples[(samples.len() as f64 * 0.50) as usize],
        samples[(samples.len() as f64 * 0.95) as usize],
        sum as f64 / samples.len() as f64,
    );

    let sent = counter.load(SeqCst);
    let recv = samples.len();
    let rate = recv as f64 / sent as f64 * 100.0;
    println!("[ping] Recv rate {recv}/{sent} = {rate:.02}% messages");

    println!("[ping] Done");
    handle.await?;

    Ok(())
}
