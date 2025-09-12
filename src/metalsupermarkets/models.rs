use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductInfo<'a> {
	pub sku: &'a str,
	pub id: &'a str,
	pub qualifier_a: &'a str,
	pub qualifier_b: &'a str,
	pub qualifier_c: &'a str,
	pub description: &'a str,
	pub requires_length: bool,
	pub requires_width: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Store {
	pub id: String,
	pub page_id: String,
	pub name: String,
}