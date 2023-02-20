// use std::fmt::format;

use core::fmt;

use crate::app_errors;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Enum created for helpimg identify which mode application is used
/// The inner char represents the type of request and activate and deactivate represent + and - respectively
pub enum Mode {
    Activate(char),
    Deactivate(char),
}

impl Mode {
    /// Given a mode object reference, creates the corresponding string
    pub fn to_mode_string(&self) -> String {
        match self {
            Mode::Activate(c) => format!("+{c}"),
            Mode::Deactivate(c) => format!("-{c}"),
        }
    }
}
#[derive(PartialEq, Eq, Debug, Clone)]
/// Enum with all implemented commands with their respective inputs
/// Invalid command is used for error codes
pub enum Command {
    /// PASS (value)
    Pass(String),
    /// NICK (value)
    Nick(String, i32),
    /// USER ( username, realname)
    User(String, String),
    /// OPER (user, password)
    Oper(String, String),
    /// QUIT (away message)
    Quit(Option<String>),
    /// PRIVMSG (nick, message)
    Privmsg(String, String),
    /// NOTICE (nick, text)
    Notice(String, String),
    /// JOIN (list of channels, list of keys)
    Join(Vec<String>, Vec<Option<String>>),
    /// PART (list of channels)
    Part(Vec<String>),
    /// NAMES (list of channels)
    Names(Vec<String>),
    /// LIST (optional list of channels)
    List(Vec<String>),
    /// INVITE (nickname, channel)
    Invite(String, String),
    /// WHO (user or mask, mode (optional))
    Who(String, Option<String>),
    /// WHOIS (server, nick)
    Whois(String),
    /// TOPIC (channel, new_topic)
    Topic(String, Option<String>),
    /// SERVER (servername, hopcount, info)
    Server(String, i32, String),
    /// SQUIT (servername, comment (optional))
    Squit(String, Option<String>),
    /// MODE (nick, mode, limit|user|ban mask (optional))
    Mode(String, Mode, Option<String>),
    /// KICK (channel, user, comment (optional))
    Kick(String, String, Option<String>),
    /// Away (message (optional))
    Away(Option<String>),
    /// CODE ERROR
    Invalid(((i32, &'static str), Vec<String>)),
}

impl Command {
    /// builds the command by delegating the parsing of the command to the right function
    /// and returns the command
    /// ## Example of implementation in pseudo-code
    ///
    /// if (params[0] == "PASS")
    ///     { parse_pass(params) }
    /// return Command::Pass
    ///
    pub fn build(params: Vec<String>) -> Command {
        // println!("params: {:?}", params);
        if params.is_empty() {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec![]));
        }
        return match params
            .get(0)
            .expect("Error: parameter length changed during parsing")
            .as_str()
        {
            "PASS" => Command::parse_pass(params),
            "NICK" => Command::parse_nick(params),
            "PRIVMSG" => Command::parse_privmsg(params),
            "USER" => Command::parse_user(params),
            "OPER" => Command::parse_oper(params),
            "QUIT" => Command::parse_quit(params),
            "NOTICE" => Command::parse_notice(params),
            "JOIN" => Command::parse_join(params),
            "PART" => Command::parse_part(params),
            "NAMES" => Command::parse_names(params),
            "LIST" => Command::parse_list(params),
            "INVITE" => Command::parse_invite(params),
            "WHO" => Command::parse_who(params),
            "WHOIS" => Command::parse_whois(params),
            "TOPIC" => Command::parse_topic(params),
            "SERVER" => Command::parse_server(params),
            "SQUIT" => Command::parse_squit(params),
            "MODE" => Command::parse_mode(params),
            "KICK" => Command::parse_kick(params),
            "AWAY" => Command::parse_away(params),
            other => Command::Invalid((app_errors::ERR_UNKNOWNCOMMAND, vec![other.to_string()])),
        };
    }

    /// Given separated parameters in a list returns the correct away command
    /// Should only be called from build
    fn parse_away(mut params: Vec<String>) -> Command {
        match params.len() {
            1 => Command::Away(None),
            _ => {
                params.remove(0);
                Command::Away(Some(params.join(" ")))
            }
        }
    }

    /// Given separated parameters in a list returns the correct kick command
    /// Should only be called from build
    fn parse_kick(mut params: Vec<String>) -> Command {
        let len = params.len();
        if len < 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["KICK".to_string()]));
        }
        if len == 3 {
            return Command::Kick(params.remove(1), params.remove(1), None);
        }
        params.remove(0);
        Command::Kick(params.remove(0), params.remove(0), Some(params.join(" ")))
    }

    /// Auxiliary function for helping identify the mode request
    fn build_mode(mut mode: String) -> Result<Mode, String> {
        if mode.len() == 2 {
            match mode.remove(0) {
                '+' => return Ok(Mode::Activate(mode.remove(0))),
                '-' => return Ok(Mode::Deactivate(mode.remove(0))),
                _ => return Err(mode),
            }
        }
        Err(mode)
    }

    /// Given separated parameters in a list returns the correct mode command
    /// Should only be called from build
    fn parse_mode(mut params: Vec<String>) -> Command {
        let len = params.len();
        match len {
            4 | 3 => {
                let channel_name = params.remove(1);
                match Command::build_mode(params.remove(1)) {
                    Ok(mode) => match len {
                        4 => Command::Mode(channel_name, mode, Some(params.remove(1))),
                        3 => Command::Mode(channel_name, mode, None),
                        _ => Command::Invalid((
                            app_errors::ERR_NEEDMOREPARAMS,
                            vec!["MODE".to_string()],
                        )),
                    },
                    Err(err) => Command::Invalid((app_errors::ERR_UNKNOWNMODE, vec![err])),
                }
            }
            _ => Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["MODE".to_string()])),
        }
    }

    /// Given separated parameters in a list returns the correct server command
    /// Should only be called from build
    fn parse_server(mut params: Vec<String>) -> Command {
        if params.len() != 4 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["SERVER".to_string()]));
        }
        match params[2].parse::<i32>() {
            Err(_) => {
                Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["SERVER".to_string()]))
            }
            Ok(value) => Command::Server(params.remove(1), value, params.remove(2)),
        }
    }

    /// Given separated parameters in a list returns the correct squit command
    /// Should only be called from build
    fn parse_squit(mut params: Vec<String>) -> Command {
        match params.len() {
            3 => Command::Squit(params.remove(1), Some(params.remove(1))),
            2 => Command::Squit(params.remove(1), None),
            _ => Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["SQUIT".to_string()])),
        }
    }

    /// Given separated parameters in a list returns the correct pass command
    /// Should only be called from build
    fn parse_pass(mut params: Vec<String>) -> Command {
        if params.len() < 2 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["PASS".to_string()]));
        }
        Command::Pass(params.remove(1))
    }

    /// Given separated parameters in a list returns the correct nick command
    /// Should only be called from build
    fn parse_nick(mut params: Vec<String>) -> Command {
        if params.len() < 2 || params.len() > 3 {
            return Command::Invalid((app_errors::ERR_NONICKNAMEGIVEN, vec![]));
        }
        let mut hopcount = 0;
        if params.len() == 3 {
            hopcount = params
                .remove(2)
                .parse()
                .expect("Error: parameter length changed during parsing nick");
        }
        Command::Nick(params.remove(1), hopcount)
    }

    /// Given separated parameters in a list returns the correct privmsg command
    /// Should only be called from build
    fn parse_privmsg(mut params: Vec<String>) -> Command {
        if params.len() < 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["PRIVMSG".to_string()]));
        }
        params.remove(0);
        Command::Privmsg(params.remove(0), params.join(" "))
    }

    /// Given separated parameters in a list returns the correct user command
    /// Should only be called from build
    fn parse_user(mut params: Vec<String>) -> Command {
        if params.len() < 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["USER".to_string()]));
        }
        params.remove(0);
        Command::User(params.remove(0), params.join(" "))
    }

    /// Given separated parameters in a list returns the correct oper command
    /// Should only be called from build
    fn parse_oper(mut params: Vec<String>) -> Command {
        if params.len() != 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["OPER".to_string()]));
        }
        params.remove(0);
        Command::Oper(params.remove(0), params.remove(0))
    }

    /// Given separated parameters in a list returns the correct quit command
    /// Should only be called from build
    fn parse_quit(mut params: Vec<String>) -> Command {
        if params.len() == 1 {
            return Command::Quit(None);
        }
        params.remove(0);
        Command::Quit(Some(params.join(" ")))
    }

    /// Given separated parameters in a list returns the correct notice command
    /// Should only be called from build
    fn parse_notice(mut params: Vec<String>) -> Command {
        if params.len() < 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["NOTICE".to_string()]));
        }
        params.remove(0);
        Command::Notice(params.remove(0), params.join(" "))
    }

    /// Given separated parameters in a list returns the correct join command
    /// Should only be called from build
    fn parse_join(params: Vec<String>) -> Command {
        if params.len() < 2 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["JOIN".to_string()]));
        }
        let channels: Vec<String> = params
            .get(1)
            .expect("Error: parameter length changed during parsing join")
            .split(',')
            .map(|value| value.to_string())
            .collect();

        let mut keys;
        if params.len() == 3 {
            keys = params
                .get(2)
                .expect("Error: parameter length changed during parsing join")
                .split(',')
                .map(|value| Some(value.to_string()))
                .collect();
        } else {
            keys = Vec::new();
        }

        while channels.len() > keys.len() {
            keys.push(None);
        }

        Command::Join(channels, keys)
    }

    /// Given separated parameters in a list returns the correct part command
    /// Should only be called from build
    fn parse_part(params: Vec<String>) -> Command {
        if params.len() != 2 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["PART".to_string()]));
        }
        let channels: Vec<String> = params
            .get(1)
            .expect("Error: parameter length changed during parsing part")
            .split(',')
            .map(|value| value.to_string())
            .collect();

        Command::Part(channels)
    }

    /// Given separated parameters in a list returns the correct names command
    /// Should only be called from build
    fn parse_names(params: Vec<String>) -> Command {
        if params.len() > 2 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["NAMES".to_string()]));
        }
        if params.len() == 1 {
            return Command::Names(Vec::new());
        }

        let names: Vec<String> = params
            .get(1)
            .expect("Error: parameter length changed during parsing names")
            .split(',')
            .map(|value| value.to_string())
            .collect();

        Command::Names(names)
    }

    /// Given separated parameters in a list returns the correct list command
    /// Should only be called from build
    fn parse_list(params: Vec<String>) -> Command {
        if params.len() >= 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["LIST".to_string()]));
        }
        if params.len() == 1 {
            Command::List(Vec::new())
        } else {
            let names: Vec<String> = params
                .get(1)
                .expect("Error: parameter length changed during parsing list")
                .split(',')
                .map(|value| value.to_string())
                .collect();

            Command::List(names)
        }
    }

    /// Given separated parameters in a list returns the correct invite command
    /// Should only be called from build
    fn parse_invite(mut params: Vec<String>) -> Command {
        if params.len() != 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["INVITE".to_string()]));
        }
        Command::Invite(params.remove(1), params.remove(1))
    }

    /// Given separated parameters in a list returns the correct topic command
    /// Should only be called from build
    fn parse_topic(mut params: Vec<String>) -> Command {
        if params.len() < 2 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["TOPIC".to_string()]));
        }
        params.remove(0);
        let channel = params.remove(0);
        if params.is_empty() {
            return Command::Topic(channel, None);
        }

        Command::Topic(channel, Some(params.join(" ")))
    }

    /// Given separated parameters in a list returns the correct who command
    /// Should only be called from build
    fn parse_who(mut params: Vec<String>) -> Command {
        if params.len() == 1 || params.len() > 3 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["WHO".to_string()]));
        }
        if params.len() == 2 {
            Command::Who(params.remove(1), None)
        } else {
            Command::Who(params.remove(1), Some(params.remove(1)))
        }
    }

    /// Given separated parameters in a list returns the correct whois command
    /// Should only be called from build
    fn parse_whois(mut params: Vec<String>) -> Command {
        if params.len() != 2 {
            return Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec!["WHOIS".to_string()]));
        }
        Command::Whois(params.remove(1))
    }

    /// Given the parameters of pass, return the string corresponding to it's irc command
    fn pass_to_string(pass: &String) -> String {
        format!("PASS {}", pass)
    }

    /// Given the parameters of nick, return the string corresponding to it's irc command
    fn nick_to_string(nick: &String, hopcount: i32) -> String {
        format!("NICK {} {}", nick, hopcount)
    }

    /// Given the parameters of privmsg, return the string corresponding to it's irc command
    fn privmsg_to_string(nick: &String, msg: &String) -> String {
        format!("PRIVMSG {} :{}", nick, msg)
    }

    /// Given the parameters of user, return the string corresponding to it's irc command
    fn user_to_string(username: &String, realname: &String) -> String {
        format!("USER {} {}", username, realname)
    }

    /// Given the parameters of oper, return the string corresponding to it's irc command
    fn oper_to_string(username: &String, pass: &String) -> String {
        format!("OPER {} {}", username, pass)
    }

    /// Given the parameters of quit, return the string corresponding to it's irc command
    fn quit_to_string(msg: &Option<String>) -> String {
        match msg {
            Some(x) => format!("QUIT :{}", x),
            None => "QUIT".to_string(),
        }
    }

    /// Given the parameters of notice, return the string corresponding to it's irc command
    fn notice_to_string(nick: &String, msg: &String) -> String {
        format!("NOTICE {} :{}", nick, msg)
    }

    /// Given the parameters of join, return the string corresponding to it's irc command
    fn join_to_string(channels: Vec<String>, keys: Vec<Option<String>>) -> String {
        let keys: Vec<String> = keys
            .iter()
            .filter(|&x| x.is_some())
            .map(|x| {
                x.clone()
                    .expect("Error: string object modified during join to string")
            })
            .collect();
        format!("JOIN {} {}", channels.join(","), keys.join(","))
    }

    /// Given the parameters of part, return the string corresponding to it's irc command
    fn part_to_string(channels: &[String]) -> String {
        format!("PART {}", channels.join(","))
    }

    /// Given the parameters of names, return the string corresponding to it's irc command
    fn names_to_string(channels: &[String]) -> String {
        format!("NAMES {}", channels.join(","))
    }

    /// Given the parameters of topic, return the string corresponding to it's irc command
    fn topic_to_string(channel: &String, new_topic: &Option<String>) -> String {
        match new_topic {
            Some(new_topic) => format!("TOPIC {} {}", channel, new_topic),
            None => format!("TOPIC {}", channel),
        }
    }

    /// Given the parameters of list, return the string corresponding to it's irc command
    fn list_to_string(channels: &[String]) -> String {
        format!("LIST {}", channels.join(","))
    }

    /// Given the parameters of invite, return the string corresponding to it's irc command
    fn invite_to_string(nick: &String, channel: &String) -> String {
        format!("INVITE {} {}", nick, channel)
    }

    /// Given the parameters of who, return the string corresponding to it's irc command
    fn who_to_string(nick: &String, mask: &Option<String>) -> String {
        match mask {
            Some(x) => format!("WHO {} {}", nick, x),
            None => format!("WHO {}", nick),
        }
    }

    /// Given the parameters of whois, return the string corresponding to it's irc command
    fn whois_to_string(nick: &String) -> String {
        format!("WHOIS {}", nick)
    }

    /// Given the parameters of server, return the string corresponding to it's irc command
    fn server_to_string(servername: &String, hopcount: &i32, message: &String) -> String {
        format!("SERVER {} {} :{}", servername, hopcount, message)
    }

    /// Given the parameters of squit, return the string corresponding to it's irc command
    fn squit_to_string(servername: &String, message: &Option<String>) -> String {
        match message {
            Some(x) => format!("SQUIT {} :{}", servername, x),
            None => format!("SQUIT {}", servername),
        }
    }

    /// Given the parameters of mode, return the string corresponding to it's irc command
    fn mode_to_string(channel_name: &String, mode: &Mode, extra: &Option<String>) -> String {
        let (sign, mode) = match mode {
            Mode::Activate(mode) => ('+', mode),
            Mode::Deactivate(mode) => ('-', mode),
        };
        match extra {
            Some(x) => format!("MODE {} {}{} {}", channel_name, sign, mode, x),
            None => format!("MODE {} {}{}", channel_name, sign, mode),
        }
    }

    /// Given the parameters of mode, return the string corresponding to it's irc command
    fn kick_to_string(channel: &String, user: &String, comment: &Option<String>) -> String {
        match comment {
            Some(x) => format!("KICK {} {} {}", channel, user, x),
            None => format!("KICK {} {}", channel, user),
        }
    }

    /// Given the parameters of away, return the string corresponding to it's irc command
    fn away_to_string(message: &Option<String>) -> String {
        match message {
            Some(x) => format!("AWAY {}", x),
            None => "AWAY".to_string(),
        }
    }
}
impl fmt::Display for Command {
    /// Implementation of the Display trait for command
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Pass(x) => write!(f, "{}", Command::pass_to_string(x)),
            Command::Nick(x, y) => write!(f, "{}", Command::nick_to_string(x, *y)),
            Command::Privmsg(x, y) => write!(f, "{}", Command::privmsg_to_string(x, y)),
            Command::User(x, y) => write!(f, "{}", Command::user_to_string(x, y)),
            Command::Oper(x, y) => write!(f, "{}", Command::oper_to_string(x, y)),
            Command::Quit(x) => write!(f, "{}", Command::quit_to_string(x)),
            Command::Notice(x, y) => write!(f, "{}", Command::notice_to_string(x, y)),
            Command::Join(x, y) => write!(f, "{}", Command::join_to_string(x.clone(), y.clone())),
            Command::Part(x) => write!(f, "{}", Command::part_to_string(x)),
            Command::Names(x) => write!(f, "{}", Command::names_to_string(x)),
            Command::List(x) => write!(f, "{}", Command::list_to_string(x)),
            Command::Invite(x, y) => write!(f, "{}", Command::invite_to_string(x, y)),
            Command::Who(x, y) => write!(f, "{}", Command::who_to_string(x, y)),
            Command::Whois(x) => write!(f, "{}", Command::whois_to_string(x)),
            Command::Topic(x, y) => write!(f, "{}", Command::topic_to_string(x, y)),
            Command::Server(x, y, z) => write!(f, "{}", Command::server_to_string(x, y, z)),
            Command::Squit(x, y) => write!(f, "{}", Command::squit_to_string(x, y)),
            Command::Mode(x, y, z) => write!(f, "{}", Command::mode_to_string(x, y, z)),
            Command::Kick(x, y, z) => write!(f, "{}", Command::kick_to_string(x, y, z)),
            Command::Away(x) => write!(f, "{}", Command::away_to_string(x)),
            Command::Invalid(_) => write!(f, ""),
        }
    }
}



#[cfg(test)]
mod command_test {
    // use crate::app_errors;
    // use crate::server::clients_info::ClientsInfo;
    use crate::server::client_s::message::command::Command;
    use crate::app_errors;

    #[test]
    fn empty_params_at_build_returns_invalid_command() {
        let command = Command::build(vec![]);
        assert_eq!(command, Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec![])));
    }

    #[test]
    fn build_params_for_pass_command_is_ok(){
        let command = Command::build(vec!["PASS".to_string(), "1234".to_string()]);
        assert_eq!(command, Command::Pass("1234".to_string()));
    }

    #[test]
    fn build_params_for_nick_command_is_ok(){
        let command = Command::build(vec!["NICK".to_string(), "1234".to_string(), "1".to_string()]);
        assert_eq!(command, Command::Nick("1234".to_string(), 1));
    }

    #[test]
    fn build_params_for_nick_returns_no_nicknamegiven(){
        let command = Command::build(vec!["NICK".to_string()]);
        assert_eq!(command, Command::Invalid((app_errors::ERR_NONICKNAMEGIVEN, vec![])));
    }

    #[test]
    fn build_privmsg_command_is_ok(){
        let command = Command::build(vec!["PRIVMSG".to_string(), "juan".to_string(), "hola".to_string()]);
        assert_eq!(command, Command::Privmsg("juan".to_string(), "hola".to_string()));
    }

    #[test]
    fn build_privmsg_command_returns_needmoreparams(){
        let command = Command::build(vec![]);
        assert_eq!(command, Command::Invalid((app_errors::ERR_NEEDMOREPARAMS, vec![])));
    }

    #[test]
    fn build_user_command_is_ok(){
        let command = Command::build(vec!["USER".to_string(), "juancho".to_string(), "guest".to_string(), "server_name".to_string(), "Juan".to_string()]);
        assert_eq!(command, Command::User("juancho".to_string(), "guest server_name Juan".to_string()));
    }

    #[test]
    fn build_join_command_is_ok(){
        let command = Command::build(vec!["JOIN".to_string(), "#rust #wiki #algo".to_string(), "clave".to_string()]);
        assert_eq!(command, Command::Join(vec!["#rust #wiki #algo".to_string()], vec![Some("clave".to_string())]));
    }    

    #[test]
    fn build_join_command_is_ok_2(){
        let command = Command::build(vec!["JOIN".to_string(), "#rust".to_string()]);
        assert_eq!(command, Command::Join(vec!["#rust".to_string()], vec![None]));
    }

    #[test]
    fn build_kick_command_is_ok(){
        let command = Command::build(vec!["KICK".to_string(), "#rust".to_string(), "juan".to_string(), "bye".to_string()]);
        assert_eq!(command, Command::Kick("#rust".to_string(), "juan".to_string(), Some("bye".to_string())));
    }
}
