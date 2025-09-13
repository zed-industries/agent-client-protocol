package transport

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.encodeToJsonElement
import kotlinx.serialization.json.put

class RequestError(val code: Int, message: String, val data: JsonElement? = null) : RuntimeException(message) {
    companion object {
        fun parseError(data: Any? = null) = RequestError(-32700, "Parse error", toJson(data))
        fun invalidRequest(data: Any? = null) = RequestError(-32600, "Invalid request", toJson(data))
        fun methodNotFound(method: String) = RequestError(-32601, "Method not found", buildJsonObject { put("method", method) })
        fun invalidParams(data: Any? = null) = RequestError(-32602, "Invalid params", toJson(data))
        fun internalError(data: Any? = null) = RequestError(-32603, "Internal error", toJson(data))
        fun authRequired(data: Any? = null) = RequestError(-32000, "Authentication required", toJson(data))
        private fun toJson(data: Any?): JsonElement? = when (data) {
            null -> null
            is JsonElement -> data
            is String -> buildJsonObject { put("details", data) }
            else -> Json.encodeToJsonElement(data)
        }
    }
}
