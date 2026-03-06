use std::io::{self, Write};

use sf_formula_lsp::{
    handle_notification, handle_request, logging::setup_logging_to_stderr_and_file, parse_message,
    structs::request::Message,
};

use anyhow::{Context, Result};
use tracing::{debug, error, info};

fn main() -> Result<()> {
    setup_logging_to_stderr_and_file("lsp.log")?;
    match real_main() {
        Err(e) => {
            error!("{e}");
            Err(e)
        }
        rest => rest,
    }
}

fn real_main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    info!("Starting Server");

    loop {
        let data = parse_message(stdin.lock())?;
        debug!("Successfully parsed message");

        let request = serde_json::from_str::<Message>(&data)?;

        let request = match request {
            Message::Request(request_message) => request_message,
            Message::Notification(notification_message) => {
                info!("Recieved Notification: {notification_message:?}");
                handle_notification(notification_message).context("handling notification")?;
                continue;
            }
        };

        let response = handle_request(request).context("handling request")?;
        debug!("response: {response:?}");

        stdout
            .write(response.as_bytes())
            .context("Failed to write response out")?;
        stdout.flush()?;
        info!("responded successfully");
    }
}
