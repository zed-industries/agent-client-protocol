package transport

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.long
import util.json
import java.io.InputStream
import java.io.OutputStream
import java.nio.charset.StandardCharsets
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong
import kotlin.collections.contains
import kotlin.collections.set

private typealias MethodHandler = suspend (method: String, params: JsonElement?) -> JsonElement?

class Connection(
    private val handler: MethodHandler,
    private val peerInput: OutputStream,
    peerOutput: InputStream
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val pending = ConcurrentHashMap<Long, CompletableDeferred<JsonElement>>()
    private val nextId = AtomicLong(0)

    init { scope.launch { receiveLoop(peerOutput) } }

    suspend fun <Resp> sendRequest(
        method: String,
        params: JsonElement?,
        respSer: KSerializer<Resp>
    ): Resp {
        val id = nextId.getAndIncrement()
        val wait = CompletableDeferred<JsonElement>()
        pending[id] = wait
        send(jsonRpcRequest(id, method, params))
        val payload = wait.await()
        return if (payload is JsonObject && "error" in payload) {
            throw payload["error"]!!.let { decodeError(it) }
        } else {
            json.decodeFromJsonElement(respSer, (payload as JsonObject)["result"]!!)
        }
    }

    suspend fun sendNotification(method: String, params: JsonElement?) {
        send(jsonRpcNotification(method, params))
    }

    private suspend fun receiveLoop(input: InputStream) {
        val decoder = StandardCharsets.UTF_8
        val buf = ByteArray(8192)
        var acc = StringBuilder()
        while (true) {
            val n = input.read(buf)
            if (n <= 0) break
            acc.append(String(buf, 0, n, decoder))
            var idx: Int
            while (true) {
                idx = acc.indexOf("\n")
                if (idx < 0) break
                val line = acc.substring(0, idx).trim()
                acc.delete(0, idx + 1)
                if (line.isEmpty()) continue
                processLine(line)
            }
        }
    }

    private suspend fun processLine(line: String) {
        val msg = runCatching { json.parseToJsonElement(line) }.getOrElse {
            // no id to echo reliably here; ignore
            return
        }
        when (msg) {
            is JsonObject -> when {
                "method" in msg && "id" in msg -> { // request
                    val id = msg["id"]!!
                    val method = msg["method"]!!.jsonPrimitive.content
                    val params = msg["params"]
                    val resultOrErr = try {
                        val res = handler(method, params)
                        jsonRpcResult(id, res ?: JsonNull)
                    } catch (e: RequestError) {
                        jsonRpcError(id, e)
                    } catch (e: Throwable) {
                        jsonRpcError(id, RequestError.internalError(e.message ?: "internal"))
                    }
                    send(resultOrErr)
                }
                "method" in msg -> { // notification
                    val method = msg["method"]!!.jsonPrimitive.content
                    val params = msg["params"]
                    runCatching { handler(method, params) }
                }
                "id" in msg -> { // response
                    val id = msg["id"]!!.jsonPrimitive.long
                    pending.remove(id)?.complete(msg)
                }
            }

            else -> {}
        }
    }

    private suspend fun send(obj: JsonObject) {
        val bytes = (json.encodeToString(JsonObject.serializer(), obj) + "\n")
            .toByteArray(StandardCharsets.UTF_8)
        synchronized(peerInput) {
            peerInput.write(bytes)
            peerInput.flush()
        }
    }
}