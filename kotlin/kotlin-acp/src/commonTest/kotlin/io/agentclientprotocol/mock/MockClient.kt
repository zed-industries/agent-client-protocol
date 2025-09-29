package io.agentclientprotocol.mock

import io.agentclientprotocol.client.Client
import io.agentclientprotocol.model.CreateTerminalRequest
import io.agentclientprotocol.model.CreateTerminalResponse
import io.agentclientprotocol.model.KillTerminalCommandRequest
import io.agentclientprotocol.model.KillTerminalCommandResponse
import io.agentclientprotocol.model.PermissionOptionId
import io.agentclientprotocol.model.ReadTextFileRequest
import io.agentclientprotocol.model.ReadTextFileResponse
import io.agentclientprotocol.model.ReleaseTerminalRequest
import io.agentclientprotocol.model.ReleaseTerminalResponse
import io.agentclientprotocol.model.RequestPermissionOutcome
import io.agentclientprotocol.model.RequestPermissionRequest
import io.agentclientprotocol.model.RequestPermissionResponse
import io.agentclientprotocol.model.SessionNotification
import io.agentclientprotocol.model.TerminalOutputRequest
import io.agentclientprotocol.model.TerminalOutputResponse
import io.agentclientprotocol.model.WaitForTerminalExitRequest
import io.agentclientprotocol.model.WaitForTerminalExitResponse
import io.agentclientprotocol.model.WriteTextFileRequest
import io.agentclientprotocol.model.WriteTextFileResponse

class MockClient : Client {
    var readTextFileResult: ReadTextFileResponse = ReadTextFileResponse("test content")
    var requestPermissionResult: RequestPermissionResponse = RequestPermissionResponse(
        RequestPermissionOutcome.Selected(PermissionOptionId("allow"))
    )
    var createTerminalResult: CreateTerminalResponse = CreateTerminalResponse("mock-terminal")
    var terminalOutputResult: TerminalOutputResponse = TerminalOutputResponse("", false, null)
    var releaseTerminalResult: ReleaseTerminalResponse = ReleaseTerminalResponse()
    var waitForTerminalExitResult: WaitForTerminalExitResponse = WaitForTerminalExitResponse(0u, null)
    var killTerminalResult: KillTerminalCommandResponse = KillTerminalCommandResponse()

    val readTextFileCalls = mutableListOf<ReadTextFileRequest>()
    val writeTextFileCalls = mutableListOf<WriteTextFileRequest>()
    val requestPermissionCalls = mutableListOf<RequestPermissionRequest>()
    val sessionUpdateCalls = mutableListOf<SessionNotification>()
    val terminalCreateCalls = mutableListOf<CreateTerminalRequest>()
    val terminalOutputCalls = mutableListOf<TerminalOutputRequest>()
    val terminalReleaseCalls = mutableListOf<ReleaseTerminalRequest>()
    val terminalWaitForExitCalls = mutableListOf<WaitForTerminalExitRequest>()
    val terminalKillCalls = mutableListOf<KillTerminalCommandRequest>()

    override suspend fun fsReadTextFile(request: ReadTextFileRequest): ReadTextFileResponse {
        readTextFileCalls.add(request)
        return readTextFileResult
    }

    override suspend fun fsWriteTextFile(request: WriteTextFileRequest): WriteTextFileResponse {
        writeTextFileCalls.add(request)
        return WriteTextFileResponse()
    }

    override suspend fun sessionRequestPermission(request: RequestPermissionRequest): RequestPermissionResponse {
        requestPermissionCalls.add(request)
        return requestPermissionResult
    }

    override suspend fun sessionUpdate(notification: SessionNotification) {
        sessionUpdateCalls.add(notification)
    }

    override suspend fun terminalCreate(request: CreateTerminalRequest): CreateTerminalResponse {
        terminalCreateCalls.add(request)
        return createTerminalResult
    }

    override suspend fun terminalOutput(request: TerminalOutputRequest): TerminalOutputResponse {
        terminalOutputCalls.add(request)
        return terminalOutputResult
    }

    override suspend fun terminalRelease(request: ReleaseTerminalRequest): ReleaseTerminalResponse {
        terminalReleaseCalls.add(request)
        return releaseTerminalResult
    }

    override suspend fun terminalWaitForExit(request: WaitForTerminalExitRequest): WaitForTerminalExitResponse {
        terminalWaitForExitCalls.add(request)
        return waitForTerminalExitResult
    }

    override suspend fun terminalKill(request: KillTerminalCommandRequest): KillTerminalCommandResponse {
        terminalKillCalls.add(request)
        return killTerminalResult
    }

    fun reset() {
        readTextFileCalls.clear()
        writeTextFileCalls.clear()
        requestPermissionCalls.clear()
        sessionUpdateCalls.clear()
        terminalCreateCalls.clear()
        terminalOutputCalls.clear()
        terminalReleaseCalls.clear()
        terminalWaitForExitCalls.clear()
        terminalKillCalls.clear()
    }
}