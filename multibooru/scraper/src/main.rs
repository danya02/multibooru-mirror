use persistence::Persistence;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    //env_logger::init();
    tracing_subscriber::fmt::init();
    //tracing_log::LogTracer::init().unwrap();
    log::info!("Starting up...");

    let mut persistence = persistence::make_persistence().await;
    persistence.init().await;

    let (tx, rx) = tokio::sync::mpsc::channel(100);
    tokio::spawn(media_assets::scraping::download_files(rx));

    tokio::spawn(danbooru_scraping::post::new_posts(
        persistence.get_sender(),
        tx,
    ));
    tokio::spawn(danbooru_scraping::tag::new_tags(persistence.get_sender()));

    log::info!("Started all scrapers, waiting for Ctrl+C...");
    tokio::signal::ctrl_c().await.unwrap();
    log::info!("Received Ctrl+C, shutting down...");
    persistence.shutdown().await;
    log::info!("Shut down.");
}
