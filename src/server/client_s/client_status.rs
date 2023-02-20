#[derive(PartialEq, Eq, Clone)]
/// Enum representing the different status a client can have.
/// Unregistered must only refer to the state before the client inputs
/// the correct PASS, NICK and USER combination.
pub enum ClientStatus {
    Unregistered,
    Registered,
    Oper,
}
