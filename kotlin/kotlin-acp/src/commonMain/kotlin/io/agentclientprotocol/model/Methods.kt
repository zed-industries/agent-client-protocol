@file:Suppress("unused")

package io.agentclientprotocol.model

/**
 * ACP method names for agent-side operations.
 * 
 * These are methods that agents can call on clients.
 */
public object AgentMethods {
    public const val INITIALIZE: String = "initialize"
    public const val AUTHENTICATE: String = "authenticate"
    public const val SESSION_NEW: String = "session/new"
    public const val SESSION_LOAD: String = "session/load"
    public const val SESSION_PROMPT: String = "session/prompt"
    public const val SESSION_CANCEL: String = "session/cancel"
    public const val SESSION_SET_MODE: String = "session/set_mode"
}

/**
 * ACP method names for client-side operations.
 * 
 * These are methods that clients can call on agents.
 */
public object ClientMethods {
    public const val FS_READ_TEXT_FILE: String = "fs/read_text_file"
    public const val FS_WRITE_TEXT_FILE: String = "fs/write_text_file"
    public const val SESSION_REQUEST_PERMISSION: String = "session/request_permission"
    public const val SESSION_UPDATE: String = "session/update"
    public const val TERMINAL_CREATE: String = "terminal/create"
    public const val TERMINAL_OUTPUT: String = "terminal/output"
    public const val TERMINAL_RELEASE: String = "terminal/release"
    public const val TERMINAL_WAIT_FOR_EXIT: String = "terminal/wait_for_exit"
    public const val TERMINAL_KILL: String = "terminal/kill"
}