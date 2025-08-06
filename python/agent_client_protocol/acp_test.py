"""
Test suite for the Agent Client Protocol (ACP) Python implementation.

This module contains comprehensive tests for the ACP protocol, including:
- Bidirectional communication between agents and clients
- Error handling in both directions
- Concurrent request handling
- Message ordering verification
- Notification handling
- All core protocol operations (initialize, sessions, file operations, etc.)

Run tests with: uv run pytest agent_client_protocol/acp_test.py -v
"""
import pytest
import pytest_asyncio
import asyncio
from typing import Tuple, cast

from agent_client_protocol.acp import (
    Agent,
    Client,
    AgentSideConnection,
    ClientSideConnection,
    PROTOCOL_VERSION,
)
from agent_client_protocol import schema


async def create_stream() -> Tuple[asyncio.StreamReader, asyncio.StreamWriter]:
    """
    Create a connected stream pair for testing communication.

    Uses subprocess 'cat' process to create real asyncio streams that can
    handle the JSON-RPC protocol communication.

    Returns:
        Tuple of (reader, writer) for the stream
    """
    proc = await asyncio.create_subprocess_exec(
        'cat',
        stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE
    )

    # Ensure we have valid streams
    assert proc.stdin is not None
    assert proc.stdout is not None

    return proc.stdout, proc.stdin


@pytest_asyncio.fixture
async def connection_setup():
    """Fixture to setup test connections"""
    # Create stream pairs
    client_to_agent_r, client_to_agent_w = await create_stream()
    agent_to_client_r, agent_to_client_w = await create_stream()

    # Create test instances
    test_client = MockClient()
    test_agent = MockAgent()

    # Create connections
    agent_connection = ClientSideConnection(
        lambda agent: test_client,
        agent_to_client_r,
        client_to_agent_w,
    )

    client_connection = AgentSideConnection(
        lambda client: test_agent,
        client_to_agent_r,
        agent_to_client_w,
    )

    yield {
        'agent_connection': agent_connection,
        'client_connection': client_connection,
        'test_client': test_client,
        'test_agent': test_agent
    }

    # Cleanup
    await agent_connection.close()
    await client_connection.close()


class MockClient(Client):
    """Mock client implementation for testing"""

    def __init__(self):
        self.write_file_calls = []
        self.read_file_calls = []
        self.permission_calls = []
        self.session_update_calls = []
        self.should_raise = False
        self.error_message = "Test error"

    async def write_text_file(
        self, params: schema.WriteTextFileRequest
    ) -> schema.WriteTextFileResponse:
        self.write_file_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)
        return None

    async def read_text_file(
        self, params: schema.ReadTextFileRequest
    ) -> schema.ReadTextFileResponse:
        self.read_file_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)
        return cast(schema.ReadTextFileResponse, {"content": f"Content of {params['path']}"})

    async def request_permission(
        self, params: schema.RequestPermissionRequest
    ) -> schema.RequestPermissionResponse:
        self.permission_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)
        return cast(schema.RequestPermissionResponse, {
            "outcome": cast(schema.RequestPermissionOutcome, {
                "outcome": "selected",
                "optionId": "allow",
            })
        })

    async def session_update(self, params: schema.SessionNotification) -> None:
        self.session_update_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)


class MockAgent(Agent):
    """Mock agent implementation for testing"""

    def __init__(self):
        self.initialize_calls = []
        self.new_session_calls = []
        self.load_session_calls = []
        self.authenticate_calls = []
        self.prompt_calls = []
        self.cancel_calls = []
        self.should_raise = False
        self.error_message = "Test error"

    async def initialize(
        self, params: schema.InitializeRequest
    ) -> schema.InitializeResponse:
        self.initialize_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)
        return cast(schema.InitializeResponse, {
            "protocolVersion": params["protocolVersion"],
            "agentCapabilities": cast(schema.AgentCapabilities, {"loadSession": True}),
            "authMethods": [
                cast(schema.AuthMethod, {
                    "id": "oauth",
                    "label": "OAuth",
                    "description": "Authenticate with OAuth",
                })
            ],
        })

    async def new_session(
        self, params: schema.NewSessionRequest
    ) -> schema.NewSessionResponse:
        self.new_session_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)
        return cast(schema.NewSessionResponse, {
            "sessionId": "test-session"
        })

    async def load_session(
        self, params: schema.LoadSessionRequest
    ) -> schema.LoadSessionResponse:
        self.load_session_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)
        return cast(schema.LoadSessionResponse, {
            "authRequired": False,
            "authMethods": []
        })

    async def authenticate(self, params: schema.AuthenticateRequest) -> None:
        self.authenticate_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)

    async def prompt(self, params: schema.PromptRequest) -> None:
        self.prompt_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)

    async def cancel(self, params: schema.CancelNotification) -> None:
        self.cancel_calls.append(params)
        if self.should_raise:
            raise Exception(self.error_message)


@pytest.mark.asyncio
class TestACP:
    """
    Comprehensive test suite for the Agent Client Protocol (ACP).

    These tests verify the protocol implementation by testing:
    - Error handling and propagation
    - Concurrent request processing
    - Message ordering guarantees
    - Notification delivery
    - All protocol methods and their expected behavior
    """

    async def test_handles_errors_in_bidirectional_communication(self, connection_setup):
        """
        Test that errors are properly handled and propagated in both directions.

        Verifies that when either the client or agent throws an exception,
        the error is properly caught and re-raised on the calling side.
        """
        setup = connection_setup

        # Configure client to throw errors
        setup['test_client'].should_raise = True
        setup['test_client'].error_message = "Write failed"

        # Test error handling in client->agent direction
        with pytest.raises(Exception):
            await setup['client_connection'].write_text_file({
                "path": "/test.txt",
                "content": "test",
                "sessionId": "test-session",
            })

        # Configure agent to throw errors
        setup['test_agent'].should_raise = True
        setup['test_agent'].error_message = "Failed to create session"

        # Test error handling in agent->client direction
        with pytest.raises(Exception):
            await setup['agent_connection'].new_session({
                "cwd": "/test",
                "mcpServers": [],
            })

    async def test_handles_concurrent_requests(self, connection_setup):
        """
        Test that multiple concurrent requests are handled correctly.

        Sends multiple file write requests concurrently and verifies that
        all complete successfully without interfering with each other.
        """
        request_count = 0

        class ConcurrentMockClient(Client):
            async def write_text_file(self, params):
                nonlocal request_count
                request_count += 1
                current_count = request_count
                await asyncio.sleep(0.04)  # 40ms delay
                print(f"Write request {current_count} completed")
                return None

            async def read_text_file(self, params):
                return cast(schema.ReadTextFileResponse, {"content": f"Content of {params['path']}"})

            async def request_permission(self, params):
                return cast(schema.RequestPermissionResponse, {
                    "outcome": cast(schema.RequestPermissionOutcome, {
                        "outcome": "selected",
                        "optionId": "allow",
                    })
                })

            async def session_update(self, params):
                pass

        # Create new connection with concurrent client
        client_to_agent_r, client_to_agent_w = await create_stream()
        agent_to_client_r, agent_to_client_w = await create_stream()

        concurrent_client = ConcurrentMockClient()
        concurrent_agent_connection = ClientSideConnection(
            lambda agent: concurrent_client,
            agent_to_client_r,
            client_to_agent_w,
        )

        concurrent_client_connection = AgentSideConnection(
            lambda client: connection_setup['test_agent'],
            client_to_agent_r,
            agent_to_client_w,
        )

        # Send multiple concurrent requests
        tasks = [
            concurrent_client_connection.write_text_file({
                "path": "/file1.txt",
                "content": "content1",
                "sessionId": "session1",
            }),
            concurrent_client_connection.write_text_file({
                "path": "/file2.txt",
                "content": "content2",
                "sessionId": "session1",
            }),
            concurrent_client_connection.write_text_file({
                "path": "/file3.txt",
                "content": "content3",
                "sessionId": "session1",
            }),
        ]

        results = await asyncio.gather(*tasks)

        # Verify all requests completed successfully
        assert len(results) == 3
        assert all(result is None for result in results)
        assert request_count == 3

        # Cleanup
        await concurrent_agent_connection.close()
        await concurrent_client_connection.close()

    async def test_handles_message_ordering_correctly(self, connection_setup):
        """
        Test that messages are processed in the correct order.

        Sends a sequence of different operations and verifies they are
        executed in the same order they were sent.
        """
        message_log = []

        class OrderedMockClient(Client):
            async def write_text_file(self, params):
                message_log.append(f"writeTextFile called: {params['path']}")
                return None

            async def read_text_file(self, params):
                message_log.append(f"readTextFile called: {params['path']}")
                return cast(schema.ReadTextFileResponse, {"content": "test content"})

            async def request_permission(self, params):
                message_log.append(f"requestPermission called: {params['toolCall']['title']}")
                return cast(schema.RequestPermissionResponse, {
                    "outcome": cast(schema.RequestPermissionOutcome, {
                        "outcome": "selected",
                        "optionId": "allow",
                    })
                })

            async def session_update(self, params):
                message_log.append("sessionUpdate called")

        class OrderedMockAgent(Agent):
            async def initialize(self, params):
                return cast(schema.InitializeResponse, {
                    "protocolVersion": 1,
                    "agentCapabilities": cast(schema.AgentCapabilities, {}),
                    "authMethods": [],
                })

            async def new_session(self, params):
                message_log.append(f"newSession called: {params['cwd']}")
                return cast(schema.NewSessionResponse, {
                    "sessionId": "test-session"
                })

            async def load_session(self, params):
                message_log.append(f"loadSession called: {params['sessionId']}")
                return cast(schema.LoadSessionResponse, {
                    "authMethods": [],
                    "authRequired": False,
                })

            async def authenticate(self, params):
                message_log.append(f"authenticate called: {params['methodId']}")

            async def prompt(self, params):
                message_log.append(f"prompt called: {params['sessionId']}")

            async def cancel(self, params):
                message_log.append(f"cancelled called: {params['sessionId']}")

        # Create ordered connections
        ordered_client = OrderedMockClient()
        ordered_agent = OrderedMockAgent()

        client_to_agent_r, client_to_agent_w = await create_stream()
        agent_to_client_r, agent_to_client_w = await create_stream()

        agent_connection = ClientSideConnection(
            lambda agent: ordered_client,
            agent_to_client_r,
            client_to_agent_w,
        )

        client_connection = AgentSideConnection(
            lambda client: ordered_agent,
            client_to_agent_r,
            agent_to_client_w,
        )

        # Send requests in specific order
        await agent_connection.new_session({
            "cwd": "/test",
            "mcpServers": [],
        })

        await client_connection.write_text_file({
            "path": "/test.txt",
            "content": "test",
            "sessionId": "test-session",
        })

        await client_connection.read_text_file({
            "path": "/test.txt",
            "sessionId": "test-session",
        })

        await client_connection.request_permission({
            "sessionId": "test-session",
            "toolCall": cast(schema.ToolCall, {
                "title": "Execute command",
                "kind": "execute",
                "status": "pending",
                "toolCallId": "tool-123",
                "content": [
                    cast(schema.ToolCallContent, {
                        "type": "content",
                        "content": cast(schema.ContentBlock, {
                            "type": "text",
                            "text": "ls -la",
                        }),
                    })
                ],
            }),
            "options": [
                cast(schema.PermissionOption, {
                    "kind": "allow_once",
                    "name": "Allow",
                    "optionId": "allow",
                }),
                cast(schema.PermissionOption, {
                    "kind": "reject_once",
                    "name": "Reject",
                    "optionId": "reject",
                }),
            ],
        })

        # Clean up
        await agent_connection.close()
        await client_connection.close()

        # Verify order
        expected_log = [
            "newSession called: /test",
            "writeTextFile called: /test.txt",
            "readTextFile called: /test.txt",
            "requestPermission called: Execute command",
        ]
        assert message_log == expected_log

    async def test_handles_notifications_correctly(self, connection_setup):
        """
        Test that notifications are properly delivered and handled.

        Verifies that both session updates and cancel notifications
        are delivered to their respective handlers.
        """
        notification_log = []

        class NotificationMockClient(Client):
            async def write_text_file(self, params):
                return None

            async def read_text_file(self, params):
                return cast(schema.ReadTextFileResponse, {"content": "test"})

            async def request_permission(self, params):
                return cast(schema.RequestPermissionResponse, {
                    "outcome": cast(schema.RequestPermissionOutcome, {
                        "outcome": "selected",
                        "optionId": "allow",
                    })
                })

            async def session_update(self, params: schema.SessionNotification):
                if (
                    params.get("update")
                    and isinstance(params["update"], dict)
                    and params["update"].get("sessionUpdate") == "agent_message_chunk"
                ):
                    content = params["update"].get("content", {})
                    if isinstance(content, dict) and content.get("type") == "text" and "text" in content:
                        notification_log.append(f"agent message: {content['text']}")

        class NotificationMockAgent(Agent):
            async def initialize(self, params):
                return cast(schema.InitializeResponse, {
                    "protocolVersion": 1,
                    "agentCapabilities": cast(schema.AgentCapabilities, {}),
                    "authMethods": [],
                })

            async def new_session(self, params):
                return cast(schema.NewSessionResponse, {
                    "sessionId": "test-session"
                })

            async def load_session(self, params):
                return cast(schema.LoadSessionResponse, {
                    "authMethods": [],
                    "authRequired": False,
                })

            async def authenticate(self, params):
                pass

            async def prompt(self, params):
                pass

            async def cancel(self, params):
                notification_log.append(f"cancelled: {params['sessionId']}")

        # Create notification connections
        notification_client = NotificationMockClient()
        notification_agent = NotificationMockAgent()

        client_to_agent_r, client_to_agent_w = await create_stream()
        agent_to_client_r, agent_to_client_w = await create_stream()

        agent_connection = ClientSideConnection(
            lambda agent: notification_client,
            agent_to_client_r,
            client_to_agent_w,
        )

        client_connection = AgentSideConnection(
            lambda client: notification_agent,
            client_to_agent_r,
            agent_to_client_w,
        )

        # Send notifications
        await client_connection.session_update({
            "sessionId": "test-session",
            "update": cast(schema.SessionUpdate, {
                "sessionUpdate": "agent_message_chunk",
                "content": cast(schema.ContentBlock, {
                    "type": "text",
                    "text": "Hello from agent",
                }),
            }),
        })

        await agent_connection.cancel({
            "sessionId": "test-session",
        })

        # Wait for async handlers
        await asyncio.sleep(0.05)

        # Clean up
        await agent_connection.close()
        await client_connection.close()

        # Verify notifications were received
        assert "agent message: Hello from agent" in notification_log
        assert "cancelled: test-session" in notification_log

    async def test_handles_initialize_method(self, connection_setup):
        """
        Test the protocol initialization handshake.

        Verifies that the initialize request/response works correctly
        and returns the expected agent capabilities and auth methods.
        """
        setup = connection_setup

        # Test initialize request
        response = await setup['agent_connection'].initialize({
            "protocolVersion": PROTOCOL_VERSION,
            "clientCapabilities": cast(schema.ClientCapabilities, {
                "fs": cast(schema.FileSystemCapability, {
                    "readTextFile": False,
                    "writeTextFile": False,
                }),
            }),
        })

        assert response["protocolVersion"] == PROTOCOL_VERSION
        assert response["agentCapabilities"]["loadSession"] is True
        assert len(response["authMethods"]) == 1
        assert response["authMethods"][0]["id"] == "oauth"

        # Verify agent received the call
        assert len(setup['test_agent'].initialize_calls) == 1
        assert setup['test_agent'].initialize_calls[0]["protocolVersion"] == PROTOCOL_VERSION

    async def test_basic_client_operations(self, connection_setup):
        """
        Test all basic client-side operations.

        Verifies file operations (read/write) and permission requests
        work as expected and return proper responses.
        """
        setup = connection_setup

        # Test write file
        result = await setup['client_connection'].write_text_file({
            "path": "/test.txt",
            "content": "test content",
            "sessionId": "test-session",
        })
        assert result is None
        assert len(setup['test_client'].write_file_calls) == 1
        assert setup['test_client'].write_file_calls[0]["path"] == "/test.txt"

        # Test read file
        result = await setup['client_connection'].read_text_file({
            "path": "/test.txt",
            "sessionId": "test-session",
        })
        assert result["content"] == "Content of /test.txt"
        assert len(setup['test_client'].read_file_calls) == 1

        # Test request permission
        result = await setup['client_connection'].request_permission({
            "sessionId": "test-session",
            "toolCall": cast(schema.ToolCall, {
                "title": "Test Tool",
                "kind": "execute",
                "status": "pending",
                "toolCallId": "tool-123",
                "content": [],
            }),
            "options": [
                cast(schema.PermissionOption, {
                    "kind": "allow_once",
                    "name": "Allow",
                    "optionId": "allow",
                })
            ],
        })
        assert result["outcome"]["outcome"] == "selected"
        assert result["outcome"]["optionId"] == "allow"
        assert len(setup['test_client'].permission_calls) == 1

    async def test_basic_agent_operations(self, connection_setup):
        """
        Test all basic agent-side operations.

        Verifies session management (new/load), authentication, prompts,
        and cancellation notifications work correctly.
        """
        setup = connection_setup

        # Test new session
        result = await setup['agent_connection'].new_session({
            "cwd": "/test",
            "mcpServers": [],
        })
        assert result["sessionId"] == "test-session"
        assert len(setup['test_agent'].new_session_calls) == 1

        # Test load session
        result = await setup['agent_connection'].load_session({
            "sessionId": "existing-session",
            "cwd": "/test",
            "mcpServers": [],
        })
        assert result["authRequired"] is False
        assert len(setup['test_agent'].load_session_calls) == 1

        # Test authenticate
        await setup['agent_connection'].authenticate({
            "methodId": "oauth",
        })
        assert len(setup['test_agent'].authenticate_calls) == 1

        # Test prompt
        await setup['agent_connection'].prompt({
            "sessionId": "test-session",
            "prompt": [
                cast(schema.ContentBlock, {
                    "type": "text",
                    "text": "Hello",
                })
            ],
        })
        assert len(setup['test_agent'].prompt_calls) == 1

        # Test cancel notification - need to wait a bit for async notification handling
        await setup['agent_connection'].cancel({
            "sessionId": "test-session",
        })
        await asyncio.sleep(0.01)  # Give time for notification to be processed
        assert len(setup['test_agent'].cancel_calls) == 1
