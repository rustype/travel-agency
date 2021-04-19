use crate::bank::bank_api;
use crate::bank::bank_api::{CheckBalanceState, DepositState, FinishState, WithdrawState};
use crate::Trip;
use agency_api::*;
use typestate::typestate;

#[typestate(enumerate = "TSession")]
pub mod agency_api {
    use crate::*;
    use std::result::Result;
    #[automata]
    pub struct Session;

    #[state]
    pub struct Guest;
    pub trait Guest {
        fn init() -> Guest;
        fn login(self, username: &str, password: &str) -> Login;
    }

    pub enum Login {
        Empty,
        Error,
    }

    #[state]
    pub struct Error;
    pub trait Error {
        fn close(self);
    }

    #[state]
    pub struct Empty {
        pub last_search: Vec<Trip>,
    }
    pub trait Empty {
        fn search_trip(&mut self, query: &str) -> Vec<Trip>;
        fn add_trip(self, idx: usize) -> Selection;
        fn close(self);
    }

    #[state]
    pub struct NonEmpty {
        pub last_search: Vec<Trip>,
        pub selected: Vec<Trip>,
    }
    pub trait NonEmpty {
        fn search_trip(&mut self, query: &str) -> Vec<Trip>;
        fn add_trip(&mut self, idx: usize) -> Result<(), String>;
        fn buy(self, token: &str) -> Transaction;
        fn close(self);
    }

    #[state]
    pub struct TError {
        pub selected: Vec<Trip>,
    }
    pub trait TError {
        fn retry(self) -> NonEmpty;
        fn close(self);
    }

    pub enum Selection {
        NonEmpty,
        Empty,
    }

    pub enum Transaction {
        Empty,
        TError,
    }
}

impl GuestState for Session<Guest> {
    fn init() -> Self {
        return Session::<Guest> { state: Guest };
    }
    fn login(self, username: &str, password: &str) -> Login {
        if username == "client" && password == "client" {
            Login::Empty(Session::<Empty> {
                state: Empty {
                    last_search: vec![],
                },
            })
        } else {
            Login::Error(Session::<Error> { state: Error })
        }
    }
}

impl EmptyState for Session<Empty> {
    fn search_trip(&mut self, query: &str) -> Vec<Trip> {
        let trips: Vec<Trip> = Trip::mocks()
            .into_iter()
            .filter(|trip| trip.matches(query))
            .collect();
        self.state.last_search = trips.clone();
        trips
    }
    fn add_trip(self, idx: usize) -> Selection {
        println!("{:?}", self.state.last_search);
        if idx < self.state.last_search.len() {
            Selection::NonEmpty(Session::<NonEmpty> {
                state: NonEmpty {
                    selected: vec![self.state.last_search[idx].clone()],
                    last_search: self.state.last_search,
                },
            })
        } else {
            Selection::Empty(self)
        }
    }
    fn close(self) {
        // consume
    }
}

impl NonEmptyState for Session<NonEmpty> {
    fn search_trip(&mut self, query: &str) -> Vec<Trip> {
        let trips: Vec<Trip> = Trip::mocks()
            .into_iter()
            .filter(|trip| trip.matches(query))
            .collect();
        self.state.last_search = trips.clone();
        trips
    }
    fn add_trip(&mut self, idx: usize) -> Result<(), String> {
        if idx < self.state.last_search.len() {
            self.state
                .selected
                .push(self.state.last_search[idx].clone());
            Ok(())
        } else {
            Err(format!("invalid index: {}", idx))
        }
    }
    fn buy(self, token: &str) -> Transaction {
        // TODO finish
        let mut passed = vec![true; self.state.selected.len()];
        for (i, trip) in self.state.selected.iter().enumerate() {
            let check = bank_api::Transaction::<bank_api::CheckBalance>::start_transaction(
                token,
                "travel_agency",
                trip.price,
            );
            match check.check_balance() {
                bank_api::BalanceResult::Withdraw(w) => {
                    let deposit = w.withdraw();
                    let fin = deposit.deposit();
                    fin.finish();
                    passed[i] = false;
                }
                bank_api::BalanceResult::Error(e) => {
                    use crate::bank::bank_api::ErrorState;
                    e.finish();
                    let mut selected = self.state.selected;
                    let mut j = 0;
                    selected.retain(|_| (passed[j], j += 1).0);
                    return Transaction::TError(Session::<TError> {
                        state: TError { selected },
                    });
                }
            };
        }
        Transaction::Empty(Session::<Empty> {
            state: Empty {
                last_search: vec![],
            },
        })
    }
    fn close(self) {
        // consume
    }
}

impl ErrorState for Session<Error> {
    fn close(self) {
        // consume
    }
}

impl TErrorState for Session<TError> {
    fn retry(self) -> Session<NonEmpty> {
        Session::<NonEmpty> {
            state: NonEmpty {
                last_search: vec![],
                selected: self.state.selected,
            },
        }
    }
    fn close(self) {
        // consume
    }
}

impl TSession {
    pub fn new() -> Self {
        Self::Guest(Session::<Guest>::init())
    }
}
