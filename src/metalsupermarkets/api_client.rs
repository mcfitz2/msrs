use reqwest::{Client, Response, header};
use reqwest::cookie::Jar;
use std::sync::Arc;
use serde::Serialize;

#[derive(Serialize)]
pub struct AddToCartParams<'a> {
    pub action: &'static str,
    pub store_id: &'a str,
    pub store_country: &'a str,
    pub pro_id: &'a str,
    pub pro_sku: &'a str,
    pub prowidth: Option<&'a str>,
    pub prolength: &'a str,
    pub selunits: &'a str,
    pub selquantity: &'a str,
    pub pro_price: &'a str,
}

#[derive(Serialize)]
pub struct GetProductPriceParams<'a> {
    pub action: &'static str,
    pub store_id: &'a str,
    pub store_country: &'a str,
    pub pro_id: &'a str,
    pub pro_sku: &'a str,
    pub prolength: &'a str,
    pub selunits: &'a str,
    pub selquantity: &'a str,
}
pub struct ApiClient {
    client: Client,
    pub cookie_jar: Arc<Jar>,
}

impl ApiClient {
    pub fn new() -> Self {
        let cookie_jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();
        Self { client, cookie_jar }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<Response, reqwest::Error> {
        #[derive(Serialize)]
        struct LoginForm<'a> {
            msm_action: &'static str,
            msm_redirect_to: &'static str,
            msm_email: &'a str,
            msm_password: &'a str,
            defaultstorekeylogin: &'static str,
        }
        let form = LoginForm {
            msm_action: "form_login",
            msm_redirect_to: "/my-account/my-orders/",
            msm_email: email,
            msm_password: password,
            defaultstorekeylogin: "023001",
        };
        let body = serde_urlencoded::to_string(&form).unwrap();
        self.client
            .post("https://www.metalsupermarkets.com/login/")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
    }

    pub async fn add_to_cart<'a>(&self, params: AddToCartParams<'a>) -> Result<Response, reqwest::Error> {
        let body = serde_urlencoded::to_string(&params).unwrap();
        self.client
            .post("https://www.metalsupermarkets.com/wp-admin/admin-ajax.php")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("x-requested-with", "XMLHttpRequest")
            .body(body)
            .send()
            .await
    }

    pub async fn get_cart(&self) -> Result<Response, reqwest::Error> {
        self.client
            .get("https://www.metalsupermarkets.com/wp-admin/admin-ajax.php?action=getajaxcart&mpg=addprods")
            .header("x-requested-with", "XMLHttpRequest")
            .send()
            .await
    }

    pub async fn get_product_price<'a>(&self, params: GetProductPriceParams<'a>) -> Result<Response, reqwest::Error> {
        let body = serde_urlencoded::to_string(&params).unwrap();
        self.client
            .post("https://www.metalsupermarkets.com/wp-admin/admin-ajax.php")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("x-requested-with", "XMLHttpRequest")
            .body(body)
            .send()
            .await
    }
}


