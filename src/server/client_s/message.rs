pub mod command;
use crate::app_errors;
use std::error::Error;

/// is the conversion of the messages received by the stream.
/// Contains a prefix (optional) and a command
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: command::Command,
}

impl Message {
    /// Parses a line given by the client and returns a Message struct with the command built
    /// Online reference for message format: rfc-editor.org/rfc/rfc1459#section-2.3.1
    pub fn build(line: String) -> Result<Message, Box<dyn Error>> {
        let prefix_indicator = ':';
        let mut prefix = None;

        let mut line = line.trim().to_string();

        //an empty line is not a valid message
        if line.is_empty() {
            return Ok(Message {
                prefix: None,
                command: command::Command::Invalid((app_errors::ERR_UNKNOWNCOMMAND, vec![])),
            });
        }

        // Check if message begins with a prefix
        let first_char = match line.chars().next() {
            None => {
                return Ok(Message {
                    prefix: None,
                    command: command::Command::Invalid((app_errors::ERR_UNKNOWNCOMMAND, vec![])),
                })
            }
            Some(x) => x,
        };
        if prefix_indicator == first_char {
            match line.split_once(' ') {
                Some((nick, rest)) => {
                    let mut start = nick.to_string();
                    start.remove(0);
                    prefix = Some(start);
                    line = rest.to_string();
                }
                // Only containts a nick therefore not valid message
                None => {
                    return Ok(Message {
                        prefix: None,
                        command: command::Command::Invalid((
                            app_errors::ERR_UNKNOWNCOMMAND,
                            vec![],
                        )),
                    });
                }
            }
        }
        let mut l_split: Vec<String>;

        // Contains a message
        // Messages must not be separated
        if line.starts_with(':') {
            // guaranteed to exist
            match line.split_once(':') {
                Some((start, end)) => {
                    l_split = start
                        .trim()
                        .split(' ')
                        .map(|value| value.to_string())
                        .collect();
                    l_split.push(end.to_string());
                }
                None => {
                    return Ok(Message {
                        prefix: None,
                        command: command::Command::Invalid((
                            app_errors::ERR_UNKNOWNCOMMAND,
                            vec![],
                        )),
                    });
                }
            };
        } else {
            l_split = line.split(' ').map(|value| value.to_string()).collect();
        }

        //build the command given the vector of parameters (strings)
        let command = command::Command::build(l_split);
        Ok(Message { prefix, command })
    }
}

impl ToString for Message {
    /// Implementation of the ToString trait for message object
    fn to_string(&self) -> String {
        match self.prefix.clone() {
            Some(x) => {
                format!("{}: {}", x, self.command)
            }
            None => self.command.to_string(),
        }
    }
}

#[cfg(test)]
mod message_test {
    use crate::app_errors;
    use crate::server::client_s::message::command::Command;
    use crate::server::client_s::message::Message;
    #[test]
    fn empty_buffer_fails() {
        let buffer = "\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Invalid((app_errors::ERR_UNKNOWNCOMMAND, vec![])),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected);
    }
    #[test]
    fn pass_message_builds() {
        let buffer = "PASS hola\n".to_string();
        match Message::build(buffer) {
            Ok(msg) => {
                assert_eq!(
                    Message {
                        prefix: None,
                        command: Command::Pass("hola".to_string())
                    },
                    msg
                );
            }
            _ => panic!(),
        }
    }

    #[test]
    fn pass_message_builds_with_prefix() {
        let buffer = ":WIZ PASS hola\n".to_string();
        match Message::build(buffer) {
            Ok(msg) => {
                assert_eq!(
                    Message {
                        prefix: Some("WIZ".to_string()),
                        command: Command::Pass("hola".to_string())
                    },
                    msg
                );
            }
            _ => panic!(),
        }
    }

    #[test]
    fn nick_message_builds() {
        let buffer = "NICK hola\n".to_string();
        match Message::build(buffer) {
            Ok(msg) => {
                assert_eq!(
                    Message {
                        prefix: None,
                        command: Command::Nick("hola".to_string(), 0)
                    },
                    msg
                );
            }
            _ => panic!(),
        }
    }

    #[test]
    fn user_message_builds() {
        let buffer = "USER guest tolmoon".to_string();
        let expected = Message {
            prefix: None,
            command: Command::User("guest".to_string(), "tolmoon".to_string()),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected);
    }

    #[test]
    fn oper_message_builds() {
        let buffer = "OPER foo bar\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Oper("foo".to_string(), "bar".to_string()),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn quit_message_builds() {
        let buffer = "QUIT :Gone to have lunch\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Quit(Some(":Gone to have lunch".to_string())),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn privmsg_message_builds() {
        let buffer = "PRIVMSG Wiz :Hello are you receiving this message ?\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Privmsg(
                "Wiz".to_string(),
                ":Hello are you receiving this message ?".to_string(),
            ),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn notice_message_builds() {
        let buffer = "NOTICE Wiz :Hello are you receiving this message ?\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Notice(
                "Wiz".to_string(),
                ":Hello are you receiving this message ?".to_string(),
            ),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn basic_join_message_builds() {
        let buffer = "JOIN &foo fubar\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Join(vec!["&foo".to_string()], vec![Some("fubar".to_string())]),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn composite_join_message_builds() {
        let buffer = "JOIN #foo,&bar fubar \n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Join(
                vec!["#foo".to_string(), "&bar".to_string()],
                vec![Some("fubar".to_string()), None],
            ),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn basic_part_message_builds() {
        let buffer = "PART #twilight_zone\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Part(vec!["#twilight_zone".to_string()]),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn composite_part_message_builds() {
        let buffer = "PART #oz-ops,&group5\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Part(vec!["#oz-ops".to_string(), "&group5".to_string()]),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn names_message_builds() {
        let buffer = "NAMES #twilight_zone,#42\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Names(vec!["#twilight_zone".to_string(), "#42".to_string()]),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn list_alone_message_builds() {
        let buffer = "LIST\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::List(vec![]),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn list_of_channels_message_builds() {
        let buffer = "LIST #twilight_zone,#42 \n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::List(vec!["#twilight_zone".to_string(), "#42".to_string()]),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn invite_message_builds() {
        let buffer = ":Angel INVITE Wiz #Dust\n".to_string();
        let expected = Message {
            prefix: Some("Angel".to_string()),
            command: Command::Invite("Wiz".to_string(), "#Dust".to_string()),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn who_message_builds() {
        let buffer = "WHO jto* o\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Who("jto*".to_string(), Some("o".to_string())),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }

    #[test]
    fn whois_message_builds() {
        let buffer = "WHOIS trillian\n".to_string();
        let expected = Message {
            prefix: None,
            command: Command::Whois("trillian".to_string()),
        };
        let actual = Message::build(buffer).expect("");
        assert_eq!(actual, expected)
    }
}
