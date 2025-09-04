import asyncio
from acp import (
    AgentSideConnection,
    InitializeRequest,
    InitializeResponse,
    NewSessionRequest,
    NewSessionResponse,
    PromptRequest,
    PromptResponse,
    CancelNotification,
    AuthenticateRequest,
    stdio_streams,
)


class EchoAgent:
    async def initialize(self, params: InitializeRequest) -> InitializeResponse:
        # Avoid serializer warnings by omitting defaults
        return InitializeResponse(protocolVersion=params.protocolVersion, agentCapabilities=None, authMethods=[])

    async def newSession(self, params: NewSessionRequest) -> NewSessionResponse:
        return NewSessionResponse(sessionId="sess-1")

    async def loadSession(self, params):
        return None

    async def authenticate(self, params: AuthenticateRequest) -> None:
        return None

    async def prompt(self, params: PromptRequest) -> PromptResponse:
        # Normally you'd stream updates via sessionUpdate
        return PromptResponse(stopReason="end_turn")

    async def cancel(self, params: CancelNotification) -> None:
        return None


async def main() -> None:
    reader, writer = await stdio_streams()
    # For an agent process, local writes go to client stdin (writer=stdout)
    AgentSideConnection(lambda _conn: EchoAgent(), writer, reader)
    # Keep running; in a real agent you would await tasks or add your own loop
    await asyncio.Event().wait()


if __name__ == "__main__":
    asyncio.run(main())
