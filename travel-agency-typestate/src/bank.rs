use bank_api::*;
use std::collections::HashMap;
use typestate::typestate;

#[typestate(enumerate, state_constructors)]
pub mod bank_api {
    use std::collections::HashMap;

    #[automata]
    pub struct Transaction {
        pub accounts: HashMap<String, isize>,
    }

    #[state]
    pub struct AccountValidation {
        pub from: String,
        pub to: String,
        pub amount: isize,
    }

    pub trait AccountValidation {
        fn start_transaction(from: &str, to: &str, amount: isize) -> AccountValidation;
        fn validate_accounts(self) -> AccountValidationResult;
    }

    pub enum AccountValidationResult {
        Valid,
        Error,
    }

    #[state]
    pub struct Error {
        pub message: String,
    }

    pub trait Error {
        fn finish(self);
    }

    #[state]
    pub struct Valid {
        pub from: String,
        pub to: String,
        pub amount: isize,
    }

    pub trait Valid {
        fn perform_transaction(self) -> TransactionResult;
    }

    pub enum TransactionResult {
        Finish,
        Error,
    }

    #[state]
    pub struct Finish;

    pub trait Finish {
        fn finish(self);
    }
}

impl AccountValidationState for Transaction<AccountValidation> {
    fn start_transaction(from: &str, to: &str, amount: isize) -> Transaction<AccountValidation> {
        // pretend that the accounts are the connection to a DB
        let mut accounts = HashMap::new();
        accounts.insert("valid_client".to_string(), 5000);
        accounts.insert("travel_agency".to_string(), 50000);
        Self {
            accounts,
            state: AccountValidation::new_state(from.to_string(), to.to_string(), amount),
        }
    }
    fn validate_accounts(self) -> AccountValidationResult {
        if !self.accounts.contains_key(&self.state.from) {
            AccountValidationResult::Error(Transaction::<Error> {
                accounts: self.accounts,
                state: Error::new_state("Unknown client account".to_string()),
            })
        } else if !self.accounts.contains_key(&self.state.to) {
            AccountValidationResult::Error(Transaction::<Error> {
                accounts: self.accounts,
                state: Error::new_state("Unknown destination account".to_string()),
            })
        } else {
            AccountValidationResult::Valid(Transaction::<Valid> {
                accounts: self.accounts,
                state: Valid::new_state(self.state.from, self.state.to, self.state.amount),
            })
        }
    }
}

impl ValidState for Transaction<Valid> {
    fn perform_transaction(mut self) -> TransactionResult {
        let accounts = &mut self.accounts;
        // safe unwraps
        let client_balance = accounts.get_mut(&self.state.from).unwrap();
        let amount = self.state.amount;
        if *client_balance - amount < 0 {
            return TransactionResult::Error(Transaction::<Error> {
                accounts: self.accounts,
                state: Error::new_state("Insufficient funds".to_string()),
            });
        }

        *client_balance -= amount;

        let destination_balance = accounts.get_mut(&self.state.to).unwrap();
        *destination_balance += amount;
        TransactionResult::Finish(Transaction::<Finish> {
            accounts: self.accounts,
            state: Finish,
        })
    }
}

impl ErrorState for Transaction<Error> {
    fn finish(self) {
        // consume
    }
}

impl FinishState for Transaction<Finish> {
    fn finish(self) {
        // consume
    }
}
