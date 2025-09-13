package client

import schema.CreateTerminalRequest
import schema.CreateTerminalResponse
import schema.KillTerminalRequest
import schema.ReadTextFileRequest
import schema.ReadTextFileResponse
import schema.ReleaseTerminalRequest
import schema.RequestPermissionRequest
import schema.RequestPermissionResponse
import schema.SessionNotification
import schema.TerminalOutputRequest
import schema.TerminalOutputResponse
import schema.WaitForTerminalExitRequest
import schema.WaitForTerminalExitResponse
import schema.WriteTextFileRequest
import transport.RequestError

/**
 * The Client interface defines the interface that ACP-compliant clients must implement.
 *
 * Clients are typically code editors (IDEs, text editors) that provide the interface
 * between users and AI agents. They manage the environment, handle user interactions,
 * and control access to resources.
 */
interface Client {
    /**
     * Requests permission from the user for a tool call operation.
     *
     * Called by the agent when it needs user authorization before executing
     * a potentially sensitive operation. The client should present the options
     * to the user and return their decision.
     *
     * If the client cancels the prompt turn via `session/cancel`, it MUST
     * respond to this request with `RequestPermissionOutcome::Cancelled`.
     *
     * See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
     */
    suspend fun requestPermission(params: RequestPermissionRequest): RequestPermissionResponse

    /**
     * Handles session update notifications from the agent.
     *
     * This is a notification endpoint (no response expected) that receives
     * real-time updates about session progress, including message chunks,
     * tool calls, and execution plans.
     *
     * Note: Clients SHOULD continue accepting tool call updates even after
     * sending a `session/cancel` notification, as the agent may send final
     * updates before responding with the cancelled stop reason.
     *
     * See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
     */
    suspend fun sessionUpdate(params: SessionNotification)

    /**
     * Writes content to a text file in the client's file system.
     *
     * Only available if the client advertises the `fs.writeTextFile` capability.
     * Allows the agent to create or modify files within the client's environment.
     *
     * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
     */
    suspend fun writeTextFile(params: WriteTextFileRequest)

    /**
     * Reads content from a text file in the client's file system.
     *
     * Only available if the client advertises the `fs.readTextFile` capability.
     * Allows the agent to access file contents within the client's environment.
     *
     * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
     */
    suspend fun readTextFile(params: ReadTextFileRequest): ReadTextFileResponse

    /**
     *  @internal **UNSTABLE**
     *
     * This method is not part of the spec, and may be removed or changed at any point.
     */
    suspend fun createTerminal(params: CreateTerminalRequest): CreateTerminalResponse =
        throw RequestError.methodNotFound("terminal/create")

    /**
     *  @internal **UNSTABLE**
     *
     * This method is not part of the spec, and may be removed or changed at any point.
     */
    suspend fun terminalOutput(params: TerminalOutputRequest): TerminalOutputResponse =
        throw RequestError.methodNotFound("terminal/output")

    /**
     *  @internal **UNSTABLE**
     *
     * This method is not part of the spec, and may be removed or changed at any point.
     */
    suspend fun releaseTerminal(params: ReleaseTerminalRequest): Nothing =
        throw RequestError.methodNotFound("terminal/release")

    /**
     *  @internal **UNSTABLE**
     *
     * This method is not part of the spec, and may be removed or changed at any point.
     */
    suspend fun waitForTerminalExit(params: WaitForTerminalExitRequest): WaitForTerminalExitResponse =
        throw RequestError.methodNotFound("terminal/waitForExit")

    /**
     *  @internal **UNSTABLE**
     *
     * This method is not part of the spec, and may be removed or changed at any point.
     */
    suspend fun killTerminal(params: KillTerminalRequest): Nothing = throw RequestError.methodNotFound("terminal/kill")
}