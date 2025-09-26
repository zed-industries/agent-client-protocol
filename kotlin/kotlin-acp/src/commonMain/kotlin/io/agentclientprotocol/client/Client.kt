package io.agentclientprotocol.client

import io.agentclientprotocol.model.ReadTextFileRequest
import io.agentclientprotocol.model.ReadTextFileResponse
import io.agentclientprotocol.model.RequestPermissionRequest
import io.agentclientprotocol.model.RequestPermissionResponse
import io.agentclientprotocol.model.SessionNotification
import io.agentclientprotocol.model.WriteTextFileRequest

/**
 * Interface that clients must implement to handle agent requests.
 *
 * This interface defines the contract for client implementations,
 * covering file system operations, permission handling, and session updates.
 *
 * See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
 */
public interface Client {
    /**
     * Read content from a text file in the client's file system.
     *
     * Only called if the client advertises the `fs.readTextFile` capability.
     * The client should validate the path and ensure it's within allowed boundaries.
     *
     * @param request The file read request containing path and optional line/limit
     * @return The file contents
     */
    public suspend fun readTextFile(request: ReadTextFileRequest): ReadTextFileResponse

    /**
     * Write content to a text file in the client's file system.
     *
     * Only called if the client advertises the `fs.writeTextFile` capability.
     * The client should validate the path and ensure it's within allowed boundaries.
     *
     * @param request The file write request containing path and content
     */
    public suspend fun writeTextFile(request: WriteTextFileRequest)

    /**
     * Request permission from the user for a tool call operation.
     *
     * The client should present the permission options to the user and return their choice.
     * This is called when the agent needs authorization for potentially sensitive operations.
     *
     * @param request The permission request with tool call details and options
     * @return The user's permission decision
     */
    public suspend fun requestPermission(request: RequestPermissionRequest): RequestPermissionResponse

    /**
     * Handle session update notifications from the agent.
     *
     * This is a notification method (no response expected) that receives
     * real-time updates about session progress, including message chunks,
     * tool calls, and execution plans.
     *
     * @param notification The session update notification
     */
    public suspend fun sessionUpdate(notification: SessionNotification)
}