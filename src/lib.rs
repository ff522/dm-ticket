pub mod client;
pub mod clients;
pub mod errors;
pub mod models;
pub mod server;
pub mod ticket;

use rand::Rng;

fn rand_i64(value: i64) -> u64 {
    let min_value = value / 5 * 4;
    let max_value = value + value / 10;
    let mut rng = rand::thread_rng();
    rng.gen_range(min_value..max_value) as u64
}
