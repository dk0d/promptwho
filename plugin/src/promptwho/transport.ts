import { encode } from "@msgpack/msgpack";
import type { Event, OpencodeClient, Project } from "@opencode-ai/sdk";

export interface PromptwhoTransport {
  publish(
    client: OpencodeClient,
    events: Array<{
      context: {
        project: Pick<Project, "id" | "worktree" | "vcs"> & { name?: string };
        directory: string;
        worktree: string;
      };
      event: Event;
    }>,
  ): Promise<void>;
}

export class HttpMsgpackTransport implements PromptwhoTransport {
  constructor(private readonly baseUrl = "http://127.0.0.1:8765") { }

  async publish(
    client: OpencodeClient,
    events: Array<{
      context: {
        project: Pick<Project, "id" | "worktree" | "vcs"> & { name?: string };
        directory: string;
        worktree: string;
      };
      event: Event;
    }>,
  ): Promise<void> {
    const message = {
      flavor: "opencode",
      request_id: crypto.randomUUID(),
      events,
    };

    const response = await fetch(`${this.baseUrl}/v1/events`, {
      method: "POST",
      headers: {
        "content-type": "application/msgpack",
        accept: "application/json",
      },
      body: encode(message),
    });

    if (!response.ok) {
      const responseBody = await response.text().catch(() => "<unavailable>");

      client.app.log({
        body: {
          service: "promptwho",
          level: "error",
          message: "Failed to publish events to promptwho server",
          extra: {
            status: response.status,
            statusText: response.statusText,
            responseBody,
            eventTypes: events.map(({ event }) => event.type),
          }
        }
      },)
      // throw new Error(
      //   `promptwho server rejected events: ${response.status} ${response.statusText}`,
      // );
    }
  }
}
