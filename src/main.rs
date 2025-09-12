/// Returns the bundled product list as Vec<ProductInfo>
fn bundled_products() -> Vec<msrs::metalsupermarkets::models::ProductInfo<'static>> {
    let bytes = include_bytes!("./resources/products.json");
    serde_json::from_slice(bytes).expect("Failed to parse bundled products.json")
}
struct ChromedriverGuard {
    child: std::process::Child,
}

impl Drop for ChromedriverGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}


use reqwest::cookie::CookieStore;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::process;
use msrs::metalsupermarkets::api_client::{ApiClient, AddToCartParams};
use fantoccini::{ClientBuilder, cookies::Cookie};
use std::process::{Command, Stdio};
use serde_json;
use clap::Parser;

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    secure: bool,
    http_only: bool,
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Place an order from a CSV file
    Order {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long, default_value = "023001")]
        store_id: String,
    },
    /// Products scraping commands
    Products {
        #[command(subcommand)]
        subcmd: Subcommand,
    },
}

#[derive(clap::Subcommand, Debug)]
enum Subcommand {
    List,
}

#[derive(Debug, Clone)]
struct Part {
    id: String,
    sku: String,
    qualifier_a: String,
    qualifier_b: String,
    qualifier_c: String,
    length: Option<String>,
    width: Option<String>,
    quantity: usize,
}

impl PartialEq for Part {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.sku == other.sku &&
        self.qualifier_a == other.qualifier_a &&
        self.qualifier_b == other.qualifier_b &&
        self.qualifier_c == other.qualifier_c &&
        self.length == other.length &&
        self.width == other.width
    }
}
impl Eq for Part {}
impl Hash for Part {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.sku.hash(state);
        self.qualifier_a.hash(state);
        self.qualifier_b.hash(state);
        self.qualifier_c.hash(state);
        self.length.hash(state);
        self.width.hash(state);
    }
}
async fn open_cart_with_cookies(cookies: Vec<SerializableCookie>) -> Result<(), fantoccini::error::CmdError> {
    // Start WebDriver session (assumes chromedriver or geckodriver running on localhost:9515 or 4444)
    // Check if chromedriver is installed
    let chromedriver_check = Command::new("which")
        .arg("chromedriver")
        .stdout(Stdio::null())
        .status();
    match chromedriver_check {
        Ok(status) if status.success() => {},
        _ => {
            eprintln!("chromedriver is not installed or not in PATH. Please install it with 'brew install chromedriver' or from https://sites.google.com/chromium.org/driver/");
            std::process::exit(1);
        }
    }

    // Start chromedriver as a background process
    let chromedriver_child = Command::new("chromedriver")
        .arg("--port=9515")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start chromedriver");
    let _chromedriver_guard = ChromedriverGuard { child: chromedriver_child };

    // Wait a moment for chromedriver to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    let client = ClientBuilder::native().connect("http://localhost:9515").await.expect("failed to connect to webdriver");
    // Go to base domain to set cookies
    client.goto("https://www.metalsupermarkets.com/").await?;
    // Set cookies
    for c in cookies {
    let mut cookie = Cookie::new(c.name.clone(), c.value.clone());
    let domain: &'static str = Box::leak(c.domain.clone().into_boxed_str());
    let path: &'static str = Box::leak(c.path.clone().into_boxed_str());
    cookie.set_domain(domain as &str);
    cookie.set_path(path as &str);
    cookie.set_secure(c.secure);
    cookie.set_http_only(c.http_only);
    client.add_cookie(cookie).await?;
    }
    // Go to cart page
    client.goto("https://www.metalsupermarkets.com/cart/").await?;
    println!("Cart page opened in browser. Complete checkout, then press Enter here to close the browser.");
    let _ = std::io::stdin().read_line(&mut String::new());
    let _ = client.close().await;
    Ok(())
}




#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Order { input, username, password, store_id } => {
            // ...existing order logic...
            let file = File::open(&input).unwrap_or_else(|e| {
                eprintln!("Failed to open file {}: {}", &input, e);
                process::exit(1);
            });
            let mut rdr = csv::Reader::from_reader(file);
            let headers = rdr.headers().expect("Failed to read headers").clone();
            let required = ["ID", "SKU", "Qualifier A", "Qualifier B", "Qualifier C", "Length", "Width"];
            for &col in &required {
                if !headers.iter().any(|h| h == col) {
                    eprintln!("Missing required column: {}", col);
                    process::exit(1);
                }
            }
            let mut part_map: HashMap<Part, usize> = HashMap::new();
            for result in rdr.records() {
                let record = result.expect("Failed to read record");
                let get = |col: &str| record.get(headers.iter().position(|h| h == col).unwrap()).unwrap_or("");
                let part = Part {
                    id: get("ID").to_string(),
                    sku: get("SKU").to_string(),
                    qualifier_a: get("Qualifier A").to_string(),
                    qualifier_b: get("Qualifier B").to_string(),
                    qualifier_c: get("Qualifier C").to_string(),
                    length: match get("Length").trim() { "" => None, s => Some(s.to_string()) },
                    width: match get("Width").trim() { "" => None, s => Some(s.to_string()) },
                    quantity: 1,
                };
                *part_map.entry(part).or_insert(0) += 1;
            }
            let deduped: Vec<Part> = part_map.into_iter().map(|(mut part, qty)| { part.quantity = qty; part }).collect();
            println!("Logging in as {}...", username);
            let api = ApiClient::new();
            let login_res = api.login(&username, &password).await;
            match login_res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        println!("Login successful");
                    } else {
                        eprintln!("Login failed: {}", resp.status());
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Login error: {}", e);
                    process::exit(1);
                }
            }
            for part in &deduped {
                let params = AddToCartParams {
                    action: "put_addtocart",
                    store_id: &store_id,
                    store_country: "USA",
                    pro_id: &part.id,
                    pro_sku: &part.sku,
                    prowidth: part.width.as_deref(),
                    prolength: part.length.as_deref().unwrap_or(""),
                    selunits: "Inches",
                    selquantity: &part.quantity.to_string(),
                    pro_price: "0.0",
                };
                let res = api.add_to_cart(params).await;
                match res {
                    Ok(resp) => {
                        let status = resp.status();
                        match resp.text().await {
                            Ok(body) => println!("Add to cart response for {:?} (status: {}):\n{}", part, status, body),
                            Err(e) => println!("Add to cart response for {:?} (status: {}): <failed to read body: {}>", part, status, e),
                        }
                    }
                    Err(e) => eprintln!("Failed to add to cart: {:?} ({})", part, e),
                }
            }

            let jar = &api.cookie_jar;
            let url = url::Url::parse("https://www.metalsupermarkets.com/").unwrap();
            let mut cookies_vec: Vec<SerializableCookie> = Vec::new();
            if let Some(cookie_str) = jar.cookies(&url) {
                let cookie_str = cookie_str.to_str().unwrap_or("");
                for kv in cookie_str.split(';') {
                    let mut parts = kv.trim().splitn(2, '=');
                    if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                        cookies_vec.push(SerializableCookie {
                            name: name.to_string(),
                            value: value.to_string(),
                            domain: "www.metalsupermarkets.com".to_string(),
                            path: "/".to_string(),
                            secure: true,
                            http_only: false,
                        });
                    }
                }
            }

            // Launch fantoccini to open cart page with cookies
            println!("Launching browser for manual checkout...");
            if let Err(e) = open_cart_with_cookies(cookies_vec).await {
                eprintln!("Failed to launch browser for manual checkout: {}", e);
            }
        }
        Commands::Products { subcmd } => {
            match subcmd {
                Subcommand::List => {
                    let products = bundled_products();
                    if products.is_empty() {
                        println!("No products found in bundled products");
                    } else {
                        println!("Products:");
                        println!("{}", "-".repeat(60));
                        for product in &products {
                            println!("{} || SKU: {} | ID: {} | Size: {} X {}", product.description, product.sku, product.id, product.qualifier_a, product.qualifier_b);
                        }
                    }
                }
            }
        }
    }
}
