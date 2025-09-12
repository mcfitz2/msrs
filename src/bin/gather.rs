
use tokio::runtime::Runtime;
use msrs::metalsupermarkets::scraper::gather;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        gather().await;
    });
}
