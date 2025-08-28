package io.agentclientprotocol.samples.client

import io.agentclientprotocol.transport.StdioTransport
import io.agentclientprotocol.transport.Transport
import kotlinx.io.asSink
import kotlinx.io.asSource
import kotlinx.io.buffered

/**
 * Creates a Transport that wraps a StdioTransport connected to an external process.
 * 
 * Convenience overload that accepts a vararg of command parts.
 * 
 * @param command The command to execute as varargs (command and arguments)
 * @return A Transport connected to the process's stdin/stdout
 */
fun createProcessStdioTransport(vararg command: String): Transport {
    val process = ProcessBuilder(*command)
        .redirectInput(ProcessBuilder.Redirect.PIPE)
        .redirectOutput(ProcessBuilder.Redirect.PIPE)
        .redirectError(ProcessBuilder.Redirect.PIPE)
        .start()
    val stdin = process.outputStream.asSink().buffered()
    val stdout = process.inputStream.asSource().buffered()
    return StdioTransport(input = stdout, output = stdin)
}