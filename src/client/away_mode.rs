#[derive(Clone)]
pub struct AwayMode {
    away_msg: Option<String>,
}

impl AwayMode {
    pub fn try_switch_on(&mut self, line: String) -> bool {
        true
    }
    pub fn is_active(&mut self, line: String) -> Option<String> {
        None
    }
    pub fn switch_off(&mut self) {
        self.away_msg = None;
    }
}

