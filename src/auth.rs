use zeroize::Zeroize;

use crate::pam::{converse::Converse, session::PamSession, PamError};

struct PasswordConv {
    password: String,
}

impl Drop for PasswordConv {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}

impl Converse for PasswordConv {
    fn prompt_echo(&self, _msg: &str) -> Result<String, ()> {
        Err(())
    }

    fn prompt_blind(&self, _msg: &str) -> Result<String, ()> {
        Ok(self.password.clone())
    }

    fn info(&self, _msg: &str) -> Result<(), ()> {
        Ok(())
    }

    fn error(&self, _msg: &str) -> Result<(), ()> {
        Ok(())
    }
}

pub fn verify(service: &str, user: &str, password: &str) -> Result<(), PamError> {
    let conv = Box::pin(PasswordConv {
        password: password.to_string(),
    });
    let mut session = PamSession::start(service, user, conv)?;
    session.authenticate(pam_sys::PamFlag::NONE)?;
    session.acct_mgmt(pam_sys::PamFlag::NONE)?;
    session.end()?;
    Ok(())
}
