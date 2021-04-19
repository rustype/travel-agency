use typestate::typestate;

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