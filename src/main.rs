mod agency;
// mod bank;

use agency::agency_api::*;
use std::{
    io::{stdin, stdout, Result, Write},
    usize,
};

const LOGIN: &'static str = "login";
const SEARCH: &'static str = "search";
const SELECT: &'static str = "select";
const CLOSE: &'static str = "close";
const BUY: &'static str = "buy";
const RETRY: &'static str = "retry";

fn main() -> Result<()> {
    // let input = stdin();
    // let mut output = stdout();
    let mut input_buffer = String::new();
    let mut session = TSession::new();
    loop {
        prompt(&mut input_buffer, &session)?;
        let split_input: Vec<_> = input_buffer.trim().split(" ").collect();
        println!("{:?}", split_input);
        if let Some(&cmd) = split_input.first() {
            session = match session {
                TSession::Guest(s) => match cmd {
                    LOGIN => {
                        if split_input.len() != 3 {
                            println!("invalid login command. usage: login <username> <password>");
                            s.into()
                        } else {
                            match s.login(split_input.get(1).unwrap(), split_input.get(2).unwrap())
                            {
                                Login::Empty(empty) => {
                                    println!("login successful");
                                    empty.into()
                                }
                                Login::Error(error) => error.into(),
                            }
                        }
                    }
                    CLOSE => {
                        println!("goodbye!");
                        // NOTE breaks disable the "one-line handler" approach
                        break;
                    }
                    _ => {
                        println!("invalid command: {}", cmd);
                        s.into()
                    }
                },
                TSession::Empty(mut s) => match cmd {
                    SEARCH => {
                        if split_input.len() != 2 {
                            println!("invalid search command. usage: search <keyword>")
                        } else {
                            let trips = s.search_trip(split_input[1]);
                            for (i, trip) in trips.iter().enumerate() {
                                println!("{}: {:?}", i, trip);
                            }
                        }
                        s.into()
                    }
                    SELECT => {
                        if split_input.len() != 2 {
                            println!("invalid search command. usage: search <idx>");
                            s.into()
                        } else {
                            let idx = split_input[1].parse::<usize>().unwrap();
                            match s.add_trip(idx) {
                                Selection::Empty(s) => {
                                    println!("invalid index: {}", idx);
                                    s.into()
                                }
                                Selection::NonEmpty(s) => s.into(),
                            }
                        }
                    }
                    CLOSE => {
                        s.close();
                        println!("closing session!");
                        break;
                    }
                    _ => {
                        println!("invalid command: {}", cmd);
                        s.into()
                    }
                },
                TSession::NonEmpty(mut s) => match cmd {
                    SEARCH => {
                        if split_input.len() != 2 {
                            println!("invalid search command. usage: search <keyword>")
                        } else {
                            let trips = s.search_trip(split_input[1]);
                            for (i, trip) in trips.iter().enumerate() {
                                println!("{}: {:?}", i, trip);
                            }
                        }
                        s.into()
                    }
                    SELECT => {
                        if split_input.len() != 2 {
                            println!("invalid search command. usage: search <idx>");
                            s.into()
                        } else {
                            let idx = split_input[1].parse::<usize>().unwrap();
                            match s.add_trip(idx) {
                                Ok(()) => {}
                                Err(s) => println!("{}", s),
                            }
                            s.into()
                        }
                    }
                    BUY => {
                        if split_input.len() != 2 {
                            println!("invalid search command. usage: buy <token>");
                            s.into()
                        } else {
                            match s.buy(split_input[1]) {
                                Transaction::Empty(empty_sess) => empty_sess.into(),
                                Transaction::TError(error) => error.into(),
                            }
                        }
                    }
                    CLOSE => {
                        s.close();
                        println!("closing session!");
                        break;
                    }
                    _ => {
                        println!("invalid command: {}", cmd);
                        s.into()
                    }
                },
                TSession::TError(s) => match cmd {
                    RETRY => s.retry().into(),
                    CLOSE => {
                        s.close();
                        println!("closing session!");
                        break;
                    }
                    _ => {
                        println!("invalid command: {}", cmd);
                        s.into()
                    }
                },
                TSession::Error(s) => match cmd {
                    CLOSE => {
                        s.close();
                        println!("closing session!");
                        break;
                    }
                    _ => {
                        println!("invalid command: {}", cmd);
                        s.into()
                    }
                },
            };
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
