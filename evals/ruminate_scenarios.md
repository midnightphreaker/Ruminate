# Ruminate Evaluation Scenarios

Use these scenarios with MCP Inspector or an MCP client pointed at `/mcp`.

1. Planning capture
   - Call `ruminate` for three planning thoughts.
   - Add `ruminate_note` records for an assumption, risk, and decision.
   - Add a `plan` `ruminate_gate` with passing checks and evidence.
   - Expect `ruminate_inspect` summary to show three thoughts, three notes, and one ready gate.

2. Debugging trail
   - Record a hypothesis with `ruminate`.
   - Add finding and blocker notes.
   - Add a checkpoint with open questions and next steps.
   - Expect inspect views to preserve the session-local debug trail.

3. Decision comparison
   - Add two comparison notes and one decision note.
   - Inspect notes with `limit: 2`.
   - Expect only the two newest notes in insertion order.

4. Verification gate failure
   - Add a verification gate with a passing check that has no evidence.
   - Expect `ready: false`.
   - Add another verification gate with all passing checks and concrete evidence.
   - Expect `ready: true`.

5. Handoff
   - Create timeline entries, notes, a checkpoint, and a release gate.
   - Read `ruminate://session/checkpoints` and `ruminate://session/gates`.
   - Expect JSON resources to match the current session only.

6. Session isolation
   - Connect two separate Streamable HTTP clients.
   - Add notes in client A.
   - Inspect summary in client B.
   - Expect client B to show zero notes.

7. Prompt discovery
   - List prompts.
   - Expect `ruminate_plan`, `ruminate_debug`, `ruminate_review`, `ruminate_verify`, and `ruminate_handoff`.

8. Reflect disabled safety
   - Leave `RUMINATE_LLM_ENABLED` unset.
   - Call `ruminate_reflect`.
   - Expect `advisory: true` and `status: disabled`.

