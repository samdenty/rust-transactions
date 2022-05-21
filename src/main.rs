mod client;
mod transaction;

use client::Client;
use rust_decimal_macros::dec;
use std::error::Error;
use std::io::BufWriter;
use std::{collections::HashMap, env, io::Write};
use tabwriter::TabWriter;
use transaction::{Transaction, TransactionType};

fn main() -> Result<(), Box<dyn Error>> {
  let args: Vec<String> = env::args().collect();
  let file_name = args.get(1).expect("No file specified");

  let mut rdr = csv::ReaderBuilder::new()
    .trim(csv::Trim::All)
    .from_path(file_name)?;

  let mut clients: HashMap<u16, Client> = HashMap::new();

  let transactions: Vec<Transaction> = rdr
    .deserialize()
    .filter_map(|result: Result<Transaction, csv::Error>| result.ok())
    .collect();

  let balance_transactions: HashMap<u32, &Transaction> = transactions
    .iter()
    .filter(|transaction| {
      transaction.transaction_type == TransactionType::Deposit
        || transaction.transaction_type == TransactionType::Withdrawal
    })
    .map(|transaction| (transaction.id, transaction))
    .collect();

  for transaction in &transactions {
    let client = clients.entry(transaction.client).or_insert_with(|| Client {
      available: dec!(0),
      held: dec!(0),
      total: dec!(0),
      locked: false,
    });

    match transaction.transaction_type {
      TransactionType::Deposit => {
        let amount = transaction.amount.expect(&format!(
          "expected deposit to have amount {:?}",
          transaction
        ));
        client.available += amount;
        client.total += amount;
      }
      TransactionType::Withdrawal => {
        let amount = transaction.amount.expect(&format!(
          "expected withdrawal to have amount {:?}",
          transaction
        ));
        let available = client.available - amount;

        if available > dec!(0) {
          client.available -= amount;
          client.total -= amount;
        }
      }
      TransactionType::Dispute => {
        let disputed_transaction = balance_transactions.get(&transaction.id);
        if disputed_transaction.is_none() {
          continue;
        }
        let disputed_transaction = disputed_transaction.unwrap();

        let amount = match disputed_transaction.transaction_type {
          TransactionType::Deposit => disputed_transaction.amount.expect(&format!(
            "expected disputed deposit to have amount {:?}",
            disputed_transaction
          )),
          TransactionType::Withdrawal => -disputed_transaction.amount.expect(&format!(
            "expected disputed withdrawal to have amount {:?}",
            disputed_transaction
          )),
          _ => unreachable!(),
        };

        client.available -= amount;
        client.held += amount;
      }
      TransactionType::Resolve => {
        let resolved_transaction = balance_transactions.get(&transaction.id);
        if resolved_transaction.is_none() {
          continue;
        }
        let resolved_transaction = resolved_transaction.unwrap();

        let amount = match resolved_transaction.transaction_type {
          TransactionType::Deposit => resolved_transaction.amount.expect(&format!(
            "expected resolved deposit to have amount {:?}",
            resolved_transaction
          )),
          TransactionType::Withdrawal => -resolved_transaction.amount.expect(&format!(
            "expected resolved withdrawal to have amount {:?}",
            resolved_transaction
          )),
          _ => unreachable!(),
        };

        client.available += amount;
        client.held -= amount;
      }
      TransactionType::Chargeback => {
        let chargebacked_transaction = balance_transactions.get(&transaction.id);
        if chargebacked_transaction.is_none() {
          continue;
        }

        client.locked = true;
      }
    }
  }

  // write CSV to stdout, with pretty printing
  let mut tw = TabWriter::new(std::io::stdout());
  let mut buf = BufWriter::new(Vec::new());

  {
    let mut wtr = csv::Writer::from_writer(&mut buf);
    wtr.write_record(&["client", "available", "held", "total", "chargeback"])?;
    for (client_id, client) in clients {
      wtr.write_record(&[
        client_id.to_string(),
        client.available.to_string(),
        client.held.to_string(),
        client.total.to_string(),
        client.locked.to_string(),
      ])?;
    }
    wtr.flush()?;
  }

  let csv = String::from_utf8(buf.into_inner()?)?.replace(",", ",\t");
  tw.write_all(csv.as_bytes())?;
  tw.flush()?;

  Ok(())
}
