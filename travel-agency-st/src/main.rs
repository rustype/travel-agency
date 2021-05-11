use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
    thread, usize,
};

use session_types::{
    offer, session_channel, Branch, Chan, Choose, Eps, HasDual, Offer, Rec, Recv, Send, Var, Z,
};

macro_rules! offer_chain {
    ($ty:ty) => {
        ::session_types::Offer<$ty, ::session_types::Eps>
    };
    ($ty:ty, $($tys:ty),+) => {
        ::session_types::Offer<$ty, offer_chain!( $($tys),+ )>
    };
}

struct LoginDetails {
    username: String,
    password: String,
}

impl LoginDetails {
    fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

struct LoginError {
    error_message: String,
}

impl LoginError {
    fn new(error_message: String) -> Self {
        Self { error_message }
    }
}

#[derive(Debug, Clone)]
struct Trip {
    from: String,
    to: String,
}

impl Trip {
    fn new(from: String, to: String) -> Self {
        Self { from, to }
    }

    fn mock() -> Vec<Self> {
        let mut res = vec![];
        res.push(Self::new("Lisbon".to_string(), "Berlin".to_string()));
        res.push(Self::new("Lisbon".to_string(), "London".to_string()));
        res.push(Self::new("Beijing".to_string(), "Tokyo".to_string()));
        res.push(Self::new("Amsterdam".to_string(), "London".to_string()));
        res.push(Self::new("London".to_string(), "Berlin".to_string()));
        res
    }

    fn matches(&self, location: &str) -> bool {
        self.from == location || self.to == location
    }
}

#[derive(Debug)]
struct Search(String);
#[derive(Debug)]
struct SearchResult(Vec<Trip>);
#[derive(Debug)]
struct Select(usize);
#[derive(Debug)]
struct SelectResult;
#[derive(Debug)]
struct Buy(Vec<Trip>, String);
#[derive(Debug)]
struct BuyResult;

type RecCommand<S, R> = Recv<S, Send<R, Var<Z>>>;

// Recv<Search, Send<SearchResult, Var<Z>>>
type SearchCmd = RecCommand<Search, SearchResult>;
// Recv<Select, Send<SelectResult, Var<Z>>>
type SelectCmd = RecCommand<Select, SelectResult>;
// Recv<Buy, Send<BuyResult, Var<Z>>>
type BuyCmd = RecCommand<Buy, BuyResult>;

// Offer<SearchCmd, Offer<SelectCmd, Offer<BuyCmd, Offer<Var<Z>, Eps>>>>
//
// when "expanded":
//
// Offer<
//   Recv<Search, Send<SearchResult, Var<Z>>>,
//   Offer<
//     Recv<Select, Send<SelectResult, Var<Z>>>,
//     Offer<
//       Recv<Buy, Send<BuyResult, Var<Z>>>,
//       Offer<
//         Var<Z>, Eps
//       >
//     >
//   >
// >
type Commands = offer_chain!(SearchCmd, SelectCmd, BuyCmd, Var<Z>);

type PostLogin = Rec<Commands>;
type AgencyServer = Recv<LoginDetails, Choose<PostLogin, Send<LoginError, Eps>>>;
type AgencyClient = <AgencyServer as HasDual>::Dual;

fn main() {
    let (server_chan, client_chan): (Chan<(), AgencyServer>, Chan<(), AgencyClient>) =
        session_channel();
    let server_thread = thread::spawn(move || agency_server(server_chan));
    let client_thread = thread::spawn(move || agency_client(client_chan));
    let _ = (server_thread.join(), client_thread.join());
}

fn agency_server(c: Chan<(), AgencyServer>) {
    authentication(c);
}

fn authentication(c: Chan<(), AgencyServer>) {
    let (c, login_details) = c.recv();
    if login_details.username == "client" && login_details.password == "client" {
        let c = c.sel1();
        post_authentication(c);
    } else {
        c.sel2()
            .send(LoginError::new("failed authentication".to_string()))
            .close();
    }
}

fn post_authentication(c: Chan<(), PostLogin>) {
    let trips = Trip::mock();
    let mut c = c.enter();
    // the offer! macro does not work in loops
    loop {
        c = match c.offer() {
            Branch::Left(c) => {
                let (c, res) = c.recv();
                println!("{:?}", res);

                c.send(SearchResult(
                    trips
                        .iter()
                        .filter(|trip| trip.matches(&res.0))
                        .cloned()
                        .collect(),
                ))
                .zero()
            }
            Branch::Right(c) => match c.offer() {
                Branch::Left(c) => {
                    let (c, res) = c.recv();
                    println!("{:?}", res);
                    c.send(SelectResult).zero()
                }
                Branch::Right(c) => match c.offer() {
                    Branch::Left(c) => {
                        let (c, res) = c.recv();
                        println!("{:?}", res);
                        launch_bank(res).and_then(|err| Some(println!("{:?}", err)));
                        c.send(BuyResult).zero()
                    }
                    Branch::Right(c) => match c.offer() {
                        Branch::Left(c) => c.zero(),
                        Branch::Right(c) => {
                            c.close();
                            break;
                        }
                    },
                },
            },
        };
    }
}

fn agency_client(c: Chan<(), AgencyClient>) {
    let mut last_trips = vec![];
    let mut selected_trips = vec![];

    let mut prompt_buffer = String::new();
    let input = stdin();
    let mut output = stdout();

    output
        .write("insert username and password:\n".as_bytes())
        .unwrap();
    output.flush().unwrap();

    output.write("username: ".as_bytes()).unwrap();
    output.flush().unwrap();
    input.read_line(&mut prompt_buffer).unwrap();
    let username = prompt_buffer.trim().to_string();
    prompt_buffer.clear();

    output.write("password: ".as_bytes()).unwrap();
    output.flush().unwrap();
    input.read_line(&mut prompt_buffer).unwrap();
    let password = prompt_buffer.trim().to_string();
    prompt_buffer.clear();

    let c = c.send(LoginDetails::new(username, password));

    match c.offer() {
        Branch::Right(c) => {
            let (c, error) = c.recv();
            println!("{:?}", error.error_message);
            c.close();
        }
        Branch::Left(c) => {
            let mut c = c.enter();
            loop {
                output
                    .write("insert command (quit/close to end):\n".as_bytes())
                    .unwrap();
                output.flush().unwrap();

                input.read_line(&mut prompt_buffer).unwrap();
                let command = prompt_buffer.trim().to_string();
                prompt_buffer.clear();
                c = if command == "search" {
                    input.read_line(&mut prompt_buffer).unwrap();
                    let query = prompt_buffer.trim().to_string();
                    prompt_buffer.clear();

                    let (c, res) = c.sel1().send(Search(query)).recv();
                    println!("{:?}", res);
                    last_trips = res.0;
                    c.zero()
                } else {
                    let c = c.sel2();
                    if command == "select" {
                        loop {
                            input.read_line(&mut prompt_buffer).unwrap();
                            let query = prompt_buffer.trim().to_string();
                            prompt_buffer.clear();
                            // BUG if no search was made, the trip index will never be valid
                            if let Ok(index) = query.parse::<usize>() {
                                if index < last_trips.len() {
                                    let (c, res) = c.sel1().send(Select(index)).recv();
                                    selected_trips.push(last_trips[index].clone());
                                    println!("{:?}", res);
                                    println!("{:?}", selected_trips);
                                    break c.zero();
                                }
                            }
                            println!("invalid index");
                        }
                    } else {
                        let c = c.sel2();
                        if command == "buy" {
                            let (c, res) = c
                                .sel1()
                                .send(Buy(selected_trips, "valid_client".to_string()))
                                .recv();
                            selected_trips = vec![];
                            println!("{:?}", res);
                            c.zero()
                        } else {
                            let c = c.sel2();
                            if command == "quit" || command == "close" {
                                c.sel2().close();
                                break;
                            } else {
                                c.sel1().zero()
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct Tokens(String, String);
#[derive(Debug)]
struct Transfer(u64);
#[derive(Debug)]
struct BankError(String);

type BankServer =
    Recv<Tokens, Choose<Recv<Transfer, Choose<Eps, Send<BankError, Eps>>>, Send<BankError, Eps>>>;
type BankClient = <BankServer as HasDual>::Dual;

fn launch_bank(buy_request: Buy) -> Option<BankError> {
    let (server_chan, client_chan): (Chan<(), BankServer>, Chan<(), BankClient>) =
        session_channel();
    let server_thread = thread::spawn(move || bank_server(server_chan));
    let client_thread = thread::spawn(move || bank_client(client_chan, buy_request));
    let (_, client_res) = (server_thread.join(), client_thread.join().unwrap());
    client_res
}

fn bank_server(c: Chan<(), BankServer>) {
    // HACK this is kinda BS since the function is "stateless" and the accounts HM should be "stateful"
    let mut accounts: HashMap<String, u64> = HashMap::new();
    accounts.insert("travel_agency".to_string(), 5000);
    accounts.insert("valid_client".to_string(), 500);
    let (c, tokens) = c.recv();
    if tokens.0 == "travel_agency" && tokens.1 == "valid_client" {
        let (c, transfer) = c.sel1().recv();
        if accounts[&tokens.1] > transfer.0 {
            accounts
                .get_mut(&tokens.1)
                .and_then(|amount| Some(*amount -= transfer.0)); // HACK
            accounts
                .get_mut(&tokens.0)
                .and_then(|amount| Some(*amount += transfer.0)); // HACK
            println!("{:#?}", accounts);
            c.sel1().close();
        } else {
            c.sel2()
                .send(BankError(String::from("insufficient funds")))
                .close();
        }
    } else {
        c.sel2()
            .send(BankError(String::from("invalid tokens")))
            .close();
    }
}

fn bank_client(c: Chan<(), BankClient>, buy_request: Buy) -> Option<BankError> {
    match c
        .send(Tokens("travel_agency".to_string(), buy_request.1))
        .offer()
    {
        Branch::Left(c) => match c.send(Transfer((buy_request.0.len() * 50) as u64)).offer() {
            Branch::Left(c) => {
                println!("bank: transfer has been successful");
                c.close();
                None
            }
            Branch::Right(c) => {
                let (c, err) = c.recv();
                println!("bank: {:?}", err);
                c.close();
                Some(err)
            }
        },
        Branch::Right(c) => {
            let (c, err) = c.recv();
            println!("bank: {:?}", err);
            c.close();
            Some(err)
        }
    }
}
