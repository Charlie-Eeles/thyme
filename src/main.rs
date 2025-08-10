use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::{fs, time::Instant};

fn get_env_var_or_exit(name: &str) -> String {
    match std::env::var(name) {
        Ok(val) => val,
        Err(_) => {
            println!("Required variable not set in environment: {name}");
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = get_env_var_or_exit("DATABASE_URL");

    // TODO: Clean up these pool options
    let pg_pool = match PgPoolOptions::new()
        .max_connections(100)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("Successfully connected to the database.");
            pool
        }
        Err(err) => {
            println!("An error occurred connecting to the database: {err}");
            std::process::exit(1);
        }
    };

     // TODO: Replace this hardcoded example, and test every query in a particular directory
    let query: String = fs::read_to_string("./thyme_queries/example.sql") 
        .await
        .unwrap();

    let mut query_execution_time_ms: u128 = 0;
    let mut query_execution_time_sec = 0.0;
    let query_start_time = Instant::now();

    match sqlx::query(&query).fetch_all(&pg_pool).await {
        Ok(_) => {
            let elapsed_time = query_start_time.elapsed();
            query_execution_time_ms = elapsed_time.as_millis();
            query_execution_time_sec = (elapsed_time.as_secs_f64() * 100.0).round() / 100.0;
        }
        Err(_) => {}
    }
    println!("example.sql: {query_execution_time_ms}ms | {query_execution_time_sec} secs");
}
