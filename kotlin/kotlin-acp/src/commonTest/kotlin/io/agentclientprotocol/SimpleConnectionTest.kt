package io.agentclientprotocol

import io.agentclientprotocol.client.ClientSideConnection
import io.agentclientprotocol.mock.MockClient
import io.agentclientprotocol.mock.TestTransport
import kotlinx.coroutines.test.runTest
import kotlin.test.*

class SimpleConnectionTest {
    @Test
    fun `test basic connection`() = runTest {
        // Given
        val mockClient = MockClient()
        val (clientTransport, agentTransport) = TestTransport.createPair()
        val clientConnection = ClientSideConnection(this, clientTransport, mockClient)

        // When
        clientConnection.start()
        clientTransport.start()
        agentTransport.start()

        // Then
        assertTrue(clientTransport.isConnected)
        assertTrue(agentTransport.isConnected)

        // Cleanup
        clientTransport.close()
        agentTransport.close()
    }
}