import { type Plugin, type PluginModule } from "@opencode-ai/plugin";
import { HttpMsgpackTransport } from "./promptwho/transport";

export const PromptwhoPlugin: Plugin = async ({
  client,
  project,
  directory,
  worktree,
}) => {
  const transport = new HttpMsgpackTransport();

  return {
    event: async ({ event }) => {
      await transport.publish(
        client,
        [
          {
            context: {
              project,
              directory,
              worktree,
            },
            event,
          },
        ]);
    },
    "chat.message": async ({ sessionID, agent, model, variant }, { message }) => { },
    "chat.params": async ({ sessionID, message, model }) => { },
    "shell.env": async ({ cwd, sessionID, callID }) => { },
  };
};

const pluginModule: PluginModule = {
  id: "promptwho",
  server: PromptwhoPlugin,
};

export const server = PromptwhoPlugin;
export default pluginModule;
