use futures::prelude::*;
use irc::client::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchMsg{
    pub nick: String,
    pub text: String
}

impl TwitchMsg {
   fn new(nick: String, text: String) -> TwitchMsg {
       TwitchMsg { nick, text }
   }
}

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    //Config from toml
    let mut client = Client::new("twitch.config.toml").await?;

    client.identify()?;

    let mut stream = client.stream()?;

    let mut twitch_msg = Vec::<TwitchMsg>::new();

    let mut counter: usize = 0;

    while let Some(message) = stream.next().await.transpose()? {
        println!("{:?}", message);

        match message.command {
            Command::PRIVMSG(_, ref msg) => {

                let user_nickname = if let Some(irc::client::prelude::Prefix::Nickname(nick, _, _)) = message.prefix {
                    nick
                }       else {
                    "noname".to_owned()
                };

                //create a TwitchMsg and push it to the Vec<TwitchMsg>
                twitch_msg.push( TwitchMsg::new( user_nickname.to_owned(), msg.to_owned() ) );
                //print Msg vector to screen
                println!("{:#?}", &twitch_msg);

                //generate a JSON String
                let json = serde_json::to_string(&TwitchMsg::new(user_nickname.to_owned(), msg.to_owned()));

                counter += 1;

                //manage json error
                match json {
                    Ok(json_string) => {
                        //format file name
                        let file_name = format!("output/{counter:0>7}.json");
                        //print json to screen
                        println!("file: {file_name} -> {json_string:#?}");
                        //save JSON file
                        let mut json_file = File::create(Path::new(&file_name))?;
                        json_file.write_all(&json_string.as_bytes())?;
                    }

                    _ => (),
                }

            }

            _ => (),
        }


    }

    Ok(())
}
