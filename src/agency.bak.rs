use typestate::typestate;

#[typestate]
mod agency {
    use super::*;
    #[automata]
    struct Cart;

    enum Login {
        LoginError,
        Empty,
    }

    #[state]
    struct LoginError;
    trait LoginError {
        fn close(self);
    }

    #[state]
    struct Empty;

    trait Empty {
        fn login(username: &str, password: &str) -> Empty;
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

