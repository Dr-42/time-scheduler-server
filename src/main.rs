use std::{env, path::PathBuf};

use time_scheduler_server as app;

macro_rules! password_input {
    ($($fmt:expr),*) => {
        {
            use std::io::{self, Write};
            print!($($fmt),*);
            io::stdout().flush()?;
            let input = rpassword::read_password()?;
            input
        }
    };
}

macro_rules! input {
    ($($fmt:expr),*) => {
        {
            use std::io::{self, Write};
            print!($($fmt),*);
            io::stdout().flush().unwrap();
            let input = std::io::stdin().lines().next().ok_or("No input")??;
            input
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        println!("Usage: time-scheduler-server --data-dir <data_dir> --port <port> --help");
        return Ok(());
    }

    let mut data_dir = None;
    let mut port = None;

    let mut args_iter = args.iter().map(|s| s.as_str());

    while let Some(arg) = args_iter.next() {
        match arg {
            "--data-dir" => {
                let data_dir_str = args_iter.next().ok_or("Missing data directory")?;
                println!("Data directory: {}", data_dir_str);
                data_dir = Some(PathBuf::from(data_dir_str));
            }
            "--port" => {
                let port_str = args_iter.next().ok_or("Missing port")?;
                println!("Port: {}", port_str);
                port = Some(port_str.parse()?);
            }
            "--help" => {
                println!("Usage: time-scheduler-server");
                println!("Options:");
                println!("  --data-dir <data_dir>    Data directory");
                println!("  --port <port>            Port");
                println!("  --help                   Show this help message");
                println!("  --version                Show version");
                println!();
                println!("If no data directory or port is specified, the user will be prompted to enter it.");
                println!("If nor password.txt is found in the data directory, the user will be prompted to enter it.");
                return Ok(());
            }
            "--version" => {
                println!("{}: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    let data_dir = match data_dir {
        Some(data_dir) => data_dir,
        None => {
            println!("Data directory not specified");
            let data_dir_str = input!("Enter data directory: ");
            PathBuf::from(data_dir_str)
        }
    };

    let port = match port {
        Some(port) => port,
        None => {
            println!("Port not specified");
            let port_str = input!("Enter port: ");
            port_str.parse()?
        }
    };

    let password_path = data_dir.join("password.txt");
    let password = if password_path.exists() {
        std::fs::read_to_string(password_path)?.trim().to_string()
    } else {
        let password = password_input!("Enter password: ");
        let hash = sha256::digest(&password);
        std::fs::write(password_path, hash)?;
        password
    };

    app::run(port, password, data_dir).await
}
