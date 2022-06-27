#![allow(dead_code)]
#![allow(unused_variables)]

use accounts::{AccountDatabase, AccountSummary};
use csv::{Reader, ReaderBuilder, Writer};
use std::fs::File;
use std::ops::Sub;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::{env, io};
use std::{error::Error, ops::Add};
use transactions::{TransactionRecord, TransactionText};

/*
    This is a fixed precision integer representation of money.
    In this case, our precision is 4 decimal places.

    If we were dealing with USD and cent-level precision, this would be equivalent to
    storing cents.

    The naive alternative to fixed precision is using floats.  The problem with that is
    you risk introducing rounding errors -- which is not acceptable for accounting purposes.
*/
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct Money(u64);

impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Self) -> Self::Output {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, rhs: Self) -> Self::Output {
        Money(self.0 - rhs.0)
    }
}

#[derive(Debug)]
pub enum MoneyParseError {
    ExceededPrecision,
    Malformed,
}

impl FromStr for Money {
    type Err = MoneyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.len() == 0 {
            Err(MoneyParseError::Malformed)
        } else {
            let parts: Vec<&str> = trimmed.split('.').collect();

            match parts.len() {
                0 => Err(MoneyParseError::Malformed),
                1 => Ok(Money(Money::parse_whole_part(parts[0])?)),
                2 => Ok(Money(
                    Money::parse_whole_part(parts[0])? + Money::parse_decimal_part(parts[1])?,
                )),
                _ => Err(MoneyParseError::Malformed),
            }
        }
    }
}

impl ToString for Money {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str((self.0 / 10000).to_string().as_str());
        s.push('.');
        s.push_str((self.0 % 10000).to_string().as_str());

        s
    }
}

impl Money {
    pub fn zero() -> Money {
        Money(0)
    }

    fn parse_whole_part(text: &str) -> Result<u64, MoneyParseError> {
        let whole: u64 = text.parse().map_err(|_| MoneyParseError::Malformed)?;

        if whole > u64::MAX / 10000 {
            Err(MoneyParseError::ExceededPrecision)
        } else {
            Ok(whole * 10000)
        }
    }

    fn parse_decimal_part(text: &str) -> Result<u64, MoneyParseError> {
        let decimal: u64 = text.parse().map_err(|_| MoneyParseError::Malformed)?;

        if decimal > 9999 {
            Err(MoneyParseError::ExceededPrecision)
        } else {
            Ok(decimal)
        }
    }
}

mod transactions;

mod accounts;

#[cfg(test)]
mod tests;

fn read_transactions_from_text(text: &str) -> Result<String, Box<dyn Error>> {
    let mut reader = ReaderBuilder::default()
        .trim(csv::Trim::All)
        .has_headers(true)
        .from_reader(text.as_bytes());
    let mut writer = Writer::from_writer(vec![]);

    read_transactions(&mut reader, &mut writer)?;

    let text = String::from_utf8(writer.into_inner()?)?;

    Ok(text)
}

fn read_transactions<I: io::Read, W: io::Write>(
    reader: &mut Reader<I>,
    writer: &mut Writer<W>,
) -> Result<(), Box<dyn Error>> {
    let mut accounts = AccountDatabase::new();

    for record_result in reader.deserialize() {
        let transaction_text: TransactionText = record_result?;
        let transaction: TransactionRecord = transaction_text.into();

        accounts.apply(&transaction);
    }

    for account in accounts.accounts() {
        let summary: AccountSummary = account.into();

        writer.serialize(summary)?;
    }
    writer.flush()?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: notfizzbuzz input.csv > output.csv");
        exit(0);
    }

    let path = Path::new(&args[1]);
    let file = File::open(path)?;

    let mut reader = ReaderBuilder::default()
        .trim(csv::Trim::All)
        .has_headers(true)
        .from_reader(file);
    let mut writer = Writer::from_writer(io::stdout());

    read_transactions(&mut reader, &mut writer).expect("Failed to conduct I/O");

    Ok(())
}
