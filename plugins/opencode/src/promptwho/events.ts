import type { Event, Project } from "@opencode-ai/sdk";

export interface SourceEventEnvelope {
  context: {
    project: Pick<Project, "id" | "worktree" | "vcs"> & {
      name?: string;
      repositoryFingerprint?: string;
    };
    directory: string;
    worktree: string;
  };
  event: Event;
}

interface PromptwhoPayloadByAction {
  "server.instance.disposed": { directory: string };
  "server.connected": null;
  "installation.updated": { version: string };
  "installation.update-available": { version: string };
  "lsp.client.diagnostics": { server_id: string; path: string };
  "lsp.updated": null;
  "message.updated": { message_id: string; role: string; content?: string; token_count?: number };
  "message.removed": { message_id: string };
  "message.part.updated": { message_id: string; part_id: string; part_type: string; text?: string };
  "message.part.removed": { message_id: string; part_id: string };
  "permission.updated": { permission_id: string; permission_type: string; message_id?: string; tool_call_id?: string };
  "permission.replied": { permission_id: string; response: string };
  "session.status": { status: string; attempt?: number; message?: string; next?: number };
  "session.idle": null;
  "session.compacted": null;
  "file.edited": { file: string };
  "todo.updated": { todo_count: number };
  "command.executed": { message_id: string; name: string; arguments: string };
  "session.created": null;
  "session.updated": null;
  "session.deleted": null;
  "session.diff": { diff: Array<{ file: string; patch: string; additions: number; deletions: number; status?: string }> };
  "session.error": { error_name?: string; message?: string };
  "file.watcher.updated": { file: string; event: string };
  "vcs.branch.updated": { branch?: string };
  "tui.prompt.append": { text: string };
  "tui.command.execute": { command: string };
  "tui.toast.show": { variant: string; message: string };
  "pty.created": { pty_id: string; command?: string; cwd?: string; status?: string; exit_code?: number };
  "pty.updated": { pty_id: string; command?: string; cwd?: string; status?: string; exit_code?: number };
  "pty.exited": { pty_id: string; command?: string; cwd?: string; status?: string; exit_code?: number };
  "pty.deleted": { pty_id: string; command?: string; cwd?: string; status?: string; exit_code?: number };
  "tool.execute.before": { tool_call_id: string; tool_name: string; input: Record<string, unknown> };
  "tool.execute.after": { tool_call_id: string; tool_name: string; success: boolean; output: Record<string, unknown> };
  "shell.env": { cwd: string; tool_call_id?: string };
  "git.snapshot": { branch?: string; head_commit?: string; dirty: boolean; staged_files: string[]; unstaged_files: string[] };
}

type PromptwhoAction = keyof PromptwhoPayloadByAction;

type PromptwhoResolvedEvent = {
  [TAction in PromptwhoAction]: {
    id: string;
    version: "v1";
    occurred_at: string;
    project: { id: string; root: string; name?: string };
    session?: { id: string };
    source: { plugin: string; plugin_version: string; runtime: string };
    action: TAction;
    payload: PromptwhoPayloadByAction[TAction];
  };
}[PromptwhoAction];

const SOURCE = {
  plugin: "opencode",
  plugin_version: "1.14.30",
  runtime: "bun",
} as const;

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
}

function readString(record: Record<string, unknown> | undefined, ...keys: string[]): string | undefined {
  for (const key of keys) {
    const value = record?.[key];
    if (typeof value === "string" && value.length > 0) return value;
  }
}

function readNumber(record: Record<string, unknown> | undefined, ...keys: string[]): number | undefined {
  for (const key of keys) {
    const value = record?.[key];
    if (typeof value === "number" && Number.isFinite(value)) return value;
  }
}

function isoTimestamp(value?: number): string {
  return typeof value === "number" && Number.isFinite(value)
    ? new Date(value).toISOString()
    : new Date().toISOString();
}

function eventSessionId(event: Event): string | undefined {
  const properties = asRecord(event.properties);
  const info = asRecord(properties?.info);
  const part = asRecord(properties?.part);

  return readString(properties, "sessionID")
    ?? readString(info, "sessionID", "id")
    ?? readString(part, "sessionID");
}

function eventOccurredAt(event: Event): string {
  const properties = asRecord(event.properties);
  const info = asRecord(properties?.info);
  const part = asRecord(properties?.part);
  const time = asRecord(info?.time) ?? asRecord(part?.time);

  return isoTimestamp(
    readNumber(properties, "time")
    ?? readNumber(time, "completed", "end", "updated", "created", "start"),
  );
}

function buildPatch(before: string, after: string): string {
  return before === after ? "" : `--- before\n+++ after\n-${before}\n+${after}`;
}

function envelope<TAction extends PromptwhoAction>(
  sourceEvent: SourceEventEnvelope,
  action: TAction,
  payload: PromptwhoPayloadByAction[TAction],
): PromptwhoResolvedEvent {
  const sessionId = eventSessionId(sourceEvent.event);

  return {
    id: crypto.randomUUID(),
    version: "v1",
    occurred_at: eventOccurredAt(sourceEvent.event),
    project: {
      id: sourceEvent.context.project.id,
      root: sourceEvent.context.worktree || sourceEvent.context.directory || sourceEvent.context.project.worktree,
      name: sourceEvent.context.project.name,
      repository_fingerprint: sourceEvent.context.project.repositoryFingerprint,
    },
    ...(sessionId ? { session: { id: sessionId } } : {}),
    source: SOURCE,
    action,
    payload,
  } as PromptwhoResolvedEvent;
}

export function createEvent<TAction extends PromptwhoAction>(input: {
  context: SourceEventEnvelope["context"];
  action: TAction;
  payload: PromptwhoPayloadByAction[TAction];
  sessionId?: string;
  occurredAt?: string;
}): PromptwhoResolvedEvent {
  return {
    id: crypto.randomUUID(),
    version: "v1",
    occurred_at: input.occurredAt ?? new Date().toISOString(),
    project: {
      id: input.context.project.id,
      root: input.context.worktree || input.context.directory || input.context.project.worktree,
      name: input.context.project.name,
      repository_fingerprint: input.context.project.repositoryFingerprint,
    },
    ...(input.sessionId ? { session: { id: input.sessionId } } : {}),
    source: SOURCE,
    action: input.action,
    payload: input.payload,
  } as PromptwhoResolvedEvent;
}

function messageContent(info: Record<string, unknown> | undefined): string | undefined {
  const summary = asRecord(info?.summary);
  if (readString(info, "role") === "user") {
    return readString(summary, "body", "title") ?? readString(info, "system");
  }
}

function messageTokenCount(info: Record<string, unknown> | undefined): number | undefined {
  const tokens = asRecord(info?.tokens);
  const cache = asRecord(tokens?.cache);
  return readNumber(tokens, "total")
    ?? ((readNumber(tokens, "input") ?? 0)
      + (readNumber(tokens, "output") ?? 0)
      + (readNumber(tokens, "reasoning") ?? 0)
      + (readNumber(cache, "read") ?? 0)
      + (readNumber(cache, "write") ?? 0));
}

function ptyPayload(properties: Record<string, unknown>): PromptwhoPayloadByAction["pty.created"] {
  const info = asRecord(properties.info);
  return {
    pty_id: readString(info, "id") ?? readString(properties, "id") ?? "unknown",
    command: readString(info, "command"),
    cwd: readString(info, "cwd"),
    status: readString(info, "status"),
    exit_code: readNumber(properties, "exitCode"),
  };
}

function resolveEvent(sourceEvent: SourceEventEnvelope): PromptwhoResolvedEvent | null {
  const { event } = sourceEvent;
  const properties = asRecord(event.properties) ?? {};
  const info = asRecord(properties.info);
  const part = asRecord(properties.part);
  const status = asRecord(properties.status);

  switch (event.type) {
    case "server.instance.disposed":
      return envelope(sourceEvent, event.type, { directory: readString(properties, "directory") ?? "unknown" });
    case "server.connected":
    case "lsp.updated":
    case "session.idle":
    case "session.compacted":
    case "session.created":
    case "session.updated":
    case "session.deleted":
      return envelope(sourceEvent, event.type, null);
    case "installation.updated":
    case "installation.update-available":
      return envelope(sourceEvent, event.type, { version: readString(properties, "version") ?? "unknown" });
    case "lsp.client.diagnostics":
      return envelope(sourceEvent, event.type, {
        server_id: readString(properties, "serverID") ?? "unknown",
        path: readString(properties, "path") ?? "unknown",
      });
    case "message.updated":
      return envelope(sourceEvent, event.type, {
        message_id: readString(info, "id") ?? "unknown",
        role: readString(info, "role") ?? "unknown",
        content: messageContent(info),
        token_count: messageTokenCount(info),
      });
    case "message.removed":
      return envelope(sourceEvent, event.type, { message_id: readString(properties, "messageID") ?? "unknown" });
    case "message.part.updated":
      return envelope(sourceEvent, event.type, {
        message_id: readString(part, "messageID") ?? "unknown",
        part_id: readString(part, "id") ?? "unknown",
        part_type: readString(part, "type") ?? "unknown",
        text: readString(part, "text"),
      });
    case "message.part.removed":
      return envelope(sourceEvent, event.type, {
        message_id: readString(properties, "messageID") ?? "unknown",
        part_id: readString(properties, "partID") ?? "unknown",
      });
    case "permission.updated":
      return envelope(sourceEvent, event.type, {
        permission_id: readString(properties, "id") ?? "unknown",
        permission_type: readString(properties, "type") ?? "unknown",
        message_id: readString(properties, "messageID"),
        tool_call_id: readString(properties, "callID"),
      });
    case "permission.replied":
      return envelope(sourceEvent, event.type, {
        permission_id: readString(properties, "permissionID") ?? "unknown",
        response: readString(properties, "response") ?? "unknown",
      });
    case "session.status":
      return envelope(sourceEvent, event.type, {
        status: readString(status, "type") ?? "unknown",
        attempt: readNumber(status, "attempt"),
        message: readString(status, "message"),
        next: readNumber(status, "next"),
      });
    case "file.edited":
      return envelope(sourceEvent, event.type, { file: readString(properties, "file") ?? "unknown" });
    case "todo.updated":
      return envelope(sourceEvent, event.type, {
        todo_count: Array.isArray(properties.todos) ? properties.todos.length : 0,
      });
    case "command.executed":
      return envelope(sourceEvent, event.type, {
        message_id: readString(properties, "messageID") ?? "unknown",
        name: readString(properties, "name") ?? "unknown",
        arguments: readString(properties, "arguments") ?? "",
      });
    case "session.diff": {
      const diff = Array.isArray(properties.diff) ? properties.diff : [];
      return envelope(sourceEvent, event.type, {
        diff: diff.map((entry) => {
          const file = asRecord(entry) ?? {};
          const before = readString(file, "before") ?? "";
          const after = readString(file, "after") ?? "";
          return {
            file: readString(file, "file") ?? "unknown",
            patch: readString(file, "patch") ?? buildPatch(before, after),
            additions: readNumber(file, "additions") ?? 0,
            deletions: readNumber(file, "deletions") ?? 0,
            status: readString(file, "status"),
          };
        }),
      });
    }
    case "session.error": {
      const error = asRecord(properties.error);
      const errorData = asRecord(error?.data);
      return envelope(sourceEvent, event.type, {
        error_name: readString(error, "name"),
        message: readString(errorData, "message"),
      });
    }
    case "file.watcher.updated":
      return envelope(sourceEvent, event.type, {
        file: readString(properties, "file") ?? "unknown",
        event: readString(properties, "event") ?? "unknown",
      });
    case "vcs.branch.updated":
      return envelope(sourceEvent, event.type, { branch: readString(properties, "branch") });
    case "tui.prompt.append":
      return envelope(sourceEvent, event.type, { text: readString(properties, "text") ?? "" });
    case "tui.command.execute":
      return envelope(sourceEvent, event.type, { command: readString(properties, "command") ?? "unknown" });
    case "tui.toast.show":
      return envelope(sourceEvent, event.type, {
        variant: readString(properties, "variant") ?? "info",
        message: readString(properties, "message") ?? "",
      });
    case "pty.created":
    case "pty.updated":
    case "pty.exited":
    case "pty.deleted":
      return envelope(sourceEvent, event.type, ptyPayload(properties));
    default:
      return null;
  }
}

export async function resolveEvents(sourceEvents: SourceEventEnvelope[]): Promise<PromptwhoResolvedEvent[]> {
  return sourceEvents
    .map(resolveEvent)
    .filter((event): event is PromptwhoResolvedEvent => event !== null);
}

export type { PromptwhoResolvedEvent };
