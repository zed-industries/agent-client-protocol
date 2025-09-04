import asyncio
import os
import sys
from acp import (
    ClientSideConnection,
    InitializeRequest,
    NewSessionRequest,
    PromptRequest,
    PROTOCOL_VERSION,
    stdio_streams,
)


class MinimalClient:
    async def writeTextFile(self, params):
        print(f"write {params.path}", file=sys.stderr)

    async def readTextFile(self, params):
        return {"content": "example"}

    async def requestPermission(self, params):
        return {"outcome": {"outcome": "selected", "optionId": "allow"}}

    async def sessionUpdate(self, params):
        print(f"session update: {params}", file=sys.stderr)


async def main() -> None:
    reader, writer = await stdio_streams()
    client_conn = ClientSideConnection(lambda _agent: MinimalClient(), writer, reader)
    # 1) initialize
    resp = await client_conn.initialize(InitializeRequest(protocolVersion=PROTOCOL_VERSION))
    print(f"Initialized with protocol version: {resp.protocolVersion}", file=sys.stderr)
    # 2) new session
    new_sess = await client_conn.newSession(NewSessionRequest(mcpServers=[], cwd=os.getcwd()))
    # 3) prompt
    await client_conn.prompt(
        PromptRequest(
            sessionId=new_sess.sessionId,
            prompt=[{"type": "text", "text": "Hello from client"}],
        )
    )
    # Small grace period to allow duplex messages to flush
    await asyncio.sleep(0.2)


if __name__ == "__main__":
    asyncio.run(main())
