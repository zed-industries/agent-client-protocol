import asyncio
import json
from abc import ABC, abstractmethod
from typing import Any, Dict, Optional, Union, Callable, Awaitable
from dataclasses import dataclass
import logging
from pydantic import TypeAdapter, ValidationError

from . import schema

# Re-export all schema items
from .schema import *

logger = logging.getLogger(__name__)

class RequestError(Exception):
    """JSON-RPC request error"""

    def __init__(self, code: int, message: str, details: Optional[str] = None):
        super().__init__(message)
        self.code = code
        self.message = message
        self.data = {"details": details} if details else None

    @classmethod
    def parse_error(cls, details: Optional[str] = None) -> "RequestError":
        return cls(-32700, "Parse error", details)

    @classmethod
    def invalid_request(cls, details: Optional[str] = None) -> "RequestError":
        return cls(-32600, "Invalid request", details)

    @classmethod
    def method_not_found(cls, details: Optional[str] = None) -> "RequestError":
        return cls(-32601, "Method not found", details)

    @classmethod
    def invalid_params(cls, details: Optional[str] = None) -> "RequestError":
        return cls(-32602, "Invalid params", details)

    @classmethod
    def internal_error(cls, details: Optional[str] = None) -> "RequestError":
        return cls(-32603, "Internal error", details)

    @classmethod
    def auth_required(cls, details: Optional[str] = None) -> "RequestError":
        return cls(-32000, "Authentication required", details)

    def to_result(self) -> Dict[str, Any]:
        error_dict = {
            "code": self.code,
            "message": self.message
        }
        if self.data:
            error_dict["data"] = self.data
        return {"error": error_dict}


def validate_params(params: Any, schema_type: Any) -> Any:
    """Validate parameters using pydantic"""
    try:
        adapter = TypeAdapter(schema_type)
        return adapter.validate_python(params)
    except ValidationError as e:
        raise RequestError.invalid_params(str(e))
    except Exception as e:
        raise RequestError.invalid_params(f"Validation error: {str(e)}")


@dataclass
class PendingResponse:
    """Represents a pending JSON-RPC response"""
    future: asyncio.Future


class Client(ABC):
    """Abstract client interface"""

    @abstractmethod
    async def request_permission(
        self, params: schema.RequestPermissionRequest
    ) -> schema.RequestPermissionResponse:
        pass

    @abstractmethod
    async def session_update(self, params: schema.SessionNotification) -> None:
        pass

    @abstractmethod
    async def write_text_file(
        self, params: schema.WriteTextFileRequest
    ) -> schema.WriteTextFileResponse:
        pass

    @abstractmethod
    async def read_text_file(
        self, params: schema.ReadTextFileRequest
    ) -> schema.ReadTextFileResponse:
        pass


class Agent(ABC):
    """Abstract agent interface"""

    @abstractmethod
    async def initialize(
        self, params: schema.InitializeRequest
    ) -> schema.InitializeResponse:
        pass

    @abstractmethod
    async def new_session(
        self, params: schema.NewSessionRequest
    ) -> schema.NewSessionResponse:
        pass

    async def load_session(
        self, params: schema.LoadSessionRequest
    ) -> schema.LoadSessionResponse:
        """Optional method - agents may not implement session loading"""
        raise RequestError.method_not_found("load_session not implemented")

    @abstractmethod
    async def authenticate(self, params: schema.AuthenticateRequest) -> None:
        pass

    @abstractmethod
    async def prompt(self, params: schema.PromptRequest) -> None:
        pass

    @abstractmethod
    async def cancel(self, params: schema.CancelNotification) -> None:
        pass


MethodHandler = Callable[[str, Any], Awaitable[Any]]


class Connection:
    """Handles JSON-RPC communication over async streams"""

    def __init__(
        self,
        handler: MethodHandler,
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter
    ):
        self._handler = handler
        self._reader = reader
        self._writer = writer
        self._pending_responses: Dict[Union[str, int], PendingResponse] = {}
        self._next_request_id = 0
        self._write_lock = asyncio.Lock()

        # Start the receive task
        self._receive_task = asyncio.create_task(self._receive())

    async def close(self):
        """Close the connection and cleanup resources"""
        if not self._receive_task.done():
            self._receive_task.cancel()

        self._writer.close()
        await self._writer.wait_closed()

        # Cancel all pending responses
        for pending in self._pending_responses.values():
            if not pending.future.done():
                pending.future.cancel()

    async def _receive(self):
        """Receive and process messages from the peer"""
        buffer = ""

        try:
            while True:
                data = await self._reader.read(4096)
                if not data:
                    break

                buffer += data.decode('utf-8')

                # Process complete lines
                while '\n' in buffer:
                    line, buffer = buffer.split('\n', 1)
                    line = line.strip()

                    if line:
                        try:
                            message = json.loads(line)
                            await self._process_message(message)
                        except json.JSONDecodeError as e:
                            logger.error(f"Failed to parse JSON message: {e}")
                        except Exception as e:
                            logger.error(f"Error processing message: {e}")

        except asyncio.CancelledError:
            pass
        except Exception as e:
            logger.error(f"Error in receive loop: {e}")

    async def _process_message(self, message: Dict[str, Any]):
        """Process a received JSON-RPC message"""
        if "method" in message and "id" in message:
            # It's a request
            response = await self._try_call_handler(message["method"], message.get("params"))
            await self._send_message({
                "jsonrpc": "2.0",
                "id": message["id"],
                **response
            })
        elif "method" in message:
            # It's a notification
            await self._try_call_handler(message["method"], message.get("params"))
        elif "id" in message:
            # It's a response
            await self._handle_response(message)

    async def _try_call_handler(self, method: str, params: Any = None) -> Dict[str, Any]:
        """Try to call the method handler and return a result or error"""
        try:
            result = await self._handler(method, params)
            return {"result": result if result is not None else None}
        except RequestError as e:
            return e.to_result()
        except ValidationError as e:
            return RequestError.invalid_params(str(e)).to_result()
        except Exception as e:
            logger.error(f"Unexpected error in handler for {method}: {e}")
            return RequestError.internal_error(str(e)).to_result()

    async def _handle_response(self, response: Dict[str, Any]):
        """Handle a received response message"""
        request_id = response["id"]
        pending = self._pending_responses.pop(request_id, None)

        if pending and not pending.future.done():
            if "result" in response:
                pending.future.set_result(response["result"])
            elif "error" in response:
                error = response["error"]
                pending.future.set_exception(
                    RequestError(error["code"], error["message"],
                                 error.get("data", {}).get("details"))
                )

    async def send_request(self, method: str, params: Any = None) -> Any:
        """Send a JSON-RPC request and wait for response"""
        request_id = self._next_request_id
        self._next_request_id += 1

        future = asyncio.Future()
        self._pending_responses[request_id] = PendingResponse(future)

        await self._send_message({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        })

        return await future

    async def send_notification(self, method: str, params: Any = None) -> None:
        """Send a JSON-RPC notification"""
        await self._send_message({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        })

    async def _send_message(self, message: Dict[str, Any]):
        """Send a JSON-RPC message"""
        content = json.dumps(message) + '\n'

        async with self._write_lock:
            self._writer.write(content.encode('utf-8'))
            await self._writer.drain()


class AgentSideConnection(Client):
    """Connection from agent side - implements Client interface"""

    def __init__(
        self,
        to_agent: Callable[["AgentSideConnection"], Agent],
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter
    ):
        self._agent = to_agent(self)

        async def handler(method: str, params: Any) -> Any:
            if method == schema.AGENT_METHODS["initialize"]:
                validated_params = validate_params(
                    params, schema.InitializeRequest)
                return await self._agent.initialize(validated_params)
            elif method == schema.AGENT_METHODS["session_new"]:
                validated_params = validate_params(
                    params, schema.NewSessionRequest)
                return await self._agent.new_session(validated_params)
            elif method == schema.AGENT_METHODS["session_load"]:
                validated_params = validate_params(
                    params, schema.LoadSessionRequest)
                return await self._agent.load_session(validated_params)
            elif method == schema.AGENT_METHODS["authenticate"]:
                validated_params = validate_params(
                    params, schema.AuthenticateRequest)
                return await self._agent.authenticate(validated_params)
            elif method == schema.AGENT_METHODS["session_prompt"]:
                validated_params = validate_params(
                    params, schema.PromptRequest)
                return await self._agent.prompt(validated_params)
            elif method == schema.AGENT_METHODS["session_cancel"]:
                validated_params = validate_params(
                    params, schema.CancelNotification)
                return await self._agent.cancel(validated_params)
            else:
                raise RequestError.method_not_found(method)

        self._connection = Connection(handler, reader, writer)

    async def close(self):
        """Close the connection"""
        await self._connection.close()

    async def session_update(self, params: schema.SessionNotification) -> None:
        """Send session update notification to client"""
        await self._connection.send_notification(
            schema.CLIENT_METHODS["session_update"], params
        )

    async def request_permission(
        self, params: schema.RequestPermissionRequest
    ) -> schema.RequestPermissionResponse:
        """Request permission from client"""
        return await self._connection.send_request(
            schema.CLIENT_METHODS["session_request_permission"], params
        )

    async def read_text_file(
        self, params: schema.ReadTextFileRequest
    ) -> schema.ReadTextFileResponse:
        """Request to read a text file from client"""
        return await self._connection.send_request(
            schema.CLIENT_METHODS["fs_read_text_file"], params
        )

    async def write_text_file(
        self, params: schema.WriteTextFileRequest
    ) -> schema.WriteTextFileResponse:
        """Request to write a text file via client"""
        return await self._connection.send_request(
            schema.CLIENT_METHODS["fs_write_text_file"], params
        )


class ClientSideConnection(Agent):
    """Connection from client side - implements Agent interface"""

    def __init__(
        self,
        to_client: Callable[[Agent], Client],
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter
    ):
        async def handler(method: str, params: Any) -> Any:
            client = to_client(self)

            if method == schema.CLIENT_METHODS["fs_write_text_file"]:
                validated_params = validate_params(
                    params, schema.WriteTextFileRequest)
                return await client.write_text_file(validated_params)
            elif method == schema.CLIENT_METHODS["fs_read_text_file"]:
                validated_params = validate_params(
                    params, schema.ReadTextFileRequest)
                return await client.read_text_file(validated_params)
            elif method == schema.CLIENT_METHODS["session_request_permission"]:
                validated_params = validate_params(
                    params, schema.RequestPermissionRequest)
                return await client.request_permission(validated_params)
            elif method == schema.CLIENT_METHODS["session_update"]:
                validated_params = validate_params(
                    params, schema.SessionNotification)
                return await client.session_update(validated_params)
            else:
                raise RequestError.method_not_found(method)

        self._connection = Connection(handler, reader, writer)

    async def close(self):
        """Close the connection"""
        await self._connection.close()

    async def initialize(
        self, params: schema.InitializeRequest
    ) -> schema.InitializeResponse:
        """Initialize the agent"""
        return await self._connection.send_request(
            schema.AGENT_METHODS["initialize"], params
        )

    async def new_session(
        self, params: schema.NewSessionRequest
    ) -> schema.NewSessionResponse:
        """Create a new session"""
        return await self._connection.send_request(
            schema.AGENT_METHODS["session_new"], params
        )

    async def load_session(
        self, params: schema.LoadSessionRequest
    ) -> schema.LoadSessionResponse:
        """Load an existing session"""
        return await self._connection.send_request(
            schema.AGENT_METHODS["session_load"], params
        )

    async def authenticate(self, params: schema.AuthenticateRequest) -> None:
        """Authenticate with the agent"""
        await self._connection.send_request(
            schema.AGENT_METHODS["authenticate"], params
        )

    async def prompt(self, params: schema.PromptRequest) -> None:
        """Send a prompt to the agent"""
        await self._connection.send_request(
            schema.AGENT_METHODS["session_prompt"], params
        )

    async def cancel(self, params: schema.CancelNotification) -> None:
        """Cancel an operation"""
        await self._connection.send_notification(
            schema.AGENT_METHODS["session_cancel"], params
        )
