#[macro_use]
extern crate rocket;

use chrono::{Datelike, Utc};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use serde::de::Error;
use std::fs;

#[derive(Responder)]
pub enum ApiResponse<T> {
    Ok(T),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 403)]
    Forbidden(String),
}

#[derive(Debug, Serialize)]
struct GuessFeedback {
    letters: Vec<LetterFeedback>,
    success: bool,
    remaining_attempts: u32,
}

impl GuessFeedback {
    fn compare_to_sol(guess: &str, actual: &str) -> GuessFeedback {
        let mut feeds = Vec::with_capacity(5);
        let mut good_count = 0;

        for (i, c) in guess.chars().enumerate() {
            let letter_status: LetterStatus;

            if actual.chars().nth(i).unwrap() == c {
                letter_status = LetterStatus::Good;
                good_count = good_count + 1;
            } else if actual.contains(c) {
                letter_status = LetterStatus::WrongPos;
            } else {
                letter_status = LetterStatus::Invalid;
            }

            feeds.push(LetterFeedback::new(c, letter_status));
        }

        let success = good_count == guess.len();
        let remaining_attempts = 5;

        GuessFeedback {
            letters: feeds,
            success,
            remaining_attempts,
        }
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

        res.push_str(if self.success { ":ye" } else { ":no" });
        res.push_str(&format!(":{}", self.remaining_attempts));

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

fn process_guess(guess: &str) -> Result<GuessFeedback, String> {
    if guess.len() == 5 {
        let all_words = get_words();
        if !all_words.contains(&guess.to_string()) {
            return Err("not a word!".to_string());
        }

        let actual = get_daily_word();
        Ok(GuessFeedback::compare_to_sol(&guess, &actual))
    } else {
        Err("only 5 letter words are allowed!".to_string())
    }
}

#[post("/guess", data = "<guess>")]
fn guess_word(guess: &str) -> ApiResponse<String> {
    match process_guess(&guess) {
        Ok(val) => ApiResponse::Ok(val.to_simple_feedback()),
        Err(message) => ApiResponse::BadRequest(message),
    }
}

#[post("/guess-json", data = "<guess>")]
fn guess_word_json(guess: &str) -> ApiResponse<Json<GuessFeedback>> {
    match process_guess(&guess) {
        Ok(val) => ApiResponse::Ok(Json(val)),
        Err(message) => ApiResponse::BadRequest(message),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, guess_word, guess_word_json, spoil])
}

fn get_daily_word() -> String {
    let words = get_words();
    let today: u64 = Utc::now().ordinal().into();
    let mut rng = ChaCha8Rng::seed_from_u64(today);

    words.choose(&mut rng).unwrap().to_string()
}

fn get_words() -> Vec<String> {
    let file_path = "./words.txt";
    read_lines(file_path)
}

fn read_lines(filename: &str) -> Vec<String> {
    let mut result = Vec::new();

    for line in fs::read_to_string(filename)
        .expect("Failed to read file")
        .lines()
    {
        result.push(line.to_string())
    }

    result
}
