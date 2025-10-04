@file:Suppress("unused")

package io.agentclientprotocol.model

import kotlinx.serialization.Serializable

/**
 * Terminal-related request and response types.
 * 
 * **UNSTABLE**: These types are not part of the spec yet,
 * and may be removed or changed at any point.
 */

/**
 * Request to create a new terminal session.
 */
@Serializable
public data class CreateTerminalRequest(
    val sessionId: SessionId,
    val command: String,
    val args: List<String> = emptyList(),
    val cwd: String? = null,
    val env: List<EnvVariable> = emptyList(),
    val outputByteLimit: ULong? = null
) : AcpRequest

/**
 * Response from creating a terminal session.
 */
@Serializable
public data class CreateTerminalResponse(
    val terminalId: String
) : AcpResponse

/**
 * Request to get output from a terminal.
 */
@Serializable
public data class TerminalOutputRequest(
    val sessionId: SessionId,
    val terminalId: String
) : AcpRequest

/**
 * Response containing terminal output.
 */
@Serializable
public data class TerminalOutputResponse(
    val output: String,
    val truncated: Boolean,
    val exitStatus: TerminalExitStatus? = null
) : AcpResponse

/**
 * Request to release a terminal session.
 */
@Serializable
public data class ReleaseTerminalRequest(
    val sessionId: SessionId,
    val terminalId: String
) : AcpRequest

/**
 * Request to wait for a terminal to exit.
 */
@Serializable
public data class WaitForTerminalExitRequest(
    val sessionId: SessionId,
    val terminalId: String
) : AcpRequest

/**
 * Response from waiting for terminal exit.
 */
@Serializable
public data class WaitForTerminalExitResponse(
    val exitCode: UInt? = null,
    val signal: String? = null
) : AcpResponse

/**
 * Terminal exit status information.
 */
@Serializable
public data class TerminalExitStatus(
    val exitCode: UInt? = null,
    val signal: String? = null
)

/**
 * Request to kill a terminal command without releasing the terminal.
 */
@Serializable
public data class KillTerminalCommandRequest(
    val sessionId: SessionId,
    val terminalId: String
) : AcpRequest

/**
 * Response to terminal/kill command method
 */
@Serializable
public class KillTerminalCommandResponse : AcpResponse