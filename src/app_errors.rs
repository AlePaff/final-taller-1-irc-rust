use std::error::Error;
use std::fmt;

#[allow(dead_code)]
pub const RPL_SUCCESS: (i32, &str) = (201, ": Success!");
#[allow(dead_code)]
pub const RPL_UNAWAY: (i32, &str) = (305, ": You are no longer marked as being away");
#[allow(dead_code)]
pub const RPL_NOWAWAY: (i32, &str) = (306, ": You have been marked as being away");
#[allow(dead_code)]
pub const RPL_TOPIC: (i32, &str) = (332, "{} : {}");
#[allow(dead_code)]
pub const RPL_INVITING: (i32, &str) = (341, "INVITED {} {}");
#[allow(dead_code)]
pub const RPL_YOUREOPER: (i32, &str) = (381, ":You are now an IRC operator");
#[allow(dead_code)]
pub const ERR_NOSUCHNICK: (i32, &str) = (401, "{}:No such nick/channel");
#[allow(dead_code)]
pub const ERR_NOSUCHCHANNEL: (i32, &str) = (403, "{} :No such channel");
#[allow(dead_code)]
pub const ERR_UNKNOWNCOMMAND: (i32, &str) = (421, "{} :Unknown command");
#[allow(dead_code)]
pub const RPL_YOUAREIN: (i32, &str) = (200, ":Succesfully Connected ");
#[allow(dead_code)]
pub const ERR_NONICKNAMEGIVEN: (i32, &str) = (431, ":No nickname given");
#[allow(dead_code)]
pub const ERR_NICKCOLLISION: (i32, &str) = (436, "{} :Nickname collision KILL");
#[allow(dead_code)]
pub const ERR_UNKNOWNMODE: (i32, &str) = (472, "{} :is unknown mode char to me");
#[allow(dead_code)]
pub const ERR_INVITEONLYCHAN: (i32, &str) = (473, "{} :Cannot join channel (+i)");
#[allow(dead_code)]
pub const ERR_SERVERCOLLISION: (i32, &str) = (499, "{} :Servername collision KILL");
#[allow(dead_code)]
pub const ERR_NOTONCHANNEL: (i32, &str) = (442, "{} :You're not on that channel");
#[allow(dead_code)]
pub const ERR_USERONCHANNEL: (i32, &str) = (443, "{} {}:is already on channel");
#[allow(dead_code)]
pub const ERR_NOLOGIN: (i32, &str) = (444, " :User not logged in");
#[allow(dead_code)]
pub const ERR_NEEDMOREPARAMS: (i32, &str) = (461, "{} :Not enough parameters");
#[allow(dead_code)]
pub const ERR_ALREADYREGISTRED: (i32, &str) = (462, ":You may not reregister");
#[allow(dead_code)]
pub const ERR_PASSWDMISMATCH: (i32, &str) = (464, ":Password incorrect");
#[allow(dead_code)]
pub const ERR_CHANNELISFULL: (i32, &str) = (471, "{} :Cannot join channel (+l)");
#[allow(dead_code)]
pub const ERR_NOPRIVILEGES: (i32, &str) = (481, ":Permission Denied- You're not an IRC operator");
#[allow(dead_code)]
pub const ERR_UNEXPECTED: (i32, &str) = (100, "Unexpected error."); //para los que no estan implementados aun
#[allow(dead_code)]
pub const ERR_NOCHANPRIVILEGES: (i32, &str) =
    (498, ":Permission Denied- You're not a channel operator");
#[allow(dead_code)]
pub const ERR_UNTRUSTEDSERVER: (i32, &str) = (499, "{} :Untrusted server.");
#[allow(dead_code)]
pub const ERR_NOSUCHSERVER: (i32, &str) = (403, "{} :No such server");

//ir agregando a medida que se necesitan..
/* const ERR_NOSUCHNICK: i32 = 401;
pub const ERR_NOSUCHSERVER: i32 = 402;
pub const ERR_CANNOTSENDTOCHAN: i32 = 404;
pub const ERR_TOOMANYCHANNELS: i32 = 405;
pub const ERR_WASNOSUCHNICK: i32 = 406;
pub const ERR_TOOMANYTARGETS: i32 = 407;
pub const ERR_NOORIGIN: i32 = 409;
pub const ERR_NORECIPIENT: i32 = 411;
pub const ERR_NOTEXTTOSEND: i32 = 412;
pub const ERR_NOTOPLEVEL: i32 = 413;
pub const ERR_WILDTOPLEVEL: i32 = 414;
pub const ERR_NOMOTD: i32 = 422; */

/*
pub struct Reply {
    code: (i32, &'static str),
    params: Vec<String>,
}

impl Reply {
    pub fn build(code: (i32, &'static str), params: Vec<String>) -> Reply {
        Reply { code, params }
    }

    pub fn to_string(self) -> String {
        let mut res = self.code.1.clone();
        for param in self.params {
            res.replacen("{}", param.as_str(), 1);
        }
        return res.to_string();
    }
}
*/

#[derive(Debug)]
pub struct ApplicationError(pub String);

/// Implement the `Display` trait to format the error message
/// according to the RFC numeric error codes.
impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for ApplicationError {}
