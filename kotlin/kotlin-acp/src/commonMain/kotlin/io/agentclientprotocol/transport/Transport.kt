@file:Suppress("unused")

package io.agentclientprotocol.transport

import kotlinx.coroutines.flow.Flow

/**
 * Base interface for ACP transport implementations.
 *
 * Transports handle the actual communication between clients and agents,
 * supporting various protocols like STDIO, WebSocket, and SSE.
 */
public interface Transport {
    /**
     * Start the transport and begin listening for messages.
     */
    public suspend fun start()

    /**
     * Close the transport and cleanup resources.
     */
    public suspend fun close()

    /**
     * Send a message over the transport.
     * 
     * @param message The JSON-encoded message to send
     */
    public suspend fun send(message: String)

    /**
     * Flow of incoming messages from the transport.
     * Each message is a JSON-encoded string.
     */
    public val messages: Flow<String>

    /**
     * Flow of transport errors.
     */
    public val errors: Flow<Throwable>

    /**
     * Callback invoked when the transport is closed.
     */
    public var onClose: (() -> Unit)?

    /**
     * Whether the transport is currently connected.
     */
    public val isConnected: Boolean
}