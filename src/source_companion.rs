use crate::services::control_api::{ActionResponse, AgentsResponse, CONTROL_API_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub const SOURCE_COMPANION_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionRequest {
    pub protocol_version: u16,
    pub request_id: String,
    #[serde(default)]
    pub token: Option<String>,
    pub action: CompanionAction,
    #[serde(default)]
    pub pane_id: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub all_panes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompanionAction {
    Agents,
    Focus,
    Send,
    Extensions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionResponse {
    pub protocol_version: u16,
    pub request_id: String,
    pub ok: bool,
    #[serde(default)]
    pub agents: Option<AgentsResponse>,
    #[serde(default)]
    pub action: Option<ActionResponse>,
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

impl CompanionResponse {
    pub fn agents(request_id: String, agents: AgentsResponse) -> Self {
        Self {
            protocol_version: SOURCE_COMPANION_PROTOCOL_VERSION,
            request_id,
            ok: true,
            agents: Some(agents),
            action: None,
            extensions: None,
            error_code: None,
            error_message: None,
        }
    }

    pub fn action(request_id: String, action: ActionResponse) -> Self {
        Self {
            protocol_version: SOURCE_COMPANION_PROTOCOL_VERSION,
            request_id,
            ok: true,
            agents: None,
            action: Some(action),
            extensions: None,
            error_code: None,
            error_message: None,
        }
    }

    pub fn extensions(request_id: String, extensions: serde_json::Value) -> Self {
        Self {
            protocol_version: SOURCE_COMPANION_PROTOCOL_VERSION,
            request_id,
            ok: true,
            agents: None,
            action: None,
            extensions: Some(extensions),
            error_code: None,
            error_message: None,
        }
    }

    pub fn error(request_id: String, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            protocol_version: SOURCE_COMPANION_PROTOCOL_VERSION,
            request_id,
            ok: false,
            agents: None,
            action: None,
            extensions: None,
            error_code: Some(code.into()),
            error_message: Some(message.into()),
        }
    }
}

#[derive(Debug)]
pub enum CompanionClientError {
    Resolve(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Protocol(String),
    Remote { code: String, message: String },
}

impl std::fmt::Display for CompanionClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve(message) => write!(f, "{message}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::Protocol(message) => write!(f, "{message}"),
            Self::Remote { code, message } => write!(f, "{code}: {message}"),
        }
    }
}

impl std::error::Error for CompanionClientError {}

impl From<std::io::Error> for CompanionClientError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for CompanionClientError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone)]
pub struct CompanionClient {
    endpoint: String,
    timeout: Duration,
    token: Option<String>,
}

impl CompanionClient {
    pub fn new(endpoint: impl Into<String>, timeout_ms: u64, token: Option<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout: Duration::from_millis(timeout_ms.max(1)),
            token,
        }
    }

    pub fn request(
        &self,
        action: CompanionAction,
        pane_id: Option<String>,
        text: Option<String>,
        all_panes: bool,
    ) -> Result<CompanionResponse, CompanionClientError> {
        let request_id = format!(
            "req-{}-{}",
            std::process::id(),
            crate::sources::unix_ms_now()
        );
        let request = CompanionRequest {
            protocol_version: SOURCE_COMPANION_PROTOCOL_VERSION,
            request_id: request_id.clone(),
            token: self.token.clone(),
            action,
            pane_id,
            text,
            all_panes,
        };
        let addr = self.resolve_endpoint()?;
        let mut stream = TcpStream::connect_timeout(&addr, self.timeout)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        serde_json::to_writer(&mut stream, &request)?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        let mut line = String::new();
        let mut reader = BufReader::new(stream);
        reader.read_line(&mut line)?;
        if line.trim().is_empty() {
            return Err(CompanionClientError::Protocol(
                "companion returned an empty response".to_string(),
            ));
        }
        let response: CompanionResponse = serde_json::from_str(&line)?;
        if response.protocol_version != SOURCE_COMPANION_PROTOCOL_VERSION {
            return Err(CompanionClientError::Protocol(format!(
                "unsupported companion protocol version {}",
                response.protocol_version
            )));
        }
        if response.request_id != request_id {
            return Err(CompanionClientError::Protocol(format!(
                "companion response request id mismatch: expected {request_id}, got {}",
                response.request_id
            )));
        }
        if !response.ok {
            return Err(CompanionClientError::Remote {
                code: response
                    .error_code
                    .clone()
                    .unwrap_or_else(|| "source.companion.remote-error".to_string()),
                message: response
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "companion request failed".to_string()),
            });
        }
        Ok(response)
    }

    fn resolve_endpoint(&self) -> Result<SocketAddr, CompanionClientError> {
        self.endpoint
            .to_socket_addrs()
            .map_err(|error| {
                CompanionClientError::Resolve(format!(
                    "failed to resolve companion endpoint {}: {error}",
                    self.endpoint
                ))
            })?
            .next()
            .ok_or_else(|| {
                CompanionClientError::Resolve(format!(
                    "companion endpoint {} resolved no addresses",
                    self.endpoint
                ))
            })
    }
}

pub fn validate_agents_response(response: AgentsResponse) -> Result<AgentsResponse, String> {
    if response.schema_version != CONTROL_API_SCHEMA_VERSION {
        return Err(format!(
            "companion returned unsupported agents schema version {}",
            response.schema_version
        ));
    }
    Ok(response)
}

pub fn validate_action_response(response: ActionResponse) -> Result<ActionResponse, String> {
    if response.schema_version != CONTROL_API_SCHEMA_VERSION {
        return Err(format!(
            "companion returned unsupported action schema version {}",
            response.schema_version
        ));
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::control_api::{AgentEntry, ControlInventorySummary};
    use std::net::TcpListener;
    use std::thread;

    fn response() -> AgentsResponse {
        AgentsResponse {
            schema_version: CONTROL_API_SCHEMA_VERSION,
            generated_at_unix_ms: 1,
            inventory: ControlInventorySummary {
                total_sessions: 1,
                total_windows: 1,
                total_panes: 1,
                visible_sessions: 1,
                visible_windows: 1,
                visible_panes: 1,
            },
            entries: vec![AgentEntry::test_entry("%1")],
            diagnostics: Vec::new(),
            sources: Vec::new(),
            source_diagnostics: Vec::new(),
            partial_failure_count: 0,
        }
    }

    #[test]
    fn companion_client_round_trips_json_line_protocol() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = listener.local_addr().unwrap().to_string();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let request: CompanionRequest = serde_json::from_str(&line).unwrap();
            let mut stream = stream;
            serde_json::to_writer(
                &mut stream,
                &CompanionResponse::agents(request.request_id, response()),
            )
            .unwrap();
            stream.write_all(b"\n").unwrap();
        });

        let client = CompanionClient::new(endpoint, 1_000, None);
        let result = client
            .request(CompanionAction::Agents, None, None, false)
            .unwrap();
        assert_eq!(result.agents.unwrap().entries[0].pane_id, "%1");
    }
}
