use super::models;
use scraper::{ElementRef, Html, Selector};
use std::fs::File;
use serde_json;
// --- Pipeline fetcher functions ---
#[allow(dead_code)]
pub async fn fetch_metals(category_url: &str) -> Result<std::collections::HashSet<String>, reqwest::Error> {
    use std::collections::HashSet;
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build().unwrap();
    let url = category_url.to_string();
    let mut last_err = None;
    for attempt in 1..=3 {
        match client.get(&url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let document = Html::parse_document(&text);
                    let selector = Selector::parse("div.products-list-container a").unwrap();
                    let metals: HashSet<String> = document
                        .select(&selector)
                        .filter_map(|el| {
                            let url = el.value().attr("href")?.to_string();
                            let name = el.text().collect::<Vec<_>>().join("").trim().to_string();
                            if url.contains("/metals/") && !name.is_empty() {
                                Some(url)
                            } else {
                                None
                            }
                        })
                        .collect();
                    return Ok(metals);
                }
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(e),
        }
        println!("[DEBUG] fetch_metals attempt {} failed, retrying...", attempt);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    Err(last_err.unwrap())
}

#[allow(dead_code)]
pub async fn fetch_shapes(metal_url: String) -> Result<std::collections::HashSet<String>, reqwest::Error> {
    use std::collections::HashSet;
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build().unwrap();
    let mut last_err = None;
    for attempt in 1..=3 {
        match client.get(&metal_url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let document = Html::parse_document(&text);
                    let selector = Selector::parse("div > a").unwrap();
                    let links: HashSet<String> = document
                        .select(&selector)
                        .filter(|el| el.value().attr("href").map_or(false, |h| h.contains("/metals/")))
                        .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
                        .collect();
                    return Ok(links);
                }
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(e),
        }
        println!("[DEBUG] fetch_shapes attempt {} failed, retrying...", attempt);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    Err(last_err.unwrap())
}

#[allow(dead_code)]
pub async fn fetch_products(shape_url: String) -> Result<std::collections::HashSet<String>, reqwest::Error> {
    use std::collections::HashSet;
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build().unwrap();
    let mut last_err = None;
    for attempt in 1..=3 {
        match client.get(&shape_url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let document = Html::parse_document(&text);
                    let main_selector = Selector::parse("main").unwrap();
                    let a_selector = Selector::parse("a").unwrap();
                    let mut links = HashSet::new();
                    if let Some(main) = document.select(&main_selector).next() {
                        for el in main.select(&a_selector) {
                            if let Some(href) = el.value().attr("href") {
                                if href.contains("/product/") {
                                    links.insert(href.to_string());
                                }
                            }
                        }
                    }
                    return Ok(links);
                }
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(e),
        }
        println!("[DEBUG] fetch_products attempt {} failed, retrying...", attempt);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    Err(last_err.unwrap())
}

#[allow(dead_code)]
pub async fn fetch_product_skus_and_ids(product_url: String) -> Result<Vec<models::ProductInfo<'static>>, reqwest::Error> {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build().unwrap();
    let mut last_err = None;
    for attempt in 1..=3 {
        match client.get(&product_url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let document = Html::parse_document(&text);
                    let mut products = Vec::new();
                    let price_selector = Selector::parse("tr").unwrap();
                    let input_selector = Selector::parse("input").unwrap();
                    for (_i, tr) in document.select(&price_selector).enumerate() {
                        // Use owned Strings for local variables
                        let mut sku = String::new();
                        let mut id = String::new();
                        let mut qualifier_a = String::new();
                        let mut qualifier_b = String::new();
                        let mut qualifier_c = String::new();
                        let mut description = String::new();
                        let mut requires_length = true;
                        let mut requires_width = false;
                        for input in tr.select(&input_selector) {
                            if let Some(name) = input.value().attr("name") {
                                if let Some(val) = input.value().attr("value") {
                                    match name {
                                        "pro_id" => id = val.to_string(),
                                        "pro_length" => requires_length = true,
                                        "pro_width" => requires_width = true,
                                        "pro_sku" => sku = val.to_string(),
                                        "pro_size1" => qualifier_a = val.to_string(),
                                        "pro_size2" => qualifier_b = val.to_string(),
                                        "pro_size3" => qualifier_c = val.to_string(),
                                        "prosize2" => description = val.to_string(),
                                        _ => {},
                                    }
                                }
                            }
                        }
                        if !(id.is_empty() || sku.is_empty()) {
                            // Leak the strings to get &'static str for ProductInfo
                            let sku_static: &'static str = Box::leak(sku.into_boxed_str());
                            let id_static: &'static str = Box::leak(id.into_boxed_str());
                            let qualifier_a_static: &'static str = Box::leak(qualifier_a.into_boxed_str());
                            let qualifier_b_static: &'static str = Box::leak(qualifier_b.into_boxed_str());
                            let qualifier_c_static: &'static str = Box::leak(qualifier_c.into_boxed_str());
                            let description_static: &'static str = Box::leak(description.into_boxed_str());
                            let product = models::ProductInfo {
                                sku: sku_static,
                                id: id_static,
                                qualifier_a: qualifier_a_static,
                                qualifier_b: qualifier_b_static,
                                qualifier_c: qualifier_c_static,
                                description: description_static,
                                requires_length,
                                requires_width
                            };
                            //println!("[DEBUG] Parsed product: {:#?}", product);
                            products.push(product);
                        }
                    }
                    return Ok(products);
                }
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(e),
        }
        println!("[DEBUG] fetch_product_skus_and_ids attempt {} failed, retrying...", attempt);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    Err(last_err.unwrap())
}

async fn fetch_stores() -> Result<Vec<models::Store>, reqwest::Error> {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build().unwrap();
    let resp = client.get("https://www.metalsupermarkets.com/store-finder/").send().await?.text().await?;
    let document = Html::parse_document(&resp);
    let btn_selector = Selector::parse("a.mystoresetbtn").unwrap();
    let h4_selector = Selector::parse("h4 > a").unwrap();
    let location_selector = Selector::parse("div.locationlists").unwrap();
    Ok(document.select(&location_selector).map(|div| -> Option<models::Store> {
        if let (Some(btn_element), Some(name_element)) = (div.select(&btn_selector).next(), div.select(&h4_selector).next()) {
            Some(models::Store {
                id: btn_element.value().attr("data-storeid").unwrap().to_string(),
                name: name_element.text().next().unwrap().to_string(),
            })
        } else {
            // println!("------------------------------------------------------------------");
            // println!("{:#?}", div.inner_html());
            // this currently fails when there is no "set as my store" button
            None
        }
    }).flatten().collect())
}


#[allow(dead_code)]
pub async fn gather() {
    let mut products: Vec<models::ProductInfo> = Vec::new();

    let metals = fetch_metals("https://www.metalsupermarkets.com/metals").await.expect("Failed to fetch metals");
    let total_filtered = metals.len();
    for (i, metal_url) in metals.iter().enumerate() {
        println!("[DEBUG] Processing metal {} of {}: {}", i + 1, total_filtered, metal_url);
        if let Ok(shapes) = fetch_shapes(metal_url.clone()).await {
            let total_shapes = shapes.len();
            for (j, shape_url) in shapes.iter().enumerate() {
                println!("[DEBUG]   Processing shape {} of {}: {}", j + 1, total_shapes, shape_url);
                if let Ok(product_urls) = fetch_products(shape_url.clone()).await {
                    let total_products = product_urls.len();
                    for (k, product_url) in product_urls.iter().enumerate() {
                        println!("[DEBUG]     Processing product {} of {}: {}", k + 1, total_products, product_url);
                        if let Ok(infos) = fetch_product_skus_and_ids(product_url.clone()).await {
                            products.extend(infos)
                        } else {
                            println!("[DEBUG]     Failed to fetch product SKUs/IDs for {}", product_url);
                        }
                    }
                } else {
                    println!("[DEBUG]   Failed to fetch products for shape {}", shape_url);
                }
            }
        } else {
            println!("[DEBUG] Failed to fetch shapes for metal {}", metal_url);
        }
    }

    // Fetch stores
    let stores = match fetch_stores().await {
        Ok(s) => s,
        Err(e) => {
            println!("[DEBUG] Failed to fetch stores: {}", e);
            Vec::new()
        }
    };

    // Write both products and stores to src/resources/products.json as an object
    let file = File::create("src/resources/products.json").expect("Failed to create products.json");
    let output = serde_json::json!({
        "products": products,
        "stores": stores
    });
    serde_json::to_writer_pretty(file, &output).expect("Failed to write products.json");
}