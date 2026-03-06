pub mod logging;
pub mod structs;
use std::io::BufRead;

use anyhow::{Context, Result};
use serde::Serialize;
use tracing::{debug, warn};

use crate::structs::{
    initialize::{InitializeResult, ServerInfo},
    request::{
        ErrorCodes, NotificationMessage, RequestMessage, RequestMethod, ResponseError,
        ResponsePayload,
    },
    server_capabilities::ServerCapabilities,
};

#[derive(Debug)]
pub struct Header<Body> {
    pub content_len: usize,
    pub body: Body,
}

impl<Body: Serialize> Header<Body> {
    pub fn new(obj: Body) -> Result<Self> {
        let s = serde_json::to_string(&obj)?;
        Ok(Self {
            content_len: s.len(),
            body: obj,
        })
    }

    pub fn to_string(&self) -> Result<String> {
        let s = serde_json::to_string(&self.body)?;
        Ok(format!("Content-Length: {}\r\n\r\n{s}", self.content_len))
    }
}

pub fn parse_message(mut inp: impl BufRead) -> Result<String> {
    let mut buf = Vec::new();
    let len_def = inp
        .read_until(b'\n', &mut buf)
        .context("failed len def buf read")?;

    // No need to trim whitespace because the spec specifies `: `
    // `len_def - 2` because the end should have an extra `\r\n`
    // See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#headerPart
    let s = &buf["Content-Length: ".len()..len_def - 2];

    // `s` should only contain ascii numerals in the buffer.
    let size = str::from_utf8(s)
        .context("failed to convert numerals to utf8")?
        .parse::<usize>()
        .context("failed to parse utf8 to usize")?;
    debug!("Size: {size:?}");

    // Now we need to read the extra `\r\n`
    inp.consume(2);

    // Finally, we can read the json
    buf.clear();
    buf.reserve_exact(size);
    // SAFETY: Buffer has already reserved exactly `size` bytes
    unsafe { buf.set_len(size) };

    inp.read_exact(&mut buf)
        .context("failed to read exact to buf")?;

    // Doesn't reallocate, so this mostly just ensures that it contains valid utf8
    let s = String::from_utf8(buf).context("Failed to parse the buffer as utf8")?;

    Ok(s)
}

pub fn handle_notification(_notification: NotificationMessage) -> Result<()> {
    Ok(())
}

pub fn handle_request(request: RequestMessage) -> Result<String> {
    let response = match &request.method {
        RequestMethod::Initialize {
            method: _method,
            params: _params,
        } => request.reply_with(ResponsePayload::success(InitializeResult {
            capabilities: ServerCapabilities::default(),
            server_info: Some(ServerInfo::default()),
        })),

        RequestMethod::Unknown { method, params } => {
            let message = format!("Got unknown request method: `{method}` with params: {params:?}");
            warn!(message);
            request.reply_with(ResponsePayload::failure(ResponseError {
                code: ErrorCodes::MethodNotFound,
                message: message,
                data: params.to_owned(),
            }))
        }
    };

    let response = response
        .as_message()
        .context("failed to serialize message")?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        testing: bool,
    }

    #[test]
    fn test_serialize_header() {
        let payload = TestPayload { testing: true };
        let encoded = Header::<TestPayload>::new(payload).unwrap();
        let content = "Content-Length: 16\r\n\r\n{\"testing\":true}";
        assert_eq!(encoded.to_string().unwrap(), content);
    }
    #[test]
    fn test_parse_header() {
        let content: &[u8] = b"Content-Length: 16\r\n\r\n{\"testing\":true}";

        let body = parse_message(content).unwrap();

        let body = serde_json::from_str::<TestPayload>(&body).unwrap();

        assert_eq!(body, TestPayload { testing: true });
    }
}
