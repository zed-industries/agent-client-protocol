@file:Suppress("unused")

package io.agentclientprotocol.transport

import io.agentclientprotocol.rpc.JsonRpcMessage
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.flow.Flow

/**
 * Base interface for ACP transport implementations.
 *
 * Transports handle the actual communication between clients and agents,
 * supporting various protocols like STDIO, WebSocket, and SSE.
 */
public interface Transport : AutoCloseable {
    /**
     * Start the transport and begin listening for messages.
     */
    public fun start()

    /**
     * Send a message over the transport.
     * 
     * @param message The JSON-encoded message to send
     */
    public fun send(message: JsonRpcMessage)

    /**
     * Flow of incoming messages from the transport.
     * Each message is a JSON-encoded string.
     */
    public val messages: ReceiveChannel<JsonRpcMessage>

    /**
     * Whether the transport is currently connected.
     */
    public val isConnected: Boolean
}