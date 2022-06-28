use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, HashSet},
};

use serde::Serialize;

use crate::{
    transactions::{TransactionRecord, TransactionText},
    Money,
};

#[derive(PartialEq, Eq, Debug)]
pub enum AccountStatus {
    /*
    If we receive an invalid transaction, and we're able to link it to a particular
    client account then we transition to an error state -- we don't actually know
    what the status of the account is.
    */
    Unknown(TransactionText),
    Active,
    Locked,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Account {
    client_id: u16,
    available: Money,
    held: Money,
    status: AccountStatus,
}

#[derive(Serialize)]
pub struct AccountSummary {
    client_id: u16,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

impl Into<AccountSummary> for &Account {
    fn into(self) -> AccountSummary {
        AccountSummary {
            client_id: self.client_id,
            available: self.available.to_string(),
            held: self.held.to_string(),
            total: (self.available + self.held).to_string(),
            locked: self.status == AccountStatus::Locked,
        }
    }
}

impl Account {
    pub fn create(client_id: u16) -> Account {
        Account {
            client_id,
            available: Money::zero(),
            held: Money::zero(),
            status: AccountStatus::Active,
        }
    }

    pub fn apply(&mut self, transaction: &TransactionRecord, disputed_amount: Money) {
        match *transaction {
            TransactionRecord::Deposit { id, amount } => self.available = self.available + amount,
            TransactionRecord::Withdrawl { id, amount } => {
                if amount < self.available {
                    self.available = self.available - amount;
                }
            }
            TransactionRecord::Dispute { id } => {
                self.held = self.held + min(self.available, disputed_amount);
                self.available = self.available - min(self.available, disputed_amount);
            }
            TransactionRecord::Resolve { id } => {
                self.available = self.available + min(self.held, disputed_amount);
                self.held = self.held - min(self.held, disputed_amount);
            }
            TransactionRecord::Chargeback { id } => {
                if disputed_amount > Money::zero() {
                    self.status = AccountStatus::Locked;
                }
                self.available = self.available + min(self.held, disputed_amount);
                self.held = self.held - min(self.held, disputed_amount);
            }
        }
    }
}

pub struct AccountDatabase {
    accounts: BTreeMap<u16, Account>,

    /*
    We absolutely must persist all transactions such that we can always replay them to
    achieve the same final state.

    In practice, I would persist transaction data and possibly account snapshots in some database.
    We could then batch commit them at a reasonable cadence to avoid consuming too much memory
    while ingesting transactions.

    For the purposes of this assignment though, I'm going to just store them in memory.
    */
    transactions: HashMap<u32, TransactionRecord>,

    /*
    Storing the actual set of disupted transactions may be a bit unorthodox vs.
    storing a status field on each transaction.

    Since we only care about disputed transactions, it's cheaper to store just those IDs
    under dispute vs. increasing memory on all undisputed transactions.

    If transactions had a more complex life cycle then we'd probably want a status enum.
    */
    disputed_transactions: HashSet<u32>,
}

impl AccountDatabase {
    pub fn new() -> AccountDatabase {
        AccountDatabase {
            accounts: BTreeMap::new(),
            transactions: HashMap::new(),
            disputed_transactions: HashSet::new(),
        }
    }

    pub fn apply(&mut self, transaction: &TransactionRecord) {
        let client_id = transaction.id().client_id;
        let account = self
            .accounts
            .entry(client_id)
            .or_insert(Account::create(client_id));

        if AccountDatabase::can_process_transaction(
            transaction,
            &self.transactions,
            &self.disputed_transactions,
        ) {
            AccountDatabase::record_transaction(
                transaction,
                &mut self.transactions,
                &mut self.disputed_transactions,
            );

            let disputed_amount =
                AccountDatabase::get_disputed_amount(transaction, &self.transactions);

            account.apply(transaction, disputed_amount);
        }
    }

    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    fn can_process_transaction(
        transaction: &TransactionRecord,
        recorded_transactions: &HashMap<u32, TransactionRecord>,
        disputed_transactions: &HashSet<u32>,
    ) -> bool {
        let transaction_has_been_recorded =
            recorded_transactions.contains_key(&transaction.id().transaction_id);
        let transaction_is_currently_disputed =
            disputed_transactions.contains(&transaction.id().transaction_id);
        let client_ids_are_consistent = recorded_transactions
            .get(&transaction.id().transaction_id)
            .map_or(true, |t| t.id().client_id == transaction.id().client_id);

        match transaction {
            TransactionRecord::Deposit { id, amount } => !transaction_has_been_recorded,
            TransactionRecord::Withdrawl { id, amount } => !transaction_has_been_recorded,
            TransactionRecord::Dispute { id } => {
                transaction_has_been_recorded
                    && !transaction_is_currently_disputed
                    && client_ids_are_consistent
            }
            TransactionRecord::Resolve { id } => {
                transaction_has_been_recorded
                    && transaction_is_currently_disputed
                    && client_ids_are_consistent
            }
            TransactionRecord::Chargeback { id } => {
                transaction_has_been_recorded
                    && transaction_is_currently_disputed
                    && client_ids_are_consistent
            }
        }
    }

    fn get_disputed_amount(
        transaction: &TransactionRecord,
        recorded_transactions: &HashMap<u32, TransactionRecord>,
    ) -> Money {
        let related_transaction = match transaction {
            TransactionRecord::Deposit { id, amount } => None,
            TransactionRecord::Withdrawl { id, amount } => None,
            TransactionRecord::Dispute { id } => recorded_transactions.get(&id.transaction_id),
            TransactionRecord::Resolve { id } => recorded_transactions.get(&id.transaction_id),
            TransactionRecord::Chargeback { id } => recorded_transactions.get(&id.transaction_id),
        };

        match related_transaction {
            Some(disputed_transaction) => disputed_transaction.amount(),
            None => Money::zero(),
        }
    }

    fn record_transaction(
        transaction: &TransactionRecord,
        transactions: &mut HashMap<u32, TransactionRecord>,
        disputed_transactions: &mut HashSet<u32>,
    ) {
        match transaction {
            TransactionRecord::Deposit { id, amount } => {
                transactions.insert(transaction.id().transaction_id, *transaction);
            }
            TransactionRecord::Withdrawl { id, amount } => {
                transactions.insert(transaction.id().transaction_id, *transaction);
            }
            TransactionRecord::Dispute { id } => {
                disputed_transactions.insert(transaction.id().transaction_id);
            }
            TransactionRecord::Resolve { id } => {
                disputed_transactions.remove(&transaction.id().transaction_id);
            }
            TransactionRecord::Chargeback { id } => {
                disputed_transactions.remove(&transaction.id().transaction_id);
            }
        }
    }
}
