mod connect;
use connect::*;

pub fn run_interface() {
    run_connect().expect("Error running interface");
}
