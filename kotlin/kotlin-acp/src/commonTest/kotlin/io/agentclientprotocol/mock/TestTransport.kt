package io.agentclientprotocol.mock

import io.agentclientprotocol.rpc.JsonRpcMessage
import io.agentclientprotocol.transport.Transport
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow

/**
 * In-memory transport for testing ACP implementations.
 *
 * This transport allows direct communication between agent and client
 * without requiring external protocols like STDIO or WebSocket.
 */
class TestTransport : Transport {
    private val incomingMessages = Channel<String>(Channel.Factory.UNLIMITED)
    private val outgoingMessages = Channel<String>(Channel.Factory.UNLIMITED)
    private val transportErrors = Channel<Throwable>(Channel.Factory.UNLIMITED)

    private var _isConnected = false
    override val isConnected: Boolean get() = _isConnected

    override var onClose: (() -> Unit)? = null

    override val messages: ReceiveChannel<JsonRpcMessage> = incomingMessages.receiveAsFlow()
    override val errors: Flow<Throwable> = transportErrors.receiveAsFlow()

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

    override suspend fun start() {
        _isConnected = true
    }

    override suspend fun send(message: JsonRpcMessage) {
        if (!_isConnected) {
            throw IllegalStateException("Transport is not connected")
        }

        val connected = connectedTransport
        if (connected != null && connected._isConnected) {
            connected.incomingMessages.send(message)
        }
    }

    override suspend fun close() {
        if (!_isConnected) return

        _isConnected = false
        incomingMessages.close()
        outgoingMessages.close()
        transportErrors.close()

        onClose?.invoke()
    }

    /**
     * Simulate receiving a message from the connected transport.
     */
    public suspend fun receiveMessage(message: String) {
        incomingMessages.send(message)
    }

    /**
     * Simulate a transport error.
     */
    public suspend fun simulateError(error: Throwable) {
        transportErrors.send(error)
    }
}