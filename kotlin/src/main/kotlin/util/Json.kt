package util

import kotlinx.serialization.KSerializer
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNull

internal val json: Json = Json {
    ignoreUnknownKeys = true
    isLenient = true
    encodeDefaults = true
}

internal object UnitSerializer : KSerializer<Unit> {
    override val descriptor = JsonNull.serializer().descriptor
    override fun deserialize(decoder: Decoder) = Unit
    override fun serialize(encoder: Encoder, value: Unit) = encoder.encodeNull()
}
