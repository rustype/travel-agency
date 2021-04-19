mod agency;
// mod bank;

use agency::agency_api::*;
use agency::*;
use std::{
    io::{stdin, stdout, Result, Write},
    usize,
};

fn main() -> Result<()> {
    // let input = stdin();
    // let mut output = stdout();
    let mut input_buffer = String::new();
    let mut session = TSession::new();
    loop {
        prompt(&mut input_buffer, &session)?;
        let split_input: Vec<_> = input_buffer.trim().split(" ").collect();
        println!("{:?}", split_input);
        session = match (split_input.first(), session) {
            (Some(&"login"), TSession::Guest(s)) => {
                if split_input.len() != 3 {
                    println!("invalid login command. usage: login <username> <password>");
                    TSession::Guest(s)
                } else {
                    match s.login(split_input.get(1).unwrap(), split_input.get(2).unwrap()) {
                        Login::Empty(empty) => {
                            println!("login successful");
                            empty.into()
                        }
                        Login::Error(error) => error.into(),
                    }
                }
            }
            (Some(&"search"), TSession::Empty(mut s)) => {
                // HACK
                if split_input.len() != 2 {
                    println!("invalid search command. usage: search <keyword>")
                } else {
                    let trips = s.search_trip(split_input[1]);
                    for (i, trip) in trips.iter().enumerate() {
                        println!("{}: {:?}", i, trip);
                    }
                }
                TSession::Empty(s)
            }
            (Some(&"search"), TSession::NonEmpty(mut s)) => {
                if split_input.len() != 2 {
                    println!("invalid search command. usage: search <keyword>")
                } else {
                    let trips = s.search_trip(split_input[1]);
                    for (i, trip) in trips.iter().enumerate() {
                        println!("{}: {:?}", i, trip);
                    }
                }
                TSession::NonEmpty(s)
            }
            (Some(&"select"), TSession::Empty(s)) => {
                if split_input.len() != 2 {
                    println!("invalid search command. usage: search <idx>");
                    TSession::Empty(s)
                } else {
                    let idx = split_input[1].parse::<usize>().unwrap();
                    match s.add_trip(idx) {
                        Selection::Empty(s) => {
                            println!("invalid index: {}", idx);
                            TSession::Empty(s)
                        }
                        Selection::NonEmpty(s) => TSession::NonEmpty(s),
                    }
                }
            }
            (Some(&"select"), TSession::NonEmpty(mut s)) => {
                if split_input.len() != 2 {
                    println!("invalid search command. usage: search <idx>");
                    TSession::NonEmpty(s)
                } else {
                    let idx = split_input[1].parse::<usize>().unwrap();
                    match s.add_trip(idx) {
                        Ok(()) => {}
                        Err(s) => println!("{}", s),
                    }
                    TSession::NonEmpty(s)
                }
            }
            (Some(&"buy"), TSession::NonEmpty(s)) => {
                if split_input.len() != 2 {
                    println!("invalid search command. usage: buy <token>");
                    TSession::NonEmpty(s)
                } else {
                    match s.buy(split_input[1]) {
                        Transaction::Empty(empty_sess) => TSession::Empty(empty_sess),
                        Transaction::TError(error) => TSession::TError(error),
                    }
                }
            }
            (Some(&"close"), TSession::Guest(_)) => {
                println!("goodbye!");
                return Ok(());
            }
            (Some(&"close"), TSession::Empty(s)) => {
                let _ = s.close();
                println!("closing session!");
                break;
            }
            (Some(&"close"), TSession::NonEmpty(s)) => {
                let _ = s.close();
                println!("closing session!");
                break;
            }
            (Some(cmd), session @ _) => {
                println!("invalid command: {}", cmd);
                session
            }
            (None, session @ _) => {
                println!("command cannot be empty!");
                session
            }
        }
    }
    Ok(())
}

fn prompt(input_buffer: &mut String, session: &TSession) -> Result<usize> {
    input_buffer.clear();
    let input = stdin();
    let mut output = stdout();
    output.write("(".as_bytes())?;
    output.write(session.to_string().as_bytes())?;
    output.write(")".as_bytes())?;
    output.write("> ".as_bytes())?;
    output.flush()?;
    input.read_line(input_buffer)
}

#[derive(Clone, Debug)]
pub struct Trip {
    from: String,
    to: String,
    price: usize,
}

impl Trip {
    fn new(from: String, to: String, price: usize) -> Self {
        Self { from, to, price }
    }

    fn matches(&self, city: &str) -> bool {
        self.from == city || self.to == city
    }

    pub fn mocks() -> Vec<Self> {
        let cities = vec!["Lisbon", "London", "Berlin", "Paris", "Amesterdam"];
        (1..5)
            .map(|i| Self::new(cities[i - 1].to_string(), cities[i].to_string(), i * 200))
            .collect()
    }
}
