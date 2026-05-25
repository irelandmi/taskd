# Open Questions

Unresolved design decisions and things to figure out.

## Agent Architecture

- **Planner model selection**: Should the planner be a more capable model than the executors? Planning requires deeper reasoning about scope and decomposition, while execution is more mechanical. A capable planner + cheaper executors could be the right cost trade-off.

- **Task granularity**: How small should tasks be? Agents work best with well-defined, single-responsibility tasks. But over-decomposition creates coordination overhead. What's the right heuristic — "completable in one agent session"? "One file change"?

- **Executor workspace context**: How does an executor know *how* to do a task? It needs repo structure, available tools, coding conventions, test commands. Is this baked into a system prompt per project, or does the agent discover it?

- **Coordinator: agent or process?**: Should the coordinator be an LLM agent (expensive, flexible) or a deterministic process that checks for completed spikes and triggers re-planning? A hybrid — deterministic checks with LLM escalation — might be best.

- **Agent failure handling**: What happens when an agent crashes mid-task, produces bad output, or loops? Options: timeout + auto-unassign, human review on task completion, automated test gates before marking done.

## Schema & Data Model

- ~~**Dependency cycles**: If we add task dependencies, how do we prevent cycles?~~ **Resolved**: Cycle detection enforced at insert time via graph traversal. Circular dependencies are rejected with an error.

- **Cross-project dependencies**: Can a task in project A depend on a task in project B? Probably not for v1, but worth considering for monorepo scenarios.

- ~~**Blocked status UX**: If we add `blocked`, how does it appear on the kanban board?~~ **Resolved**: Blocked appears as a fifth column on the kanban board with a yellow/warn color scheme.

- ~~**Spike findings format**: Should spike output be structured or free-text?~~ **Resolved**: Task outputs are typed references (`file`, `commit`, `url`, `text`) with an optional label. Flexible enough for structured or free-form use.

## Operational

- **Concurrency limits**: How many executor agents can safely run against one SQLite database? WAL mode helps, but mutex contention in the server could be a bottleneck. At what point do we need to move to PostgreSQL?

- **Cost tracking**: Should the system track LLM token usage per task? Would help with cost attribution and optimising which tasks are worth automating.

- **Audit trail**: The activity log captures what happened, but not *why* an agent made a decision. Should agents be required to log their reasoning before acting?

- **Human override**: When a human reassigns or modifies a task on the board, how does a running executor know to stop? SSE back to the agent? Poll before each action?
