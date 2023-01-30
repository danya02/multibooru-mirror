use persistence::Persistence;

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting up...");

    let mut persistence = persistence::make_persistence();
    persistence.init();

    tokio::spawn(danbooru_scraping::post::new_posts(persistence.get_sender()));
    tokio::spawn(danbooru_scraping::tag::new_tags(persistence.get_sender()));

    log::info!("Started all scrapers, waiting for Ctrl+C...");
    tokio::signal::ctrl_c().await.unwrap();
    log::info!("Received Ctrl+C, shutting down...");
    persistence.shutdown().await;
    log::info!("Shut down.");
}
