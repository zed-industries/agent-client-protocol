package transport

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
internal data class ErrorResponse(
    val code: Int,
    val message: String,
    val data: JsonElement? = null
)
