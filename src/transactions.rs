use serde::Deserialize;

use crate::{Money, MoneyParseError};

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct TransactionText {
    #[serde(rename = "type")]
    kind: String,

    #[serde(rename = "client")]
    client_id: String,

    #[serde(rename = "tx")]
    transaction_id: String,
    amount: Option<String>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Id {
    pub client_id: u16,
    pub transaction_id: u32,
}

impl Into<TransactionRecord> for TransactionText {
    fn into(self) -> TransactionRecord {
        let kind = self.kind.to_lowercase();
        let id = Id {
            client_id: self.client_id.parse().unwrap(),
            transaction_id: self.transaction_id.parse().unwrap(),
        };
        let amount: Result<Money, MoneyParseError> = match self.amount {
            Some(text) => text.parse(),
            None => Ok(Money::zero()),
        };

        match kind.as_str() {
            "deposit" => TransactionRecord::Deposit {
                id,
                amount: amount.unwrap(),
            },
            "withdrawal" => TransactionRecord::Withdrawl {
                id,
                amount: amount.unwrap(),
            },
            "dispute" => TransactionRecord::Dispute { id },
            "resolve" => TransactionRecord::Resolve { id },
            "chargeback" => TransactionRecord::Chargeback { id },
            _ => todo!("Add error handling"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TransactionRecord {
    Deposit { id: Id, amount: Money },
    Withdrawl { id: Id, amount: Money },
    Dispute { id: Id },
    Resolve { id: Id },
    Chargeback { id: Id },
}

impl TransactionRecord {
    pub fn id(&self) -> Id {
        *match &self {
            TransactionRecord::Deposit { id, amount } => id,
            TransactionRecord::Withdrawl { id, amount } => id,
            TransactionRecord::Dispute { id } => id,
            TransactionRecord::Resolve { id } => id,
            TransactionRecord::Chargeback { id } => id,
        }
    }

    pub fn amount(&self) -> Money {
        match self {
            TransactionRecord::Deposit { id, amount } => *amount,
            TransactionRecord::Withdrawl { id, amount } => *amount,
            TransactionRecord::Dispute { id } => Money::zero(),
            TransactionRecord::Resolve { id } => Money::zero(),
            TransactionRecord::Chargeback { id } => Money::zero(),
        }
    }
}
