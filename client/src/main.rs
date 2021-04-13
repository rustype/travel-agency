use typestate::typestate;

const AGENCY_PORT: u16 = 9001;

fn main() {
    println!("Hello, world!");
}

struct Trip {
    from: String,
    to: String,
    price: u32,
}
#[typestate]
mod agency {
    use super::*;
    #[automata]
    struct Cart;

    #[state]
    struct Empty;

    trait Empty {
        fn create() -> Empty;
        fn search_trip(self, query: &str);
        fn add_trip(self, trip: Trip) -> NonEmpty;
        fn close(self);
    }

    #[state]
    struct NonEmpty {
        selected: Vec<Trip>,
    }

    trait NonEmpty {
        fn add_trip(&mut self, trip: Trip);
        fn buy(self, token: &str) -> Empty;
        fn close(self);
    }
}

#[typestate]
mod bank {
    #[automata]
    struct Transaction;

    #[state]
    struct CheckBalance;
    trait CheckBalance {
        fn start_transaction(from: &str, to: &str, amount: u32) -> CheckBalance;
        fn check_balance(self) -> BalanceCheck;
    }

    enum BalanceCheck {
        Withdraw,
        Error,
    }

    #[state]
    struct Error {
        message: String
    }
    trait Error {
        fn finish(self);
    }

    #[state]
    struct Withdraw;
    trait Withdraw {
        fn withdraw(self) -> Deposit;
    }

    #[state]
    struct Deposit;
    trait Deposit {
        fn deposit(self) -> TransactionCheck;
    }

    #[state]
    struct Rollback;
    trait Rollback {
        fn rollback(self);
    }

    #[state]
    struct Finish;
    trait Finish {
        fn finish(self);
    }

    enum TransactionCheck {
        Finish,
        Rollback,
    }
}