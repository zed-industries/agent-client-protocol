# Agent-Client Protocol Examples

This directory contains minimal examples demonstrating how to use the Agent-Client Protocol (ACP) TypeScript library.

## Files

- `agent.ts` - A minimal agent implementation using `AgentSideConnection`
- `client.ts` - A minimal client implementation using `ClientSideConnection` that spawns the agent as a subprocess

## Usage

These examples demonstrate the basic connection setup between an agent and client. They implement the required interfaces with minimal functionality - just logging and returning valid responses without building a real agent or client.

### Building and Running

From the project root directory:

```bash
# Install tsx if not already available
npm install

# Run the client example directly with tsx (which spawns the agent automatically)
cd typescript/examples
npx tsx client.ts
```

Or run from the root directory:

```bash
# Run the client example
npx tsx typescript/examples/client.ts
```

The client will:

1. Spawn the agent as a subprocess
2. Initialize the connection using the ACP protocol
3. Create a new session
4. Send a test prompt
5. Log all interactions to stderr

## Key Features Demonstrated

- **Connection Setup**: Shows how to create `AgentSideConnection` and `ClientSideConnection` with process stdin/stdout streams
- **Interface Implementation**: Minimal implementations of the `Agent` and `Client` interfaces
- **Protocol Flow**: Demonstrates the initialize → new session → prompt flow
- **Subprocess Communication**: Shows how a client can spawn and communicate with an agent process

## Important Notes

- These are minimal examples for demonstration purposes only
- All methods just log their inputs and return valid but minimal responses
- The agent and client don't implement any real AI or file system functionality
- Error handling is minimal - production code should be more robust
- The examples use `process.stdin`/`process.stdout` for communication as specified

## Schema Validation

The examples rely on the ACP library's built-in schema validation using Zod. The examples use `tsx` to run TypeScript directly without needing to compile to JavaScript first.
