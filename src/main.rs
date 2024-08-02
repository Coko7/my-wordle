#[macro_use]
extern crate rocket;

use chrono::{Datelike, Utc};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use std::fs;

#[derive(Debug, Serialize)]
struct GuessWordFeedback {
    letters: Vec<LetterFeedback>,
}

impl GuessWordFeedback {
    fn compare_to_sol(guess: &str, actual: &str) -> GuessWordFeedback {
        let mut feeds = Vec::with_capacity(5);
        for (i, c) in guess.chars().enumerate() {
            let letter_status: LetterStatus;

            if actual.chars().nth(i).unwrap() == c {
                letter_status = LetterStatus::Good;
            } else if actual.contains(c) {
                letter_status = LetterStatus::WrongPos;
            } else {
                letter_status = LetterStatus::Invalid;
            }

            feeds.push(LetterFeedback::new(c, letter_status));
        }

        GuessWordFeedback { letters: feeds }
    }

    fn to_simple_feedback(&self) -> String {
        let mut res = "".to_owned();

        for letter_feedback in &self.letters {
            let status_num = match letter_feedback.status {
                LetterStatus::Invalid => "0",
                LetterStatus::WrongPos => "1",
                LetterStatus::Good => "2",
            };

            res.push_str(status_num);
        }

        res
    }
}

#[derive(Debug, Serialize)]
struct LetterFeedback {
    letter: char,
    status: LetterStatus,
}

impl LetterFeedback {
    fn new(letter: char, status: LetterStatus) -> LetterFeedback {
        LetterFeedback { letter, status }
    }
}

#[derive(Debug, Serialize)]
enum LetterStatus {
    Good,
    WrongPos,
    Invalid,
}

#[get("/")]
fn index() -> &'static str {
    "Hello wolrd"
}

#[get("/spoil")]
fn spoil() -> String {
    get_daily_word()
}

#[post("/guess", data = "<guess>")]
fn guess_word(guess: &str) -> Result<String, Status> {
    if guess.len() == 5 {
        let actual = get_daily_word();
        Ok(GuessWordFeedback::compare_to_sol(&guess, &actual).to_simple_feedback())
    } else {
        Err(Status::BadRequest)
    }
}

#[post("/guess-json", data = "<guess>")]
fn guess_word_json(guess: &str) -> Result<Json<GuessWordFeedback>, Status> {
    if guess.len() == 5 {
        let actual = get_daily_word();
        Ok(Json(GuessWordFeedback::compare_to_sol(&guess, &actual)))
    } else {
        Err(Status::BadRequest)
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, guess_word, guess_word_json, spoil])
}

fn get_daily_word() -> String {
    let file_path = "./words.txt";
    let contents = fs::read_to_string(file_path).expect("Should have been able to read file");

    let words: Vec<&str> = contents.split(',').collect();
    let today: u64 = Utc::now().ordinal().into();
    let mut rng = ChaCha8Rng::seed_from_u64(today);

    words.choose(&mut rng).unwrap().to_string()
}
