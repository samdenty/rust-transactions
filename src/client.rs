use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
  pub available: Decimal,
  pub held: Decimal,
  pub total: Decimal,
  pub locked: bool,
}
