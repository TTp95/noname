use futures::prelude::*;
use irc::client::prelude::*;

#[derive(Debug)]
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
    //Oauth Token
    //let oauth_token = "oauth:yg7ujx082bts5m3oj23xsh1w0bldbm".to_string();

    //Config from hand write vars
    //let config = Config {
        //nickname: Some("ttp95".to_owned()),
        //server: Some("irc.chat.twitch.tv".to_owned()),
        //password: Some(oauth_token),
        //channels: vec!["#haroichi".to_owned()],
        //..Default::default()
    //};
    //let mut client = Client::from_config(config).await?;

    //Config from toml
    let mut client = Client::new("twitch.config.toml").await?;

    client.identify()?;

    let mut stream = client.stream()?;

    let mut twitch_msg = Vec::<TwitchMsg>::new();

    while let Some(message) = stream.next().await.transpose()? {
        println!("{:?}", message);

        match message.command {
            Command::PRIVMSG(_, ref msg) => {

                let user_nickname = if let Some(irc::client::prelude::Prefix::Nickname(nick, _, _)) = message.prefix {
                    nick
                }       else {
                    "noname".to_owned()
                };

                twitch_msg.push( TwitchMsg::new( user_nickname.to_owned(), msg.to_owned() ) );

            }
            _ => (),
        }

        //print Msg vector to screen
        println!("{:#?}", twitch_msg);

    }

    Ok(())
}
