use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtInput {
    pub thought: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    #[serde(default)]
    pub is_revision: Option<bool>,
    #[serde(default)]
    pub revises_thought: Option<u32>,
    #[serde(default)]
    pub branch_from_thought: Option<u32>,
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub needs_more_thoughts: Option<bool>,
    pub next_thought_needed: bool,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ThoughtOutput {
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    pub branches: Vec<String>,
    pub thought_history_length: usize,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoteKind {
    Assumption,
    Risk,
    Decision,
    Finding,
    Question,
    Blocker,
    Comparison,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NoteInput {
    pub kind: NoteKind,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NoteRecord {
    pub id: String,
    pub kind: NoteKind,
    pub text: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointInput {
    pub summary: String,
    #[serde(default)]
    pub open_questions: Vec<String>,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointRecord {
    pub id: String,
    pub summary: String,
    pub open_questions: Vec<String>,
    pub next_steps: Vec<String>,
    pub status: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateKind {
    Plan,
    Implementation,
    Verification,
    Release,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    Pass,
    Fail,
    Warn,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GateCheck {
    pub id: String,
    pub label: String,
    pub status: CheckStatus,
    #[serde(default)]
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GateInput {
    pub gate: GateKind,
    #[serde(default)]
    pub checks: Vec<GateCheck>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GateRecord {
    pub id: String,
    pub gate: GateKind,
    pub checks: Vec<GateCheck>,
    pub tags: Vec<String>,
    pub ready: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InspectView {
    Timeline,
    Notes,
    Checkpoints,
    Gates,
    Summary,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InspectInput {
    pub view: InspectView,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SummaryOutput {
    pub thought_history_length: usize,
    pub branches: Vec<String>,
    pub notes: usize,
    pub checkpoints: usize,
    pub gates: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReflectPurpose {
    Summarize,
    Critique,
    Compare,
    SuggestNextQuestions,
    Handoff,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReflectInput {
    pub purpose: ReflectPurpose,
    pub input: String,
}
