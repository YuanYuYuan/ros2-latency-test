use anyhow::Result;
use clap::Parser;
use futures::stream::StreamExt;
use r2r::std_msgs::msg::MultiArrayLayout;
use r2r::QosProfile;
use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Arc,
};

// type Message = r2r::std_msgs::msg::String;
type Message = r2r::std_msgs::msg::ByteMultiArray;

#[derive(Parser, Debug)]
struct Args {
    /// Timeout in seconds
    #[arg(short, long, default_value_t = 10)]
    timeout_secs: u64,

    /// Payload in bytes
    #[arg(short, long, default_value_t = 32)]
    payload_size: usize,

    /// Warmup in milliseconds
    #[arg(short, long, default_value_t = 0)]
    warmup_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        timeout_secs,
        payload_size,
        warmup_ms,
    } = Args::parse();
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

    println!("[ping] Started measuring with timeout={timeout_secs}(s), payload={payload_size}(B), warmup={warmup_ms}(ms)");
    let timer = std::time::Instant::now();
    let mut samples = vec![];
    let warmup = std::time::Duration::from_millis(warmup_ms);
    while timer.elapsed() <= std::time::Duration::from_secs(timeout_secs) {
        let start = chrono::offset::Utc::now().timestamp_nanos_opt().unwrap();
        let payload = &mut vec![1u8; payload_size];
        payload[..8].clone_from_slice(&start.to_le_bytes());

        let msg = Message {
            layout: MultiArrayLayout::default(),
            data: payload.to_vec(),
        };
        publisher.publish(&msg)?;

        if timer.elapsed() < warmup {
            continue;
        }

        if let Some(msg) = subscriber.next().await {
            let payload = msg.data;
            let start = i64::from_le_bytes(payload[..8].try_into().unwrap());
            let end = chrono::offset::Utc::now().timestamp_nanos_opt().unwrap();
            let elpased = (end - start) / 1000;
            samples.push(elpased);
        } else {
            unreachable!();
        }
    }

    samples.sort();
    let sum: i64 = samples.iter().sum();
    println!(
        "[ping] RTT(us) p05: {}, p50: {}, p95: {}, avg: {:.02}",
        samples[(samples.len() as f64 * 0.05) as usize],
        samples[(samples.len() as f64 * 0.50) as usize],
        samples[(samples.len() as f64 * 0.95) as usize],
        sum as f64 / samples.len() as f64,
    );

    println!("[ping] Done");
    handle.await?;

    Ok(())
}
