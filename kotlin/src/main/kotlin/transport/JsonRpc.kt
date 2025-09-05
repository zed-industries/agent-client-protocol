package transport

import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.*
import util.json

internal fun jsonRpcRequest(id: Long, method: String, params: JsonElement?) = buildJsonObject {
    put("jsonrpc", "2.0"); put("id", id); put("method", method); params?.let { put("params", it) }
}

internal fun jsonRpcNotification(method: String, params: JsonElement?) = buildJsonObject {
    put("jsonrpc", "2.0"); put("method", method); params?.let { put("params", it) }
}

internal fun jsonRpcResult(id: JsonElement, result: JsonElement) = buildJsonObject {
    put("jsonrpc", "2.0"); put("id", id); put("result", result)
}

internal fun jsonRpcError(id: JsonElement, err: RequestError) = buildJsonObject {
    put("jsonrpc", "2.0"); put("id", id)
    putJsonObject("error") {
        put("code", err.code); put("message", err.message); err.data?.let { put("data", it) }
    }
}

internal fun <T> requireParams(el: JsonElement?, ser: KSerializer<T>): T {
    if (el == null) throw RequestError.invalidParams("missing params")
    return json.decodeFromJsonElement(ser, el)
}

internal fun decodeError(el: JsonElement): RequestError {
    val obj = el as JsonObject
    val code = obj["code"]!!.jsonPrimitive.int
    val msg = obj["message"]!!.jsonPrimitive.content
    val data = obj["data"]
    return RequestError(code, msg, data)
}
