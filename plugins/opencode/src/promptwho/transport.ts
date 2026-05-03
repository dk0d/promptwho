import { encode } from "@msgpack/msgpack";
import type { Event, OpencodeClient, Project } from "@opencode-ai/sdk";
import { resolveEvents, type PromptwhoResolvedEvent, type SourceEventEnvelope } from "./events";

export interface PromptwhoTransport {
  publish(
    client: OpencodeClient,
    events: SourceEventEnvelope[],
  ): Promise<void>;
  publishResolved(
    client: OpencodeClient,
    events: PromptwhoResolvedEvent[],
  ): Promise<void>;
}

export class HttpMsgpackTransport implements PromptwhoTransport {
  constructor(private readonly baseUrl = "http://127.0.0.1:8765") { }

  async publish(
    client: OpencodeClient,
    events: SourceEventEnvelope[],
  ): Promise<void> {
    const resolvedEvents = await resolveEvents(events);

    await this.publishResolved(client, resolvedEvents, events.map(({ event }) => event.type));
  }

  async publishResolved(
    client: OpencodeClient,
    events: PromptwhoResolvedEvent[],
    eventTypes: string[] = [],
  ): Promise<void> {
    const resolvedEvents = events;

    if (resolvedEvents.length === 0) {
      return;
    }

    const message = {
      request_id: crypto.randomUUID(),
      events: resolvedEvents,
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
            eventTypes,
            resolvedActions: resolvedEvents.map((event) => event.action),
          }
        }
      },)
      // throw new Error(
      //   `promptwho server rejected events: ${response.status} ${response.statusText}`,
      // );
    }
  }
}
