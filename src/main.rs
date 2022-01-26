use futures::prelude::*;
use irc::client::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
//use std::ascii::AsciiExt;

#[derive(Debug)]
pub struct GuessingGame {
    pub word: String,
    pub show: Vec<usize>,
    pub num: usize,
    pub old: usize,
}

impl GuessingGame {
    fn new(word: String) -> GuessingGame {
        let show = vec![0; word.len()];
        GuessingGame {
            word,
            show,
            num: 0 as usize,
            old: 0 as usize,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchMsg {
    pub nick: String,
    pub text: String,
}

impl TwitchMsg {
    fn new(nick: String, text: String) -> TwitchMsg {
        TwitchMsg { nick, text }
    }
}

fn print_guess(guess_word: &GuessingGame) {
    for (i, e) in guess_word.show.iter().enumerate() {
        if *e == 0 {
            print!("_")
        } else {
            print!("{:}", guess_word.word.as_bytes()[i] as char)
        }
    }

    println!(" ")
}

fn compare_guess(guess_word: &mut GuessingGame, msgtw: &TwitchMsg) {
    let len_guess = msgtw.text.len();
    let len_word = guess_word.word.len();

    let chars_guess = msgtw.text.as_bytes();
    let chars_word = guess_word.word.as_bytes();

    let len_maximum = if len_word > len_guess {
        len_guess
    } else {
        len_word
    };
    guess_word.old = guess_word.num;

    for (i, e) in guess_word.show.iter_mut().enumerate() {
        if i < len_maximum {
            if *e != 1 {
                if chars_guess[i] == chars_word[i] {
                    *e = 1;
                    guess_word.num += 1;
                }
            }
        } else {
            break;
        };
    }
}

fn check_space(guess_word: &mut GuessingGame) {
    let chars_word = guess_word.word.as_bytes();
    let chars_space = " ".as_bytes();

    for (i, e) in guess_word.show.iter_mut().enumerate() {
        if chars_word[i] == chars_space[0] {
            *e = 1;
        }
    }
}

fn add_score(score_book: & mut HashMap<String, usize>, msgtw: &TwitchMsg, points: usize) {

    match score_book.get(&msgtw.nick) {
        Some(_) => {
            *score_book.get_mut(&msgtw.nick.to_string()).unwrap() += 10*points;
        },
        None => {
            score_book.insert(msgtw.nick.to_string(), 10*points);
        },

    };

}

fn check_end(guess_word: &mut GuessingGame) -> bool {
    if guess_word.num == guess_word.word.len() {
        println!("The End...");
        true
    } else {
        false
    }
}

#[tokio::main]
async fn main() -> irc::error::Result<()> {

    //read scores from last game if exist
    let file = File::open("output/scores.json");
    let mut score_book : HashMap<String, usize> = if let Ok(mut score_file) = file  {
        let mut data = String::new();
        score_file.read_to_string(&mut data).unwrap();
        serde_json::from_str(&data).expect("JSON was not well-formatted")

    } else {
        HashMap::new()
    };

    // HashMap
    //let mut score_book : HashMap<String, usize> = HashMap::new();

    //Config from toml
    let mut client = Client::new("twitch.config.toml").await?;

    client.identify()?;

    let mut stream = client.stream()?;

    let twitch_command = "&".to_owned();

    let mut playgame = true;

    while playgame {

        //welcome message
        println!("Welcome to The GuessingGame!! \n");

        //read from keyboard the word
        let mut line = String::new();
        println!("Enter your word :");
        std::io::stdin().read_line(&mut line).unwrap();
        //delete the \n char <- I HATE YOU </3
        line.pop();

        //create GuessingGame var
        let mut guess_word = GuessingGame::new(line.to_uppercase());

        //show the spaces
        check_space(&mut guess_word);

        //debug print
        println!("{guess_word:?}");

        //print incognita
        println!("Send your guess:");
        print_guess(&guess_word);

        while let Some(message) = stream.next().await.transpose()? {
            //debug print
            //println!("{:?}", message);

            match message.command {
                Command::PRIVMSG(_, ref msg) => {
                    if msg.len() > twitch_command.len() {
                        if &msg[0..twitch_command.len()] == twitch_command {
                            let user_nickname =
                                if let Some(irc::client::prelude::Prefix::Nickname(nick, _, _)) =
                                    message.prefix
                                {
                                    nick
                                } else {
                                    "noname".to_owned()
                                };
                            //create a TwitchMsg
                            let msgtw = TwitchMsg::new(
                                user_nickname.to_owned(),
                                msg[twitch_command.len()..].to_owned().to_uppercase(),
                            );

                            //debug print
                            //println!("{:?}", msgtw);

                            //compare message with the twitch word
                            compare_guess(&mut guess_word, &msgtw);

                            //debug print
                            println!("{:?}", guess_word.num - guess_word.old);

                            if guess_word.num != guess_word.old {
                                println!("Send your guess:");
                                print_guess(&guess_word);
                                add_score(& mut score_book, &msgtw, guess_word.num - guess_word.old);
                            }

                            let iter = check_end(&mut guess_word);

                            if iter {
                                //print scores
                                println!("{score_book:#?}");

                                //ask for a new game
                                println!("Play again? (Y/N)");
                                let mut restart = String::new();
                                std::io::stdin().read_line(&mut restart).unwrap();
                                //delete the \n char <- I HATE YOU </3
                                restart.pop();

                                //debug print
                                //println!("{restart:?}");

                                if restart.to_uppercase() == "N" {
                                    playgame = false;
                                } else {
                                    ();
                                }

                                break;
                            }

                        }
                    }
                }

                _ => (),
            }
        }
    }

    //generate a JSON String
    let json = serde_json::to_string(&score_book);

    //manage json error
    match json {
        Ok(json_string) => {
            //format file name
            let file_name = format!("output/scores.json");
            //print json to screen
            println!("file: {file_name} -> {json_string:#?}");
            //save JSON file
            let mut json_file = File::create(Path::new(&file_name))?;
            json_file.write_all(&json_string.as_bytes())?;
        }

        _ => (),
    }

    Ok(())
}
