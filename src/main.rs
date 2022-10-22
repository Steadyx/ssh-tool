use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

macro_rules! home_dir {
            ($($name:expr),*) => {
                $(
                    if let Ok(path) = env::var($name) {
                        return Ok(path);
                    }
                )*
            }
        }

struct Key {
    name: String,
    email: String,
    passphrase: String,
}

fn main() {
    let passphrase_str = "Do you want to set your passphrase? [y/N] ";

    fn home_dir() -> io::Result<String> {
        let windows = env::consts::OS == "windows";
        let unix = env::consts::OS == "linux" || env::consts::OS == "macos";

        if windows {
            home_dir!("USERPROFILE", "HOMEDRIVE", "HOMEPATH");
        } else if unix {
            home_dir!("HOME");
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "home directory not found",
        ))
    }

    fn get_input(prompt: &str) -> String {
        print!("\x1b[36m{}\x1b[0m", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    fn passphrase(input: &str) -> String {
        if input == "y" {
            get_input("Enter passphrase: ")
        } else {
            String::new()
        }
    }

    let key = Key {
        name: get_input("Enter your keys name: > "),
        email: get_input("What is your email? > "),
        passphrase: passphrase(&get_input(passphrase_str)),
    };

    let ssh_dir = Path::new(&home_dir().unwrap()).join(".ssh");

    if !ssh_dir.exists() {
        std::fs::create_dir(&ssh_dir).unwrap();
    }

    let pub_key = ssh_dir.join(format!("{}.pub", key.name));

    let priv_key = ssh_dir.join(key.name);

    Command::new("ssh-keygen")
        .arg("-t")
        .arg("rsa")
        .arg("-b")
        .arg("4096")
        .arg("-C")
        .arg(key.email)
        .arg("-f")
        .arg(&priv_key)
        .arg("-N")
        .arg(key.passphrase)
        .spawn()
        .expect("Failed to execute ssh-keygen")
        .wait()
        .expect("Failed to wait on ssh-keygen");

    let mut pub_key_file = File::open(pub_key).unwrap();
    let mut pub_key_string = String::new();
    pub_key_file.read_to_string(&mut pub_key_string).unwrap();

    Command::new("ssh-add")
        .arg(&priv_key)
        .spawn()
        .expect("Failed to execute ssh-add")
        .wait()
        .expect("Failed to wait on ssh-add");

    Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to execute pbcopy")
        .stdin
        .unwrap()
        .write_all(pub_key_string.as_bytes())
        .expect("Failed to write to stdin");

    println!(
        "\n\x1b[32m{}\x1b[0m",
        "Your public key has been copied to the clipboard! 🍻"
    );
}
