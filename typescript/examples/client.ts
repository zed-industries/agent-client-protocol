#!/usr/bin/env node

import { ClientSideConnection, Client, PROTOCOL_VERSION } from "../acp.js";
import * as schema from "../schema.js";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { Writable, Readable } from "node:stream";
import * as fs from "node:fs";

class ExampleClient implements Client {
  async requestPermission(
    params: schema.RequestPermissionRequest,
  ): Promise<schema.RequestPermissionResponse> {
    console.log(`\n🔐 Permission requested: ${params.toolCall.title}`);

    console.log(`\nOptions:`);
    params.options.forEach((option, index) => {
      console.log(`   ${index + 1}. ${option.name} (${option.kind})`);
    });

    const answer = this.askUser(`\nYour choice: `);
    const trimmedAnswer = answer.trim();

    let selectedOption: schema.PermissionOption;

    const optionIndex = parseInt(trimmedAnswer) - 1;
    if (optionIndex >= 0 && optionIndex < params.options.length) {
      selectedOption = params.options[optionIndex];
    } else {
      selectedOption = params.options[params.options.length - 1] || params.options[0];
    }

    return {
      outcome: {
        outcome: "selected",
        optionId: selectedOption.optionId,
      },
    };
  }

  private askUser(question: string): string {
    process.stdout.write(question);

    // Read from stdin
    const fd = process.stdin.fd;
    const buffer = Buffer.alloc(1024);

    try {
      const bytesRead = fs.readSync(fd, buffer, 0, buffer.length, null);
      const input = buffer.toString('utf8', 0, bytesRead).trim();
      return input;
    } catch (error) {
      console.error("Error reading input:", error);
      return "deny"; // Default to deny for safety
    }
  }

  async sessionUpdate(params: schema.SessionNotification): Promise<void> {
    const update = params.update;

    switch (update.sessionUpdate) {
      case "agent_message_chunk":
        if (update.content.type === "text") {
          process.stdout.write(update.content.text);
        } else {
          console.log(`[${update.content.type}]`);
        }
        break;
      case "tool_call":
        console.log(`\n\n🔧 ${update.title} (${update.status})\n\n`);
        break;
      case "tool_call_update":
        console.log(`\n🔧 Tool call \`${update.toolCallId}\` updated: ${update.status}`);
        break;
      case "plan":
      case "agent_thought_chunk":
      case "user_message_chunk":
        console.log(`[${update.sessionUpdate}]`);
        break;
    }
  }

  async writeTextFile(
    params: schema.WriteTextFileRequest,
  ): Promise<schema.WriteTextFileResponse> {
    console.error(
      "[Client] Write text file called with:",
      JSON.stringify(params, null, 2),
    );

    return null;
  }

  async readTextFile(
    params: schema.ReadTextFileRequest,
  ): Promise<schema.ReadTextFileResponse> {
    console.error(
      "[Client] Read text file called with:",
      JSON.stringify(params, null, 2),
    );

    return {
      content: "Mock file content",
    };
  }
}

async function main() {
  // Get the current file's directory to find agent.ts
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);
  const agentPath = join(__dirname, "agent.ts");

  console.error("[Client] Spawning agent subprocess...");

  // Spawn the agent as a subprocess using tsx
  const agentProcess = spawn("npx", ["tsx", agentPath], {
    stdio: ["pipe", "pipe", "inherit"],
  });

  // Create streams to communicate with the agent
  const input = Writable.toWeb(agentProcess.stdin!) as WritableStream;
  const output = Readable.toWeb(agentProcess.stdout!) as ReadableStream<Uint8Array>;

  // Create the client connection
  const client = new ExampleClient();
  const connection = new ClientSideConnection(
    (agent) => client,
    input,
    output,
  );

  console.error("[Client] Initializing connection...");

  try {
    // Initialize the connection
    const initResult = await connection.initialize({
      protocolVersion: PROTOCOL_VERSION,
      clientCapabilities: {
        fs: {
          readTextFile: true,
          writeTextFile: true,
        },
      },
    });

    console.log(`✅ Connected to agent (protocol v${initResult.protocolVersion})`);

    // Create a new session
    const sessionResult = await connection.newSession({
      cwd: process.cwd(),
      mcpServers: [],
    });

    console.log(`📝 Created session: ${sessionResult.sessionId}`);
    console.log(`💬 User: Hello, agent!`);
    console.log(`🤖 Agent:`);
    process.stdout.write(" ");

    // Send a test prompt
    const promptResult = await connection.prompt({
      sessionId: sessionResult.sessionId,
      prompt: [
        {
          type: "text",
          text: "Hello, agent!",
        },
      ],
    });

    console.log(`\n\n✅ Agent completed with: ${promptResult.stopReason}`);
  } catch (error) {
    console.error("[Client] Error:", error);
  } finally {
    agentProcess.kill();
  }
}

main().catch(console.error);
