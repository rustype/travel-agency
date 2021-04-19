use bank::*;
use std::collections::HashMap;
use typestate::typestate;

#[typestate(enumerate)]
mod bank {
    use std::collections::HashMap;

    #[automata]
    pub struct Transaction {
        pub accounts: HashMap<String, usize>,
    }

    #[state]
    pub struct CheckBalance {
        pub from: String,
        pub to: String,
        pub amount: usize,
    }
    pub trait CheckBalance {
        fn start_transaction(from: &str, to: &str, amount: usize) -> CheckBalance;
        fn check_balance(self) -> BalanceResult;
    }

    pub enum BalanceResult {
        Withdraw,
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
    pub struct Withdraw {
        pub from: String,
        pub to: String,
        pub amount: usize,
    }
    pub trait Withdraw {
        fn withdraw(self) -> Deposit;
    }

    #[state]
    pub struct Deposit {
        pub from: String,
        pub to: String,
        pub amount: usize,
    }
    pub trait Deposit {
        fn deposit(self) -> Finish;
    }

    #[state]
    pub struct Finish;
    pub trait Finish {
        fn finish(self);
    }
}

impl CheckBalanceState for Transaction<CheckBalance> {
    fn start_transaction(from: &str, to: &str, amount: usize) -> Transaction<CheckBalance> {
        Transaction::<CheckBalance> {
            accounts: HashMap::new(),
            state: CheckBalance {
                from: from.to_string(),
                to: to.to_string(),
                amount,
            },
        }
    }
    fn check_balance(self) -> BalanceResult {
        if let Some(&amount) = self.accounts.get(&self.state.from) {
            if let None = self.accounts.get(&self.state.to) {
                return BalanceResult::Error(Transaction::<Error> {
                    accounts: self.accounts,
                    state: Error {
                        message: String::from("destination account does not exist!"),
                    },
                });
            }
            if amount > self.state.amount {
                BalanceResult::Withdraw(Transaction::<Withdraw> {
                    accounts: self.accounts,
                    state: Withdraw {
                        from: self.state.from,
                        to: self.state.to,
                        amount: self.state.amount,
                    },
                })
            } else {
                BalanceResult::Error(Transaction::<Error> {
                    accounts: self.accounts,
                    state: Error {
                        message: String::from("insufficient funds!"),
                    },
                })
            }
        } else {
            BalanceResult::Error(Transaction::<Error> {
                accounts: self.accounts,
                state: Error {
                    message: String::from("account does not exist"),
                },
            })
        }
    }
}

impl WithdrawState for Transaction<Withdraw> {
    fn withdraw(mut self) -> Transaction<Deposit> {
        let balance = self.accounts.get_mut(&self.state.from).unwrap(); // previously checked
        *balance -= self.state.amount;
        Transaction::<Deposit> {
            accounts: self.accounts,
            state: Deposit {
                from: self.state.from,
                to: self.state.to,
                amount: self.state.amount,
            },
        }
    }
}

impl DepositState for Transaction<Deposit> {
    fn deposit(mut self) -> Transaction<Finish> {
        let balance = self.accounts.get_mut(&self.state.to).unwrap(); // previously checked
        *balance -= self.state.amount;
        Transaction::<Finish> {
            accounts: self.accounts,
            state: Finish,
        }
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
