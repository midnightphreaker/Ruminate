use std::collections::BTreeMap;

use serde::Serialize;
use uuid::Uuid;

use crate::models::{
    CheckStatus, CheckpointInput, CheckpointRecord, GateInput, GateRecord, InspectView, NoteInput,
    NoteRecord, SummaryOutput, ThoughtInput, ThoughtOutput,
};

#[derive(Debug, Default)]
pub struct RuminateState {
    timeline: Vec<ThoughtInput>,
    branches: BTreeMap<String, Vec<ThoughtInput>>,
    notes: Vec<NoteRecord>,
    checkpoints: Vec<CheckpointRecord>,
    gates: Vec<GateRecord>,
}

impl RuminateState {
    pub fn process_thought(&mut self, mut input: ThoughtInput) -> ThoughtOutput {
        if input.thought_number > input.total_thoughts {
            input.total_thoughts = input.thought_number;
        }

        if let (Some(_), Some(branch_id)) = (input.branch_from_thought, input.branch_id.clone()) {
            self.branches
                .entry(branch_id)
                .or_default()
                .push(input.clone());
        }

        let output = ThoughtOutput {
            thought_number: input.thought_number,
            total_thoughts: input.total_thoughts,
            next_thought_needed: input.next_thought_needed,
            branches: self.branches.keys().cloned().collect(),
            thought_history_length: self.timeline.len() + 1,
        };
        self.timeline.push(input);
        output
    }

    pub fn add_note(&mut self, input: NoteInput) -> NoteRecord {
        let record = NoteRecord {
            id: Uuid::new_v4().to_string(),
            kind: input.kind,
            text: input.text,
            tags: input.tags,
        };
        self.notes.push(record.clone());
        record
    }

    pub fn add_checkpoint(&mut self, input: CheckpointInput) -> CheckpointRecord {
        let record = CheckpointRecord {
            id: Uuid::new_v4().to_string(),
            summary: input.summary,
            open_questions: input.open_questions,
            next_steps: input.next_steps,
            status: input.status,
            tags: input.tags,
        };
        self.checkpoints.push(record.clone());
        record
    }

    pub fn add_gate(&mut self, input: GateInput) -> GateRecord {
        let ready = !input.checks.is_empty()
            && input.checks.iter().all(|check| {
                check.status == CheckStatus::Pass && has_evidence(check.evidence.as_deref())
            });
        let record = GateRecord {
            id: Uuid::new_v4().to_string(),
            gate: input.gate,
            checks: input.checks,
            tags: input.tags,
            ready,
        };
        self.gates.push(record.clone());
        record
    }

    pub fn inspect(&self, view: InspectView, limit: Option<usize>) -> serde_json::Value {
        match view {
            InspectView::Timeline => limited_value(&self.timeline, limit),
            InspectView::Notes => limited_value(&self.notes, limit),
            InspectView::Checkpoints => limited_value(&self.checkpoints, limit),
            InspectView::Gates => limited_value(&self.gates, limit),
            InspectView::Summary => {
                serde_json::to_value(self.summary()).expect("summary serializes")
            }
        }
    }

    pub fn summary(&self) -> SummaryOutput {
        SummaryOutput {
            thought_history_length: self.timeline.len(),
            branches: self.branches.keys().cloned().collect(),
            notes: self.notes.len(),
            checkpoints: self.checkpoints.len(),
            gates: self.gates.len(),
        }
    }

    pub fn timeline_resource(&self) -> String {
        serde_json::to_string_pretty(&self.timeline).expect("timeline serializes")
    }

    pub fn notes_resource(&self) -> String {
        serde_json::to_string_pretty(&self.notes).expect("notes serializes")
    }

    pub fn checkpoints_resource(&self) -> String {
        serde_json::to_string_pretty(&self.checkpoints).expect("checkpoints serializes")
    }

    pub fn gates_resource(&self) -> String {
        serde_json::to_string_pretty(&self.gates).expect("gates serializes")
    }
}

fn has_evidence(evidence: Option<&str>) -> bool {
    evidence.is_some_and(|value| !value.trim().is_empty())
}

fn limited_value<T>(items: &[T], limit: Option<usize>) -> serde_json::Value
where
    T: Clone + Serialize,
{
    let limit = limit.unwrap_or(items.len()).min(items.len());
    let start = items.len().saturating_sub(limit);
    serde_json::to_value(&items[start..]).expect("records serialize")
}

#[cfg(test)]
mod tests {
    use crate::models::{GateCheck, GateKind, NoteKind};

    use super::*;

    fn thought(text: &str) -> ThoughtInput {
        ThoughtInput {
            thought: text.to_string(),
            thought_number: 1,
            total_thoughts: 3,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            next_thought_needed: true,
            mode: None,
            tags: Vec::new(),
            status: None,
            confidence: None,
        }
    }

    #[test]
    fn ruminate_tracks_history() {
        let mut state = RuminateState::default();
        assert_eq!(
            state.process_thought(thought("one")).thought_history_length,
            1
        );
        assert_eq!(
            state.process_thought(thought("two")).thought_history_length,
            2
        );
    }

    #[test]
    fn ruminate_tracks_branches() {
        let mut input = thought("branch");
        input.branch_from_thought = Some(1);
        input.branch_id = Some("branch-a".to_string());

        let mut state = RuminateState::default();
        let output = state.process_thought(input);
        assert_eq!(output.branches, vec!["branch-a"]);
    }

    #[test]
    fn records_workflow_items() {
        let mut state = RuminateState::default();
        let note = state.add_note(NoteInput {
            kind: NoteKind::Decision,
            text: "Use Rust".to_string(),
            tags: vec!["migration".to_string()],
        });
        assert_eq!(note.kind, NoteKind::Decision);

        let checkpoint = state.add_checkpoint(CheckpointInput {
            summary: "Parity done".to_string(),
            open_questions: vec!["Docker".to_string()],
            next_steps: vec!["Build image".to_string()],
            status: Some("partial".to_string()),
            tags: vec![],
        });
        assert_eq!(checkpoint.summary, "Parity done");
    }

    #[test]
    fn gate_missing_evidence_is_not_ready() {
        let mut state = RuminateState::default();
        let gate = state.add_gate(GateInput {
            gate: GateKind::Verification,
            checks: vec![GateCheck {
                id: "tests".to_string(),
                label: "Tests pass".to_string(),
                status: CheckStatus::Pass,
                evidence: None,
            }],
            tags: vec![],
        });
        assert!(!gate.ready);
    }

    #[test]
    fn gate_with_passing_evidence_is_ready() {
        let mut state = RuminateState::default();
        let gate = state.add_gate(GateInput {
            gate: GateKind::Verification,
            checks: vec![GateCheck {
                id: "tests".to_string(),
                label: "Tests pass".to_string(),
                status: CheckStatus::Pass,
                evidence: Some("cargo test --workspace passed".to_string()),
            }],
            tags: vec![],
        });
        assert!(gate.ready);
    }

    #[test]
    fn inspect_views_return_session_data() {
        let mut state = RuminateState::default();
        state.process_thought(thought("one"));
        state.add_note(NoteInput {
            kind: NoteKind::Risk,
            text: "Risk".to_string(),
            tags: vec![],
        });
        state.add_checkpoint(CheckpointInput {
            summary: "Checkpoint".to_string(),
            open_questions: vec![],
            next_steps: vec![],
            status: None,
            tags: vec![],
        });
        state.add_gate(GateInput {
            gate: GateKind::Plan,
            checks: vec![],
            tags: vec![],
        });

        assert_eq!(
            state
                .inspect(InspectView::Timeline, None)
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            state
                .inspect(InspectView::Notes, None)
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            state
                .inspect(InspectView::Checkpoints, None)
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            state
                .inspect(InspectView::Gates, None)
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(state.inspect(InspectView::Summary, None)["notes"], 1);
    }
}
