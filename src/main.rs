use bongo::{configuration::{Settings, get_local_configuration}, model::{AvailableRooms}};
use env_logger::Env;
use sqlx::PgPool;

use std::net::TcpListener;
use bongo::startup::run;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config:Settings = get_local_configuration().expect("Failed to read configuration file");

    let redis_connection = redis::Client::open(config.redis.redis_url)
        .expect("Failed to open redis, invalid ip")
        .get_async_connection()
        .await
        .expect("Failed to connect to redis");

    let connection_pool= PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let available_rooms = sqlx::query_as!(AvailableRooms,r#"SELECT * from availablerooms WHERE is_open = true"#)
        .fetch_all(&connection_pool)
        .await
        .expect("Failed to query available rooms");
    let address:String = format!("127.0.0.1:{}", config.app_port);
    let listener:TcpListener = TcpListener::bind(address).expect("Failed to bind random port");
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    run(listener, connection_pool,available_rooms,redis_connection)?.await
}

