use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicAckArguments, BasicConsumeArguments, Channel, QueueBindArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};

#[tokio::main]
async fn main() {
    println!("Started!");
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt().init();
    }

    let connection = Connection::open(&OpenConnectionArguments::new(
        &std::env::var("AMQP_SERVER").expect("AMQP_SERVER should be provided"),
        5672,
        &std::env::var("AMQP_USER").expect("AMQP_USER should be provided"),
        &std::env::var("AMQP_PASSWORD").expect("AMQP_PASSWORD should be provided"),
    ))
    .await
    .expect("Failed to connect to AMQP");
    println!("Connection established!");

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    let channel = connection
        .open_channel(None)
        .await
        .expect("Failed to open channel");

    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    println!("Channel created!");

    channel
        .queue_bind(QueueBindArguments::new("new-records", "amq.topic", ""))
        .await
        .unwrap();

    println!("Performed binding...");

    let args = BasicConsumeArguments::new("new-records", "imageboards.persistence-reader");

    let consumer = MessageConsumer {};

    channel.basic_consume(consumer, args).await.unwrap();

    println!("Connected to queue, waiting for messages...");
    tokio::signal::ctrl_c().await.unwrap();
    channel.close().await.unwrap();
    connection.close().await.unwrap();
}

struct MessageConsumer {}

#[async_trait::async_trait]
impl AsyncConsumer for MessageConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        println!("Received message with content: {content:?}");
        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();
    }
}
