use std::cmp::Reverse;

use comfy_table::Table;
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

    let mut dir = fs::read_dir("./thyme_queries").await.unwrap();

    let mut res_vec: Vec<(String, u128)> = vec![];
    while let Some(entry) = dir.next_entry().await.unwrap() {
        let query: String = fs::read_to_string(format!("{}", entry.path().display())) 
            .await
            .unwrap();

        let query_start_time = Instant::now();

        match sqlx::query(&query).fetch_all(&pg_pool).await {
            Ok(_) => {
                let elapsed_time = query_start_time.elapsed();
                let query_execution_time_ms = elapsed_time.as_millis();
                // query_execution_time_sec = (elapsed_time.as_millis() as f64) / 1000.0;
                res_vec.push((String::from(entry.file_name().to_str().unwrap_or("")), query_execution_time_ms));
            }
            Err(_) => {}
        }
    }
    res_vec.sort_by_key(|i| Reverse(i.1));

    let mut table = Table::new();
    table
        .set_header(vec!["Query", "Duration"]);

    for el in res_vec {
        table.add_row(vec![el.0, el.1.to_string()]);
    }

    println!("{table}");
}
