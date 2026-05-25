# Agent Architecture

taskd is a lightweight project management tool (SQLite, Rust CLI/API, web UI with live SSE updates). The goal is to enable AI agents to plan, execute, and report on software projects — while humans monitor progress via the kanban board in real-time.

In the real world, a project flows from requirements → planning → execution → review. We want to mirror that with agents.

## Agent Roles

### 1. Planner

Reads a requirements document (PRD, spec, brief) and decomposes it into the project structure:

- Creates the **project**
- Identifies **epics** (feature areas / workstreams)
- Writes **stories** (user-facing deliverables) within each epic
- Creates **spikes** for unknowns that need research before execution
- Breaks stories into concrete **tasks** and **sub-tasks**
- Sets priorities and identifies dependencies between tasks

The planner's output is a fully populated taskd project that an execution team can pick up.

### 2. Executor(s)

Picks up tasks and does the work. An executor:

- Claims a task (status → `in_progress`, assignee → agent ID)
- Performs the work (write code, run tests, call APIs, etc.)
- Logs progress via task events (`taskd task log`)
- Marks complete (`taskd task done`) or flags as blocked
- Picks the next available task

Executors can be a **single agent** (sequential) or a **worker pool** (parallel). The pool model is more efficient but needs conflict resolution — see schema gaps below.

### 3. Reviewer / Coordinator

Watches the board and handles feedback loops:

- When a **spike** completes, reads its findings and creates follow-up tasks
- Detects **blocked** work and re-prioritises or reassigns
- Reviews completed work (code review, test verification)
- Escalates decisions to the human when needed (via task events / comments)
- Can close epics when all tasks are done

This role is optional for simple projects but essential for anything with unknowns (spikes) or multiple executors.

## Schema Support

### Task dependencies (implemented)

The `task_dependencies` table expresses ordering constraints between tasks. A task is **ready** when all its `depends_on` tasks have status `done`. Executors query for ready tasks only. Circular dependencies are rejected at insert time via graph traversal.

### Task outputs (implemented)

The `task_outputs` table stores lightweight references (file paths, commit SHAs, URLs, free text) produced by a task. This solves the spike output problem — a spike executor records its findings as outputs, and the coordinator can read them to create follow-up tasks.

### Blocked status (implemented)

Tasks support a `blocked` status. Agents set status to `blocked` and log a comment explaining why. The coordinator or human can unblock. Blocked tasks appear in a dedicated column on the kanban board.

### Remaining gap: task claiming atomicity

Two executors could both see a `todo` task and both set it to `in_progress`. Status update isn't atomic — it's a read-then-write race.

| Approach | How it works | Trade-off |
|----------|-------------|-----------|
| Optimistic locking | Add `version` column. UPDATE includes `WHERE version = ?`. If 0 rows affected, retry. | Simple, no extra tables. Retry logic needed. |
| Assignee as lock | Only unassigned + todo tasks are claimable. Assigning is the claim. | Intuitive but same atomicity issue. |
| Queue table | Separate `task_queue` with `claimed_by` and `claimed_at`. Stale claims reclaimable after timeout. | Most robust. Handles agent crashes. More complex. |

## The Execution Loop

```
                    ┌─────────────┐
                    │  PRD / Spec │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │   Planner   │
                    │  (1 agent)  │
                    └──────┬──────┘
                           │ creates project, epics, stories, spikes, tasks
                           │ sets dependencies
                    ┌──────▼──────┐
              ┌─────┤  Task Board │◄──── Human monitors (live SSE)
              │     └──────┬──────┘
              │            │
         ready tasks       │
              │     ┌──────▼──────┐
              ├────►│ Executor(s) │──── claims task, does work, logs progress
              │     └──────┬──────┘
              │            │
              │     ┌──────▼──────┐
              └─────┤ Coordinator │──── reviews, handles spike outputs,
                    │  (optional) │     re-plans, unblocks
                    └─────────────┘
```

## Agent-LLM Interface

The harness should be **LLM-agnostic** — any model that supports tool calling can be an executor/planner/coordinator. The integration surface is:

- **Tools**: Wrap taskd CLI commands (or HTTP API) as tool definitions. The agent calls `create_task`, `update_status`, `log_comment`, etc.
- **Context**: Feed the agent its assigned task details (title, description, parent context, spike findings) as system/user messages.
- **Loop**: The harness runs the agent in a loop — present task → agent acts → check completion → next task.

This means the harness is a thin orchestrator, not tied to LangGraph or any specific framework. It could be:
- A Python script with `while` loop + OpenAI/Anthropic SDK
- A LangGraph graph with tool nodes
- A bash script calling `taskd` CLI + an LLM API
