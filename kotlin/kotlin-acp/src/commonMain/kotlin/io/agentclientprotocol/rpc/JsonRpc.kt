@file:Suppress("unused")

package io.agentclientprotocol.rpc

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement
import kotlin.jvm.JvmInline

/**
 * JSON-RPC version constant.
 */
public const val JSONRPC_VERSION: String = "2.0"

/**
 * Request ID for JSON-RPC messages.
 */
@JvmInline
@Serializable
public value class RequestId(public val value: String) {
    override fun toString(): String = value
}

/**
 * JSON-RPC request message.
 */
@Serializable
public data class JsonRpcRequest(
    val jsonrpc: String = JSONRPC_VERSION,
    val id: RequestId,
    val method: String,
    val params: JsonElement? = null
)

/**
 * JSON-RPC notification message.
 */
@Serializable
public data class JsonRpcNotification(
    val jsonrpc: String = JSONRPC_VERSION,
    val method: String,
    val params: JsonElement? = null
)

/**
 * JSON-RPC response message.
 */
@Serializable
public data class JsonRpcResponse(
    val jsonrpc: String = JSONRPC_VERSION,
    val id: RequestId,
    val result: JsonElement? = null,
    val error: JsonRpcError? = null
)

/**
 * JSON-RPC error object.
 */
@Serializable
public data class JsonRpcError(
    val code: Int,
    val message: String,
    val data: JsonElement? = null
)

/**
 * Standard JSON-RPC error codes.
 */
public object JsonRpcErrorCode {
    public const val PARSE_ERROR: Int = -32700
    public const val INVALID_REQUEST: Int = -32600
    public const val METHOD_NOT_FOUND: Int = -32601
    public const val INVALID_PARAMS: Int = -32602
    public const val INTERNAL_ERROR: Int = -32603
}