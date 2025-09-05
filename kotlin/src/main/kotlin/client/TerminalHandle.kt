package client

import transport.Connection
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import schema.ClientMethods
import schema.TerminalOutputResponse
import schema.WaitForTerminalExitResponse
import util.UnitSerializer

class TerminalHandle(
    val id: String,
    private val sessionId: String,
    private val conn: Connection
) {
    suspend fun currentOutput(): TerminalOutputResponse =
        conn.sendRequest(
            ClientMethods.terminal_output,
            buildJsonObject { put("sessionId", sessionId); put("terminalId", id) },
            TerminalOutputResponse.serializer()
        )

    suspend fun waitForExit(): WaitForTerminalExitResponse =
        conn.sendRequest(
            ClientMethods.terminal_wait_for_exit,
            buildJsonObject { put("sessionId", sessionId); put("terminalId", id) },
            WaitForTerminalExitResponse.serializer()
        )

    suspend fun kill() {
        conn.sendRequest<Unit>(
            ClientMethods.terminal_kill,
            buildJsonObject { put("sessionId", sessionId); put("terminalId", id) },
            UnitSerializer
        )
    }

    suspend fun release() {
        conn.sendRequest<Unit>(
            ClientMethods.terminal_release,
            buildJsonObject { put("sessionId", sessionId); put("terminalId", id) },
            UnitSerializer
        )
    }
}