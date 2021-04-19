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
        fn try_again(self) -> NonEmpty;
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
        if token == "valid_client" {
            Transaction::Empty(Session::<Empty> {
                state: Empty {
                    last_search: vec![],
                },
            })
        } else {
            Transaction::TError(Session::<TError> {
                state: TError {
                    selected: self.state.selected,
                },
            })
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
