import asyncio
from dataclasses import dataclass
from typing import List, Optional

import pytest

from acp import (
    AgentSideConnection,
    ClientSideConnection,
    InitializeRequest,
    InitializeResponse,
    NewSessionRequest,
    NewSessionResponse,
    PromptRequest,
    PromptResponse,
    CancelNotification,
    ReadTextFileRequest,
    ReadTextFileResponse,
    WriteTextFileRequest,
    RequestPermissionRequest,
    RequestPermissionResponse,
    SessionNotification,
)


# --------------------- Test Utilities ---------------------

class _Server:
    def __init__(self) -> None:
        self._server: Optional[asyncio.AbstractServer] = None
        self.server_reader: Optional[asyncio.StreamReader] = None
        self.server_writer: Optional[asyncio.StreamWriter] = None
        self.client_reader: Optional[asyncio.StreamReader] = None
        self.client_writer: Optional[asyncio.StreamWriter] = None

    async def __aenter__(self):
        async def handle(reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
            self.server_reader = reader
            self.server_writer = writer

        self._server = await asyncio.start_server(handle, host="127.0.0.1", port=0)
        host, port = self._server.sockets[0].getsockname()[:2]
        self.client_reader, self.client_writer = await asyncio.open_connection(host, port)

        # wait until server side is set
        for _ in range(100):
            if self.server_reader and self.server_writer:
                break
            await asyncio.sleep(0.01)
        assert self.server_reader and self.server_writer
        assert self.client_reader and self.client_writer
        return self

    async def __aexit__(self, exc_type, exc, tb):
        if self.client_writer:
            self.client_writer.close()
            try:
                await self.client_writer.wait_closed()
            except Exception:
                pass
        if self.server_writer:
            self.server_writer.close()
            try:
                await self.server_writer.wait_closed()
            except Exception:
                pass
        if self._server:
            self._server.close()
            await self._server.wait_closed()


# --------------------- Test Doubles -----------------------

@dataclass
class TestClient:
    __test__ = False  # prevent pytest from collecting this class
    permission_outcomes: List[dict]
    files: dict
    notifications: List[SessionNotification]

    def __init__(self) -> None:
        self.permission_outcomes = []
        self.files = {}
        self.notifications = []

    async def requestPermission(self, params: RequestPermissionRequest) -> RequestPermissionResponse:
        outcome = self.permission_outcomes.pop() if self.permission_outcomes else {"outcome": "cancelled"}
        return RequestPermissionResponse.model_validate({"outcome": outcome})

    async def writeTextFile(self, params: WriteTextFileRequest) -> None:
        self.files[str(params.path)] = params.content

    async def readTextFile(self, params: ReadTextFileRequest) -> ReadTextFileResponse:
        content = self.files.get(str(params.path), "default content")
        return ReadTextFileResponse(content=content)

    async def sessionUpdate(self, params: SessionNotification) -> None:
        self.notifications.append(params)


@dataclass
class TestAgent:
    __test__ = False  # prevent pytest from collecting this class
    prompts: List[PromptRequest]
    cancellations: List[str]

    def __init__(self) -> None:
        self.prompts = []
        self.cancellations = []

    async def initialize(self, params: InitializeRequest) -> InitializeResponse:
        # Avoid serializer warnings by omitting defaults
        return InitializeResponse(protocolVersion=params.protocolVersion, agentCapabilities=None, authMethods=[])

    async def newSession(self, params: NewSessionRequest) -> NewSessionResponse:
        return NewSessionResponse(sessionId="test-session-123")

    async def loadSession(self, params) -> None:  # noqa: ANN001
        return None

    async def authenticate(self, params) -> None:  # noqa: ANN001
        return None

    async def prompt(self, params: PromptRequest) -> PromptResponse:
        self.prompts.append(params)
        return PromptResponse(stopReason="end_turn")

    async def cancel(self, params: CancelNotification) -> None:
        self.cancellations.append(params.sessionId)


# ------------------------ Tests --------------------------

@pytest.mark.asyncio
async def test_initialize_and_new_session():
    async with _Server() as s:
        agent = TestAgent()
        client = TestClient()
        # server side is agent; client side is client
        agent_conn = ClientSideConnection(lambda _conn: client, s.client_writer, s.client_reader)
        client_conn = AgentSideConnection(lambda _conn: agent, s.server_writer, s.server_reader)

        resp = await agent_conn.initialize(InitializeRequest(protocolVersion=1))
        assert isinstance(resp, InitializeResponse)
        assert resp.protocolVersion == 1

        new_sess = await agent_conn.newSession(NewSessionRequest(mcpServers=[], cwd="/test"))
        assert new_sess.sessionId == "test-session-123"


@pytest.mark.asyncio
async def test_bidirectional_file_ops():
    async with _Server() as s:
        agent = TestAgent()
        client = TestClient()
        client.files["/test/file.txt"] = "Hello, World!"
        agent_conn = ClientSideConnection(lambda _conn: client, s.client_writer, s.client_reader)
        client_conn = AgentSideConnection(lambda _conn: agent, s.server_writer, s.server_reader)

        # Agent asks client to read
        res = await client_conn.readTextFile(
            ReadTextFileRequest(sessionId="sess", path="/test/file.txt")
        )
        assert res.content == "Hello, World!"

        # Agent asks client to write
        await client_conn.writeTextFile(
            WriteTextFileRequest(sessionId="sess", path="/test/file.txt", content="Updated")
        )
        assert client.files["/test/file.txt"] == "Updated"


@pytest.mark.asyncio
async def test_cancel_notification_and_capture_wire():
    async with _Server() as s:
        # Build only agent-side (server) connection. Client side: raw reader to inspect wire
        agent = TestAgent()
        client = TestClient()
        agent_conn = ClientSideConnection(lambda _conn: client, s.client_writer, s.client_reader)
        _client_conn = AgentSideConnection(lambda _conn: agent, s.server_writer, s.server_reader)

        # Send cancel notification from client-side connection to agent
        await agent_conn.cancel(CancelNotification(sessionId="test-123"))

        # Read raw line from server peer (it will be consumed by agent receive loop quickly).
        # Instead, wait a brief moment and assert agent recorded it.
        for _ in range(50):
            if agent.cancellations:
                break
            await asyncio.sleep(0.01)
        assert agent.cancellations == ["test-123"]


@pytest.mark.asyncio
async def test_session_notifications_flow():
    async with _Server() as s:
        agent = TestAgent()
        client = TestClient()
        agent_conn = ClientSideConnection(lambda _conn: client, s.client_writer, s.client_reader)
        client_conn = AgentSideConnection(lambda _conn: agent, s.server_writer, s.server_reader)

        # Agent -> Client notifications
        await client_conn.sessionUpdate(
            SessionNotification(
                sessionId="sess",
                update={
                    "sessionUpdate": "agent_message_chunk",
                    "content": {"type": "text", "text": "Hello"},
                },
            )
        )
        await client_conn.sessionUpdate(
            SessionNotification(
                sessionId="sess",
                update={
                    "sessionUpdate": "user_message_chunk",
                    "content": {"type": "text", "text": "World"},
                },
            )
        )

        # Wait for async dispatch
        for _ in range(50):
            if len(client.notifications) >= 2:
                break
            await asyncio.sleep(0.01)
        assert len(client.notifications) >= 2
        assert client.notifications[0].sessionId == "sess"


@pytest.mark.asyncio
async def test_concurrent_reads():
    async with _Server() as s:
        agent = TestAgent()
        client = TestClient()
        for i in range(5):
            client.files[f"/test/file{i}.txt"] = f"Content {i}"
        agent_conn = ClientSideConnection(lambda _conn: client, s.client_writer, s.client_reader)
        client_conn = AgentSideConnection(lambda _conn: agent, s.server_writer, s.server_reader)

        async def read_one(i: int):
            return await client_conn.readTextFile(
                ReadTextFileRequest(sessionId="sess", path=f"/test/file{i}.txt")
            )

        results = await asyncio.gather(*(read_one(i) for i in range(5)))
        for i, res in enumerate(results):
            assert res.content == f"Content {i}"
