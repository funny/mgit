use home::home_dir;

pub const GIT_VERSION: &str = ">= 2.22.0";

#[allow(clippy::zombie_processes)]
pub fn open_in_file_explorer(path: &str) {
    if cfg!(target_os = "windows") {
        let path = path.replace('/', "\\");
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .expect("open in file explorer failed");
    } else {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .expect("open in file explorer failed");
    }
}

#[allow(clippy::zombie_processes)]
pub fn open_in_file_explorer_select(path: &str) {
    if cfg!(target_os = "windows") {
        let path = path.replace('/', "\\");
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
            .expect("open in file explorer failed");
    } else {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .expect("open in file explorer failed");
    }
}

#[allow(clippy::zombie_processes)]
pub fn open_repo_in_fork(repo_path: &str) {
    if cfg!(target_os = "windows") {
        let fork = format!(
            "{}/AppData/Local/Fork/Fork.exe",
            home_dir().unwrap().display()
        );
        let _ = std::process::Command::new(fork).arg(repo_path).spawn();
    } else {
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg("Fork")
            .arg(repo_path)
            .spawn();
    }
}

#[derive(Debug)]
pub struct GitVersionInfo {
    pub version_desc: String,
    pub version: Option<String>,
}

pub fn check_git_valid() -> Result<GitVersionInfo, String> {
    // make sure git is installed
    #[cfg(target_os = "windows")]
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        std::process::Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .arg("/C")
            .arg("git --version")
            .output()
            .expect("command failed to start")
    };

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("git")
        .arg("--version")
        .output()
        .expect("command failed to start");

    if !output.status.success() {
        return Err(String::from("git is not found!\n"));
    }

    // make sure git version = GIT_VERSION
    let version_desc = String::from_utf8(output.stdout).expect("mgit error");
    let re = regex::Regex::new(r"(?P<version>(\d+\.\d+\.\d+))").unwrap();
    if let Some(caps) = re.captures(&version_desc) {
        let version = caps["version"].to_string();
        let expect_version = semver::VersionReq::parse(GIT_VERSION).expect("semver error");
        let current_version = semver::Version::parse(&version).expect("semver error");

        let info = GitVersionInfo {
            version_desc: version_desc.trim().to_string(),
            version: Some(version.clone()),
        };

        match expect_version.matches(&current_version) {
            true => Ok(info),
            false => Err(format!(
                "git version {} is required, current version is {}\n",
                GIT_VERSION, version
            )),
        }
    } else {
        Err(String::from("failed to get git version\n"))
    }
}
