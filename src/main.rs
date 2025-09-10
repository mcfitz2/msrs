use reqwest::{header, Client};
use scraper::ElementRef;
use scraper::{Html, Selector};
use serde_json::json;
use itertools::Itertools;
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex};
use std::collections::HashMap;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
struct ProductInfo {
    sku: Option<String>,
    id: Option<String>,
    source_url: String,
    // metal: Metal,
    // shape: Shape,
    // gradde: Grade
}

#[derive(Debug, PartialEq)]
enum Metal {
    Aluminum,
    MildSteel,
    StainlessSteel,
    Bronze,
    Copper,
    AlloySteel,
    Brass,
    ToolSteel
}

#[derive(Debug, PartialEq)]
enum Shape {

}

#[derive(Debug, PartialEq)]
enum Grade {

}

#[derive(Debug, PartialEq)]
struct User {}

async fn login() -> Result<User, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert("content-type", "application/x-www-form-urlencoded".parse().unwrap());

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let res = client.post("https://www.metalsupermarkets.com/login/")
        .headers(headers)
        .body("msm_action=form_login&msm_redirect_to=%2Fmy-account%2Fmy-orders%2F&msm_email=mcfitz2%40gmail.com&msm_password=cl0ser2g0d&defaultstorekeylogin=023001")
        .send().await;
    println!("{:#?}", res);

    Ok(User {  })
}

async fn fetch_product_skus_and_ids(product_url: String) -> Result<Vec<ProductInfo>, reqwest::Error> {
    let client = Client::new();
    println!("Fetching [{}]", product_url);
    let cache = RESPONSE_CACHE.get().unwrap();
    let mut cache_guard = cache.lock().await;
    let resp = if let Some(cached) = cache_guard.get(&product_url) {
        println!("[CACHE HIT] [{}]", product_url);
        cached.clone().unwrap_or_default()
    } else {
        let text = client.get(&product_url).send().await?.text().await?;
        cache_guard.insert(product_url.clone(), Some(text.clone()));
        text
    };
    drop(cache_guard);
    let document = Html::parse_document(&resp);
    let mut products = Vec::new();
    let price_selector = scraper::Selector::parse("td[data-label='Price']").unwrap();
    let input_selector = scraper::Selector::parse("input[type='hidden']").unwrap();
    for td in document.select(&price_selector) {
        let mut pro_id = None;
        let mut pro_sku = None;
        for input in td.select(&input_selector) {
            if let Some(name) = input.value().attr("name") {
                match name {
                    "pro_id" => pro_id = input.value().attr("value").map(|v| v.to_string()),
                    "pro_sku" => pro_sku = input.value().attr("value").map(|v| v.to_string()),
                    _ => {}
                }
            }
        }
        if pro_id.is_some() || pro_sku.is_some() {
            products.push(ProductInfo {
                sku: pro_sku,
                id: pro_id,
                source_url: product_url.clone()
            });
        }
    }
    Ok(products)
}
async fn fetch_shapes(metal_url: String) -> Result<Vec<String>, reqwest::Error> {
    let client = Client::new();
    println!("Fetching [{}]", metal_url);
    let cache = RESPONSE_CACHE.get().unwrap();
    let mut cache_guard = cache.lock().await;
    let resp = if let Some(cached) = cache_guard.get(&metal_url) {
        println!("[CACHE HIT] [{}]", metal_url);
        cached.clone().unwrap_or_default()
    } else {
        let text = client.get(&metal_url).send().await?.text().await?;
        cache_guard.insert(metal_url.clone(), Some(text.clone()));
        text
    };
    drop(cache_guard);
    let document = Html::parse_document(&resp);
    let selector = Selector::parse("div > a").unwrap();
    let links: Vec<String> = document
        .select(&selector)
        .filter(filter_metal_links)
        .filter_map(|element| {
            let href = element.value().attr("href")?.to_string();
            Some(href)
        })
        .collect();
    Ok(links)
}

async fn fetch_products(metal_url: String) -> Result<Vec<String>, reqwest::Error> {
    let client = Client::new();
    println!("Fetching [{}]", metal_url);
    let cache = RESPONSE_CACHE.get().unwrap();
    let mut cache_guard = cache.lock().await;
    let resp = if let Some(cached) = cache_guard.get(&metal_url) {
        println!("[CACHE HIT] [{}]", metal_url);
        cached.clone().unwrap_or_default()
    } else {
        let text = client.get(&metal_url).send().await?.text().await?;
        cache_guard.insert(metal_url.clone(), Some(text.clone()));
        text
    };
    drop(cache_guard);
    let document = Html::parse_document(&resp);
    let selector = Selector::parse("a").unwrap();
    let links: Vec<String> = document
        .select(&selector)
        .filter(filter_product_links)
        .filter_map(|element| {
            let href = element.value().attr("href")?.to_string();
            Some(href)
        })
        .collect();
    Ok(links)
}

fn filter_metal_links(element: &ElementRef) -> bool {
    match element.value().attr("href") {
        Some(href) => {
            href.contains("/metals/")
        },
        None => false,
    }
}
fn filter_product_links(element: &ElementRef) -> bool {
    match element.value().attr("href") {
        Some(href) => {
            href.contains("/product/")
        },
        None => false,
    }
}
async fn fetch_metals(category_url: &str) -> Result<Vec<String>, reqwest::Error> {
    let client = Client::new();
    println!("Fetching [{}]", category_url);
    let cache = RESPONSE_CACHE.get().unwrap();
    let mut cache_guard = cache.lock().await;
    let url = category_url.to_string();
    let resp = if let Some(cached) = cache_guard.get(&url) {
        println!("[CACHE HIT] [{}]", url);
        cached.clone().unwrap_or_default()
    } else {
        let text = client.get(&url).send().await?.text().await?;
        cache_guard.insert(url.clone(), Some(text.clone()));
        text
    };
    drop(cache_guard);
    let document = Html::parse_document(&resp);
    let selector = Selector::parse("div.products-list-container a").unwrap();
    let links: Vec<String> = document
        .select(&selector)
        .filter(filter_metal_links)
        .filter_map(|element| {
            let href = element.value().attr("href")?.to_string();
            Some(href)
        })
        .collect();
    Ok(links)
}





const MAX_CONCURRENT: usize = 50; // Set your desired concurrency limit here
use once_cell::sync::OnceCell;
static RESPONSE_CACHE: OnceCell<Arc<Mutex<HashMap<String, Option<String>>>>> = OnceCell::new();

#[tokio::main]
async fn main() {
    RESPONSE_CACHE.set(Arc::new(Mutex::new(HashMap::new()))).ok();
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let metals = fetch_metals("https://www.metalsupermarkets.com/metals").await;
    match metals {
        Ok(metals_vec) => {
            // For each metal URL, call fetch_shapes and await all futures
            let shape_futs = metals_vec.iter().sorted().dedup().map(|url| {
                let url = url.clone();
                let sem = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    fetch_shapes(url).await
                })
            });
            let shape_results = futures::future::join_all(shape_futs).await;
            let all_shapes: Vec<String> = shape_results
                .into_iter()
                .filter_map(|r| r.ok().and_then(|x| x.ok()))
                .flatten()
                .sorted()
                .dedup()
                .collect();

            let product_futs = all_shapes.iter().sorted().dedup().map(|url| {
                let url = url.clone();
                let sem = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    fetch_products(url).await
                })
            });
            let product_results = futures::future::join_all(product_futs).await;
            let all_products: Vec<String> = product_results
                .into_iter()
                .filter_map(|r| r.ok().and_then(|x| x.ok()))
                .flatten()
                .sorted()
                .dedup()
                .collect();

            let sku_futs = all_products.iter().sorted().dedup().map(|url| {
                let url = url.clone();
                let sem = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    fetch_product_skus_and_ids(url).await
                })
            });
            let sku_results = futures::future::join_all(sku_futs).await;
            let all_skus: Vec<ProductInfo> = sku_results
                .into_iter()
                .filter_map(|r| r.ok().and_then(|x| x.ok()))
                .flatten()
                .sorted()
                .dedup()
                .collect();
            println!("{:#?}", all_skus)
        },
        Err(_) => todo!(),
    }
}












