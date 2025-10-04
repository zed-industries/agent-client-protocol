package io.agentclientprotocol.util

import kotlinx.coroutines.Dispatchers

public actual val DispatcherIO: kotlinx.coroutines.CoroutineDispatcher
    get() = Dispatchers.IO