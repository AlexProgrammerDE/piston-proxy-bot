use std::{
    env,
    error::Error,
    io::{self, ErrorKind},
};

use piston_proxy_commands::global_commands;
use twilight_http::Client;
use twilight_model::id::Id;

fn load_development_environment() -> Result<(), Box<dyn Error>> {
    match dotenvy::from_filename(".dev.vars") {
        Ok(_) => Ok(()),
        Err(dotenvy::Error::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn required_environment_variable(name: &str) -> Result<String, io::Error> {
    env::var(name).map_err(|_| {
        io::Error::new(
            ErrorKind::NotFound,
            format!("the {name} environment variable is required"),
        )
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    load_development_environment()?;

    let token = required_environment_variable("DISCORD_TOKEN")?;
    let application_id = required_environment_variable("DISCORD_APPLICATION_ID")?.parse::<u64>()?;
    let commands = global_commands();
    let client = Client::new(token);

    let registered_commands = client
        .interaction(Id::new(application_id))
        .set_global_commands(&commands)
        .await?
        .models()
        .await?;

    println!(
        "Registered {} global Discord commands.",
        registered_commands.len()
    );

    Ok(())
}
