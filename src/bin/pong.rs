use futures::{future, stream::StreamExt};
use r2r::QosProfile;

// type Message = r2r::std_msgs::msg::String;
type Message = r2r::std_msgs::msg::ByteMultiArray;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "testnode", "")?;

    // let qos = QosProfile::default().reliable().transient_local();
    let mut qos = QosProfile::default();
    qos.depth = 1;

    let publisher = node.create_publisher::<Message>("ping", qos.clone())?;
    let subscriber = node.subscribe::<Message>("pong", qos)?;

    let handle = tokio::task::spawn_blocking(move || loop {
        node.spin_once(std::time::Duration::from_millis(100));
    });

    println!("[pong] Started");
    subscriber
        .for_each(|msg| {
            publisher.publish(&msg).unwrap();
            future::ready(())
        })
        .await;

    handle.await?;

    println!("[pong] Done");
    Ok(())
}
