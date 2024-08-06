use core::panic;

use rocket::serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GuessFeedback {
    letters: Vec<LetterFeedback>,
    success: bool,
    remaining_attempts: u32,
}

impl LetterFeedback {
    fn new(letter: char, status: LetterStatus) -> LetterFeedback {
        LetterFeedback { letter, status }
    }
}

impl GuessFeedback {
    fn new(word: &str) -> GuessFeedback {
        let mut letters = Vec::with_capacity(5);

        for c in word.chars() {
            letters.push(LetterFeedback::new(c, LetterStatus::Unchecked));
        }

        GuessFeedback {
            letters,
            success: false,
            remaining_attempts: 5,
        }
    }

    pub fn process_guess(guess: &str, actual: &str) -> GuessFeedback {
        let mut feedback = GuessFeedback::new(guess);
        let mut good_count = 0;

        for (i, c) in actual.chars().enumerate() {
            // Check exact match
            if c == guess.chars().nth(i).unwrap() {
                feedback.letters.get_mut(i).unwrap().status = LetterStatus::Good;
                good_count += 1;
                continue;
            }

            // Check if right letter but invalid pos
            for (j, c2) in guess.chars().enumerate() {
                let letter = feedback.letters.get_mut(j).unwrap();

                if c == c2 && letter.status == LetterStatus::Unchecked {
                    letter.status = LetterStatus::WrongPos;
                    break;
                }
            }
        }

        for letter in feedback.letters.iter_mut() {
            if letter.status == LetterStatus::Unchecked {
                letter.status = LetterStatus::Invalid;
            }
        }

        if good_count == actual.len() {
            feedback.success = true;
        }

        feedback
    }

    pub fn to_simple_feedback(&self) -> String {
        let mut res = "".to_owned();

        for letter_feedback in &self.letters {
            let status_num = match letter_feedback.status {
                LetterStatus::Unchecked => panic!("This should never happen"),
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
pub struct LetterFeedback {
    letter: char,
    status: LetterStatus,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
enum LetterStatus {
    Unchecked,
    Invalid,
    WrongPos,
    Good,
}
