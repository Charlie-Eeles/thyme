use std::{cmp::Reverse};
use clap::Parser;

use comfy_table::{Cell, Table};
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("./"))]
    target: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = get_env_var_or_exit("DATABASE_URL");

    let args = Args::parse();

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

    println!("Running queries...");

    let mut dir = fs::read_dir(args.target).await.unwrap();

    let mut res_vec: Vec<(String, u128)> = vec![];
    while let Some(entry) = dir.next_entry().await.unwrap() {
        if !entry.file_name().to_str().unwrap_or("").ends_with(".sql") {
            continue
        }

        let query: String = fs::read_to_string(format!("{}", entry.path().display()))
            .await
            .unwrap();

        let query_start_time = Instant::now();

        match sqlx::query(&query).fetch_all(&pg_pool).await {
            Ok(_) => {
                let elapsed_time = query_start_time.elapsed();
                let query_execution_time_ms = elapsed_time.as_millis();
                res_vec.push((
                    String::from(entry.file_name().to_str().unwrap_or("")),
                    query_execution_time_ms,
                ));
            }
            Err(_) => {}
        }
    }
    //TODO: Improve ordering of this, it should check this before even connecting to the db
    if res_vec.len() == 0 {
        println!("No queries found in directory.");
        return;
    }

    res_vec.sort_by_key(|i| Reverse(i.1));

    let mut table = Table::new();
    table.set_header(vec!["Query", "Duration (sec)", "Duration (ms)"]);

    for el in res_vec {
        table.add_row(vec![
            Cell::new(el.0).fg(comfy_table::Color::Blue),
            Cell::new((el.1 as f64) / 1000.0).fg(comfy_table::Color::Green),
            Cell::new(el.1).fg(comfy_table::Color::Green),
        ]);
    }

    println!("{table}");
}
