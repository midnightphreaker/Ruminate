use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::Serialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    models::{CheckpointInput, GateInput, InspectInput, NoteInput, ReflectInput, ThoughtInput},
    reflect,
    state::RuminateState,
};

const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const JSON_MIME_TYPE: &str = "application/json";

#[derive(Clone)]
#[allow(dead_code)]
pub struct RuminateServer {
    state: Arc<Mutex<RuminateState>>,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

#[tool_router]
impl RuminateServer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RuminateState::default())),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    #[tool(description = "Record a reflective thought in the current MCP session timeline.")]
    pub async fn ruminate(
        &self,
        Parameters(input): Parameters<ThoughtInput>,
    ) -> Result<CallToolResult, McpError> {
        let output = self.state.lock().await.process_thought(input);
        Ok(json_result(output))
    }

    #[tool(
        description = "Record an assumption, risk, decision, finding, question, blocker, or comparison note."
    )]
    pub async fn ruminate_note(
        &self,
        Parameters(input): Parameters<NoteInput>,
    ) -> Result<CallToolResult, McpError> {
        let output = self.state.lock().await.add_note(input);
        Ok(json_result(output))
    }

    #[tool(
        description = "Record a workflow checkpoint with summary, open questions, next steps, status, and tags."
    )]
    pub async fn ruminate_checkpoint(
        &self,
        Parameters(input): Parameters<CheckpointInput>,
    ) -> Result<CallToolResult, McpError> {
        let output = self.state.lock().await.add_checkpoint(input);
        Ok(json_result(output))
    }

    #[tool(
        description = "Record a plan, implementation, verification, or release gate and calculate readiness."
    )]
    pub async fn ruminate_gate(
        &self,
        Parameters(input): Parameters<GateInput>,
    ) -> Result<CallToolResult, McpError> {
        let output = self.state.lock().await.add_gate(input);
        Ok(json_result(output))
    }

    #[tool(
        description = "Inspect timeline, notes, checkpoints, gates, or a compact session summary."
    )]
    pub async fn ruminate_inspect(
        &self,
        Parameters(input): Parameters<InspectInput>,
    ) -> Result<CallToolResult, McpError> {
        let state = self.state.lock().await;
        Ok(json_value_result(state.inspect(input.view, input.limit)))
    }

    #[tool(
        description = "Optionally ask a configured LLM for advisory reflection. Disabled by default and never called implicitly."
    )]
    pub async fn ruminate_reflect(
        &self,
        Parameters(input): Parameters<ReflectInput>,
    ) -> Result<CallToolResult, McpError> {
        Ok(json_value_result(reflect::reflect(input).await))
    }
}

impl Default for RuminateServer {
    fn default() -> Self {
        Self::new()
    }
}

#[prompt_router]
impl RuminateServer {
    #[prompt(name = "ruminate_plan")]
    pub async fn ruminate_plan(&self) -> Result<GetPromptResult, McpError> {
        Ok(prompt_text(
            "Use Ruminate to capture assumptions, risks, plan checkpoints, and the plan gate before implementation.",
        ))
    }

    #[prompt(name = "ruminate_debug")]
    pub async fn ruminate_debug(&self) -> Result<GetPromptResult, McpError> {
        Ok(prompt_text(
            "Use Ruminate to track hypotheses, findings, blockers, and verification evidence while debugging.",
        ))
    }

    #[prompt(name = "ruminate_review")]
    pub async fn ruminate_review(&self) -> Result<GetPromptResult, McpError> {
        Ok(prompt_text(
            "Use Ruminate to record review findings, risks, decisions, and follow-up questions.",
        ))
    }

    #[prompt(name = "ruminate_verify")]
    pub async fn ruminate_verify(&self) -> Result<GetPromptResult, McpError> {
        Ok(prompt_text(
            "Use Ruminate gates to record checks, statuses, and concrete evidence before claiming completion.",
        ))
    }

    #[prompt(name = "ruminate_handoff")]
    pub async fn ruminate_handoff(&self) -> Result<GetPromptResult, McpError> {
        Ok(prompt_text(
            "Use Ruminate inspect summary, checkpoints, open questions, and gates to prepare a concise handoff.",
        ))
    }
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for RuminateServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("ruminate", SERVER_VERSION).with_title("Ruminate"))
        .with_protocol_version(ProtocolVersion::V_2025_03_26)
        .with_instructions("A session-local reflective workflow MCP server.".to_string())
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                Resource::new("ruminate://session/timeline", "Timeline".to_string())
                    .with_mime_type(JSON_MIME_TYPE),
                Resource::new("ruminate://session/notes", "Notes".to_string())
                    .with_mime_type(JSON_MIME_TYPE),
                Resource::new("ruminate://session/checkpoints", "Checkpoints".to_string())
                    .with_mime_type(JSON_MIME_TYPE),
                Resource::new("ruminate://session/gates", "Gates".to_string())
                    .with_mime_type(JSON_MIME_TYPE),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let state = self.state.lock().await;
        let text = match request.uri.as_str() {
            "ruminate://session/timeline" => state.timeline_resource(),
            "ruminate://session/notes" => state.notes_resource(),
            "ruminate://session/checkpoints" => state.checkpoints_resource(),
            "ruminate://session/gates" => state.gates_resource(),
            _ => {
                return Err(McpError::resource_not_found(
                    "resource_not_found",
                    Some(json!({ "uri": request.uri })),
                ));
            }
        };

        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(text, request.uri).with_mime_type(JSON_MIME_TYPE),
        ]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: Vec::new(),
            next_cursor: None,
            meta: None,
        })
    }
}

fn json_result<T>(value: T) -> CallToolResult
where
    T: Serialize,
{
    let value = serde_json::to_value(value).expect("tool result serializes");
    json_value_result(value)
}

fn json_value_result(value: serde_json::Value) -> CallToolResult {
    let text = serde_json::to_string_pretty(&value).expect("tool result serializes");
    let mut result = CallToolResult::success(vec![ContentBlock::text(text)]);
    result.structured_content = Some(value);
    result
}

fn prompt_text(text: &str) -> GetPromptResult {
    GetPromptResult::new(vec![PromptMessage::new_text(Role::User, text.to_string())])
}

#[cfg(test)]
mod tests {
    use rmcp::handler::server::ServerHandler;

    use crate::models::GateKind;

    use super::*;

    #[tokio::test]
    async fn tools_mutate_current_server_state() {
        let server = RuminateServer::new();
        let first = server
            .ruminate(Parameters(ThoughtInput {
                thought: "one".to_string(),
                thought_number: 1,
                total_thoughts: 3,
                is_revision: None,
                revises_thought: None,
                branch_from_thought: None,
                branch_id: None,
                needs_more_thoughts: None,
                next_thought_needed: true,
                mode: None,
                tags: vec![],
                status: None,
                confidence: None,
            }))
            .await
            .unwrap();
        assert_eq!(first.structured_content.unwrap()["thoughtHistoryLength"], 1);

        let gate = server
            .ruminate_gate(Parameters(GateInput {
                gate: GateKind::Plan,
                checks: vec![],
                tags: vec![],
            }))
            .await
            .unwrap();
        assert_eq!(gate.structured_content.unwrap()["ready"], false);
    }

    #[test]
    fn exposes_expected_tools_and_prompts() {
        let tools = RuminateServer::tool_router();
        for name in [
            "ruminate",
            "ruminate_note",
            "ruminate_checkpoint",
            "ruminate_gate",
            "ruminate_inspect",
            "ruminate_reflect",
        ] {
            assert!(tools.has_route(name), "missing tool {name}");
        }

        let prompts = RuminateServer::prompt_router();
        for name in [
            "ruminate_plan",
            "ruminate_debug",
            "ruminate_review",
            "ruminate_verify",
            "ruminate_handoff",
        ] {
            assert!(prompts.has_route(name), "missing prompt {name}");
        }
    }

    #[test]
    fn server_declares_resources_prompts_and_tools() {
        let info = RuminateServer::new().get_info();
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.prompts.is_some());
        assert!(info.capabilities.resources.is_some());
        assert_eq!(info.server_info.name, "ruminate");
    }
}
