#[derive(Clone, Debug)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

pub trait ShellInteraction: Send + Sync {
    fn warn(&self, msg: &str);
    fn ask_ssh_trust(&self, fingerprint: &str) -> bool;
    fn ask_http_auth(&self) -> Option<Auth>;
}
