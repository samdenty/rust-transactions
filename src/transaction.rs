use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_tuple::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
  Deposit,
  Withdrawal,
  Dispute,
  Resolve,
  Chargeback,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
pub struct Transaction {
  pub transaction_type: TransactionType,
  pub client: u16,
  pub id: u32,
  pub amount: Option<Decimal>,
}
