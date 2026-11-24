# Agent protocol

How an agent runtime reports to the nexios shell. The panel never runs
agents: it renders what a runtime writes and records the human's decisions
for that runtime to read. The contract is a directory of append-only
JSON-lines files under `$XDG_RUNTIME_DIR/nexios-agent/`:

```
$XDG_RUNTIME_DIR/nexios-agent/
  tasks/<id>.jsonl       events written by the runtime (append-only)
  decisions/<id>.jsonl   approvals and denials written by the panel
```

## Honesty rules

- The directory's existence IS the "a runtime is connected" signal. The
  panel never creates it; absence is shown honestly as "no runtime".
- A decision recorded by the panel is a message to the runtime, not an
  enforcement point. Whether the runtime honours it, and what isolation
  the agent runs under, is the runtime's business — the UI must not imply
  otherwise.
- `environment` and `agent` render verbatim. The panel never infers a
  sandbox that was not declared.
- `<id>` comes from the task file name and lands in paths: keep it boring
  (letters, digits, `-`, `_`).

## Task events (`tasks/<id>.jsonl`)

One JSON object per line. `ts` is unix seconds. Unterminated trailing
lines are tolerated (treated as a write in progress).

| `type`               | Fields | Effect |
|----------------------|--------|--------|
| `started`            | `name`, `objective`, `agent`, `interactive`, `environment` | Task appears, status running. |
| `milestone`          | `text` | Appended to the task's timeline; unblocks a `blocked` task. |
| `status`             | `state`: `working` \| `idle` | Flips running/idle; never overrides a question or a terminal state. |
| `approval_request`   | `request_id`, `action`, `resource`, `reason`, `reversible`, `scopes` | Task waits; the island shows the approval pill and card. |
| `question`           | `request_id`, `text`, `choices[]` | Task waits; the card offers the choices or free text. |
| `approval_resolved` / `question_resolved` | `request_id` | Clears the matching pending ask; task returns to running. |
| `blocked`            | `text` | Task shows blocked with the detail until the next milestone. |
| `finished`           | `summary` | Terminal; the task leaves the list. |
| `failed`             | `error` | Terminal; the task leaves the list. |

`approval_request.scopes` are strings like `"once"` and `"session"`;
unknown scope names are ignored, and an empty list defaults to once.
`reversible` is shown as reported ("can be undone" is the runtime's
claim, not the panel's).

## Decisions (`decisions/<id>.jsonl`)

The panel appends one line per human decision:

```json
{"decision":"approve","request_id":"req-1","scope":"once","source":"nexios-panel","ts":1795489091}
```

`decision` is `approve` or `deny` (for `question`, the line carries the
chosen or typed `answer` instead). `scope` echoes the scope the human
picked. A decision the runtime has not yet acknowledged (no
`*_resolved` for that `request_id`) renders as "Sent. Waiting for the
agent."

## Demo runtime

`shell/bin/nexios-agent-demo` walks the whole loop: writes a task with
milestones and an approval request, waits for the human's decision in
`decisions/`, resolves it, and finishes. Run it beside a live panel to
see the island approval pill, the approval card, and the task overlay.
