use bb8_redis::redis;
use futures_util::StreamExt;
use tokio::task;
use tracing::info;

pub async fn start_listener(redis_host: impl AsRef<str>) -> anyhow::Result<task::JoinHandle<()>> {
    info!("starting listener ...");

    let client = redis::Client::open(redis_host.as_ref())?;
    let (mut sink, mut stream) = client.get_async_pubsub().await?.split();
    sink.subscribe("blah").await.unwrap();

    Ok(task::spawn(async move {
        // TODO: error handling
        loop {
            let msg = stream.next().await.unwrap();
            let payload: String = msg.get_payload().unwrap();
            println!("channel '{}': {}", msg.get_channel_name(), payload);
        }
    }))
}

// TODO: this doesn't work?
/*pub async fn start_listener(redis_host: impl AsRef<str>) -> anyhow::Result<task::JoinHandle<()>> {
    info!("starting listener ...");

    let client = redis::Client::open(format!("{}/?protocol=resp3", redis_host.as_ref()))?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);
    let mut con = client
        .get_multiplexed_async_connection_with_config(&config)
        .await?;
    con.subscribe("blah").await.unwrap();

    Ok(task::spawn(async move {
        loop {
            if let Some(msg) = rx.recv().await {
                println!("received {:?}", msg);
            } else {
                println!("empty");
            }
        }
    }))
}*/
