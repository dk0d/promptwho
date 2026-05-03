import { type Plugin, type PluginModule } from "@opencode-ai/plugin";
import { getGitProjectIdentity, getGitSnapshot } from "./promptwho/git";
import { createEvent } from "./promptwho/events";
import { HttpMsgpackTransport } from "./promptwho/transport";

export const PromptwhoPlugin: Plugin = async ({
  client,
  project,
  directory,
  worktree,
}) => {
  const transport = new HttpMsgpackTransport();
  const gitIdentity = await getGitProjectIdentity(worktree || directory);
  const context = {
    project: {
      ...project,
      repositoryFingerprint: gitIdentity?.repositoryFingerprint,
    },
    directory,
    worktree,
  };

  return {
    event: async ({ event }) => {
      await transport.publish(
        client,
        [
          {
            context,
            event,
          },
        ]);
    },
    "chat.message": async () => { },
    "chat.params": async ({ sessionID, message, model }) => { },
    "tool.execute.before": async ({ tool, sessionID, callID }, { args }) => {
      await transport.publishResolved(client, [
        createEvent({
          context,
          sessionId: sessionID,
          action: "tool.execute.before",
          payload: {
            tool_call_id: callID,
            tool_name: tool,
            input: args ?? {},
          },
        }),
      ]);
    },
    "tool.execute.after": async ({ tool, sessionID, callID }, output) => {
      const snapshot = await getGitSnapshot(worktree || directory);
      const events = [
        createEvent({
          context,
          sessionId: sessionID,
          action: "tool.execute.after",
          payload: {
            tool_call_id: callID,
            tool_name: tool,
            success: true,
            output: {
              title: output.title,
              output: output.output,
              metadata: output.metadata,
            },
          },
        }),
      ];

      if (snapshot) {
        events.push(createEvent({
          context,
          sessionId: sessionID,
          action: "git.snapshot",
          payload: snapshot,
        }));
      }

      await transport.publishResolved(client, events);
    },
    "shell.env": async ({ cwd, sessionID, callID }) => {
      await transport.publishResolved(client, [
        createEvent({
          context,
          sessionId: sessionID,
          action: "shell.env",
          payload: {
            cwd,
            tool_call_id: callID,
          },
        }),
      ]);
    },
  };
};

const pluginModule: PluginModule = {
  id: "promptwho",
  server: PromptwhoPlugin,
};

export const server = PromptwhoPlugin;
export default pluginModule;
