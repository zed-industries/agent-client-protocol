package io.agentclientprotocol.transport

import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.*
import kotlinx.io.*

/**
 * STDIO transport implementation for ACP.
 *
 * This transport communicates over standard input/output streams,
 * which is commonly used for command-line agents.
 */
public class StdioTransport(
    private val input: Source,
    private val output: Sink
) : Transport {
    
    private val messagesChannel = Channel<String>(Channel.UNLIMITED)
    private val errorsChannel = Channel<Throwable>(Channel.UNLIMITED)
    
    private var _isConnected = false
    override val isConnected: Boolean get() = _isConnected
    
    override var onClose: (() -> Unit)? = null
    
    override val messages: Flow<String> = messagesChannel.receiveAsFlow()
    override val errors: Flow<Throwable> = errorsChannel.receiveAsFlow()
    
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    
    override suspend fun start() {
        _isConnected = true
        
        // Start reading messages from input
        scope.launch {
            try {
                val buffer = Buffer()
                while (_isConnected) {
                    val line = input.readLine()
                    if (line != null) {
                        messagesChannel.send(line)
                    } else {
                        // End of stream
                        break
                    }
                }
            } catch (e: Exception) {
                errorsChannel.send(e)
            } finally {
                close()
            }
        }
    }
    
    override suspend fun send(message: String) {
        if (!_isConnected) {
            throw IllegalStateException("Transport is not connected")
        }
        
        try {
            output.writeString(message)
            output.writeString("\n")
            output.flush()
        } catch (e: Exception) {
            errorsChannel.send(e)
            throw e
        }
    }
    
    override suspend fun close() {
        if (!_isConnected) return
        
        _isConnected = false
        messagesChannel.close()
        errorsChannel.close()
        scope.cancel()
        
        try {
            input.close()
            output.close()
        } catch (e: Exception) {
            // Ignore close errors
        }
        
        onClose?.invoke()
    }
}