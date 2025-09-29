@file:Suppress("unused")

package io.agentclientprotocol.model

/**
 * Base interface for ACP method enums.
 */
public sealed interface AcpMethod {
    public val methodName: String

    /**
     * ACP method names for agent-side operations.
     *
     * These are methods that agents can call on clients.
     */
    public enum class AgentMethods(override val methodName: String) : AcpMethod {
        Initialize("initialize"),
        Authenticate("authenticate"),
        SessionNew("session/new"),
        SessionLoad("session/load"),
        SessionPrompt("session/prompt"),
        SessionCancel("session/cancel"),
        SessionSetMode("session/set_mode")
    }

    /**
     * ACP method names for client-side operations.
     *
     * These are methods that clients can call on agents.
     */
    public enum class ClientMethods(override val methodName: String) : AcpMethod {
        FsReadTextFile("fs/read_text_file"),
        FsWriteTextFile("fs/write_text_file"),
        SessionRequestPermission("session/request_permission"),
        SessionUpdate("session/update"),
        TerminalCreate("terminal/create"),
        TerminalOutput("terminal/output"),
        TerminalRelease("terminal/release"),
        TerminalWaitForExit("terminal/wait_for_exit"),
        TerminalKill("terminal/kill")
    }

    public class UnknownMethod(override val methodName: String) : AcpMethod
}
