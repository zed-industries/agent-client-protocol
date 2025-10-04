package io.agentclientprotocol.mock

import io.agentclientprotocol.rpc.JsonRpcMessage
import io.agentclientprotocol.rpc.decodeJsonRpcMessage
import io.agentclientprotocol.transport.Transport
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive

/**
 * In-memory transport for testing ACP implementations.
 *
 * This transport allows direct communication between agent and client
 * without requiring external protocols like STDIO or WebSocket.
 */
class TestTransport : Transport {
    private val _messages = Channel<JsonRpcMessage>(Channel.Factory.UNLIMITED)

    private var _isConnected = false
    override val isConnected: Boolean get() = _isConnected

    override val messages: ReceiveChannel<JsonRpcMessage> = _messages

    /**
     * Create a connected pair of test transports for agent-client communication.
     */
    public companion object {
        public fun createPair(): Pair<TestTransport, TestTransport> {
            val transport1 = TestTransport()
            val transport2 = TestTransport()

            // Connect them together
            transport1.connectedTransport = transport2
            transport2.connectedTransport = transport1

            return transport1 to transport2
        }
    }

    private var connectedTransport: TestTransport? = null

    override fun start() {
        _isConnected = true
    }

    override fun send(message: JsonRpcMessage) {
        if (!_isConnected) {
            throw IllegalStateException("Transport is not connected")
        }

        val connected = connectedTransport
        if (connected != null && connected._isConnected) {
            // Send directly to connected transport's incoming channel
            // This needs to be non-blocking for the new interface
            if (!connected._messages.trySend(message).isSuccess) {
                throw IllegalStateException("Failed to send message - channel full or closed")
            }
        }
    }

    override fun close() {
        if (!_isConnected) return

        _isConnected = false
        _messages.close()
    }

    /**
     * Simulate receiving a message from the connected transport.
     */
    public fun receiveMessage(message: JsonRpcMessage) {
        if (!_messages.trySend(message).isSuccess) {
            throw IllegalStateException("Failed to receive message - channel full or closed")
        }
    }

    /**
     * Simulate receiving a JSON-encoded message from the connected transport.
     * Parses the JSON string and determines the message type based on the presence of fields.
     */
    public fun receiveMessage(jsonMessage: String) {
        val message = decodeJsonRpcMessage(jsonMessage)
        receiveMessage(message)
    }
}