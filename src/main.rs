#[macro_use]
extern crate rocket;

use chrono::{Datelike, Utc};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rocket::serde::json::Json;
use std::fs;

mod engine;

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

#[get("/")]
fn index() -> &'static str {
    "Hello wolrd"
}

#[get("/spoil")]
fn spoil() -> String {
    get_daily_word()
}

fn process_guess(guess: &str) -> Result<engine::GuessFeedback, String> {
    if guess.len() == 5 {
        let all_words = get_words();
        if !all_words.contains(&guess.to_string()) {
            return Err("not a word!".to_string());
        }

        let actual = get_daily_word();
        Ok(engine::GuessFeedback::process_guess(&guess, &actual))
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
fn guess_word_json(guess: &str) -> ApiResponse<Json<engine::GuessFeedback>> {
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
