use std::ffi::CString;

use nix::{
    sys::wait::waitpid,
    unistd::{execve, fork, initgroups, setgid, setuid, ForkResult, User},
};
use pam_sys::PamFlag;

use crate::{
    pam::{converse::Converse, session::PamSession},
    scrambler::Scrambler,
};

struct SessionConv {
    password: String,
}

impl Drop for SessionConv {
    fn drop(&mut self) {
        self.password.scramble();
    }
}

impl Converse for SessionConv {
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

pub fn launch(
    service: &str,
    username: &str,
    password: &str,
    command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = User::from_name(username)?
        .ok_or_else(|| format!("user '{username}' not found"))?;

    let conv = Box::pin(SessionConv {
        password: password.to_string(),
    });

    let mut session = PamSession::start(service, username, conv)?;
    session.acct_mgmt(PamFlag::NONE)?;
    session.setcred(PamFlag::ESTABLISH_CRED)?;

    // Standard session env vars
    let _ = session.putenv("XDG_SEAT=seat0");
    let _ = session.putenv(&format!("USER={}", user.name));
    let _ = session.putenv(&format!("LOGNAME={}", user.name));
    let _ = session.putenv(&format!("HOME={}", user.dir.display()));
    let _ = session.putenv(&format!("SHELL={}", user.shell.display()));
    let _ = session.putenv(&format!("XDG_RUNTIME_DIR=/run/user/{}", user.uid));
    let path = std::env::var("PATH").unwrap_or_else(|_| "/run/current-system/sw/bin:/bin:/usr/bin".to_string());
    let _ = session.putenv(&format!("PATH={}", path));

    session.open_session(PamFlag::NONE)?;

    let pam_env = session.getenvlist()?;
    let env_vec = pam_env.to_vec();

    let cusername = CString::new(user.name.as_str())?;
    let csh = CString::new("/bin/sh")?;
    let cflag = CString::new("-c")?;
    let ccmd = CString::new(command)?;

    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            let _ = waitpid(child, None);
            let _ = session.close_session(PamFlag::NONE);
            let _ = session.setcred(PamFlag::DELETE_CRED);
            let _ = session.end();
            Ok(())
        }
        ForkResult::Child => {
            if let Err(err) = initgroups(&cusername, user.gid) {
                eprintln!("initgroups failed: {err}");
                std::process::exit(1);
            }
            if let Err(err) = setgid(user.gid) {
                eprintln!("setgid failed: {err}");
                std::process::exit(1);
            }
            if let Err(err) = setuid(user.uid) {
                eprintln!("setuid failed: {err}");
                std::process::exit(1);
            }

            unsafe {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM, 0, 0, 0);
            }

            let _ = std::env::set_current_dir(&user.dir);

            let _ = execve(&csh, &[&csh, &cflag, &ccmd], &env_vec);
            std::process::exit(1);
        }
    }
}
