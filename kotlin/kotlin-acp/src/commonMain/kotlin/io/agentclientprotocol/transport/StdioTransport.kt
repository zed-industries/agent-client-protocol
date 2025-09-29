package io.agentclientprotocol.transport

import io.agentclientprotocol.rpc.ACPJson
import io.agentclientprotocol.rpc.JsonRpcMessage
import io.agentclientprotocol.util.DispatcherIO
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.atomicfu.atomic
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.io.Sink
import kotlinx.io.Source
import kotlinx.io.readLine
import kotlinx.io.writeString
import kotlinx.serialization.encodeToString

private val logger = KotlinLogging.logger {}

/**
 * STDIO transport implementation for ACP.
 *
 * This transport communicates over standard input/output streams,
 * which is commonly used for command-line agents.
 */
public class StdioTransport(
    private val parentScope: CoroutineScope,
    private val input: Source,
    private val output: Sink
) : Transport {
    private val childScope = CoroutineScope(parentScope.coroutineContext + CoroutineName(::StdioTransport.name) + SupervisorJob(parentScope.coroutineContext[Job]))
    private val receiveChannel = Channel<JsonRpcMessage>(Channel.UNLIMITED)
    private val sendChannel = Channel<JsonRpcMessage>(Channel.UNLIMITED)
    
    private val _isConnected = atomic(false)
    override val isConnected: Boolean get() = _isConnected.value
    
    override val messages: ReceiveChannel<JsonRpcMessage> = receiveChannel
    
    override fun start() {
        // TODO handle state properly
        _isConnected.value = true
        // Start reading messages from input
        childScope.launch(DispatcherIO + CoroutineName("${::StdioTransport.name}.send")) {
            try {
                while (_isConnected.value) {
                    // ACP assumes working with ND Json (new line delimited Json) when working over stdio
                    val line = try {
                        input.readLine()
                    } catch (e: IllegalStateException) {
                        logger.trace(e) { "Input stream closed" }
                        break
                    }
                    if (line == null) {
                        // End of stream
                        logger.trace { "End of stream" }
                        break
                    }

                    val jsonRpcMessage = try {
                        ACPJson.decodeFromString<JsonRpcMessage>(line)
                    } catch (t: Throwable) {
                        logger.error(t) { "Failed to decode JSON message: $line" }
                        continue
                    }
                    logger.trace { "Sending message to channel: $jsonRpcMessage" }
                    receiveChannel.send(jsonRpcMessage)
                }
            } catch (ce: CancellationException) {
                logger.trace(ce) { "Input read cancelled" }
                throw ce
            } catch (e: Exception) {
                logger.error(e) { "Failed to read from input stream" }
                closeChannels(e)
            } finally {
                closeChannels()
                input.close()
            }
        }
        childScope.launch(DispatcherIO + CoroutineName("${::StdioTransport.name}.receive")) {
            try {
                for (message in sendChannel) {
                    val encoded = ACPJson.encodeToString(message)
                    try {
                        output.writeString(encoded)
                        output.writeString("\n")
                        output.flush()
                    } catch (e: IllegalStateException) {
                        logger.trace(e) { "Output stream closed" }
                        break
                    }
                }
            }catch (ce: CancellationException) {
                logger.trace(ce) { "Output write cancelled" }
                throw ce
            } catch (e: Throwable) {
                closeChannels(e)
                logger.error(e) { "Failed to write to output stream" }
            } finally {
                closeChannels()
                output.close()
            }
        }
    }
    
    override fun send(message: JsonRpcMessage) {
        logger.trace { "Sending message: $message" }
        val channelResult = sendChannel.trySend(message)
        logger.trace { "Send result: $channelResult" }
    }

    override fun close() {
        if (!_isConnected.getAndSet(false)) {
            logger.trace { "Transport is already closed" }
            return
        }

        closeChannels()

        try {
            input.close()
            output.close()
        } catch (e: Throwable) {
            logger.trace(e) { "Exception when closing input/output streams" }
        }
    }

    private fun closeChannels(t: Throwable? = null) {
        if (sendChannel.close(t)) logger.trace(t) { "Send channel closed" }
        if (receiveChannel.close(t)) logger.trace(t) { "Receive channel closed" }
    }
}