@file:Suppress("unused")
@file:OptIn(ExperimentalSerializationApi::class)

package io.agentclientprotocol.model

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonClassDiscriminator

/**
 * Content blocks represent displayable information in the Agent Client Protocol.
 *
 * They provide a structured way to handle various types of user-facing content—whether
 * it's text from language models, images for analysis, or embedded resources for context.
 *
 * See protocol docs: [Content](https://agentclientprotocol.com/protocol/content)
 */
@Serializable
@JsonClassDiscriminator("type")
public sealed class ContentBlock {
    public abstract val annotations: Annotations?
    
    /**
     * Plain text content
     *
     * All agents MUST support text content blocks in prompts.
     */
    @Serializable
    @SerialName("text")
    public data class Text(
        val text: String,
        override val annotations: Annotations? = null
    ) : ContentBlock()

    /**
     * Images for visual context or analysis.
     *
     * Requires the `image` prompt capability when included in prompts.
     */
    @Serializable
    @SerialName("image")
    public data class Image(
        val data: String,
        val mimeType: String,
        val uri: String? = null,
        override val annotations: Annotations? = null
    ) : ContentBlock()

    /**
     * Audio data for transcription or analysis.
     *
     * Requires the `audio` prompt capability when included in prompts.
     */
    @Serializable
    @SerialName("audio")
    public data class Audio(
        val data: String,
        val mimeType: String,
        override val annotations: Annotations? = null
    ) : ContentBlock()

    /**
     * References to resources that the agent can access.
     *
     * All agents MUST support resource links in prompts.
     */
    @Serializable
    @SerialName("resource_link")
    public data class ResourceLink(
        val name: String,
        val uri: String,
        val description: String? = null,
        val mimeType: String? = null,
        val size: Long? = null,
        val title: String? = null,
        override val annotations: Annotations? = null
    ) : ContentBlock()

    /**
     * Complete resource contents embedded directly in the message.
     *
     * Preferred for including context as it avoids extra round-trips.
     *
     * Requires the `embeddedContext` prompt capability when included in prompts.
     */
    @Serializable
    @SerialName("resource")
    public data class Resource(
        val resource: EmbeddedResourceResource,
        override val annotations: Annotations? = null
    ) : ContentBlock()
}

/**
 * Resource content that can be embedded in a message.
 */
@Serializable
public sealed class EmbeddedResourceResource {
    /**
     * Text-based resource contents.
     */
    @Serializable
    @SerialName("TextResourceContents")
    public data class TextResourceContents(
        val text: String,
        val uri: String,
        val mimeType: String? = null
    ) : EmbeddedResourceResource()

    /**
     * Binary resource contents.
     */
    @Serializable
    @SerialName("BlobResourceContents")
    public data class BlobResourceContents(
        val blob: String,
        val uri: String,
        val mimeType: String? = null
    ) : EmbeddedResourceResource()
}

/**
 * The contents of a resource, embedded into a prompt or tool call result.
 */
@Serializable
public data class EmbeddedResource(
    val resource: EmbeddedResourceResource,
    val annotations: Annotations? = null
)

/**
 * A resource that the server is capable of reading, included in a prompt or tool call result.
 */
@Serializable
public data class ResourceLink(
    val name: String,
    val uri: String,
    val description: String? = null,
    val mimeType: String? = null,
    val size: Long? = null,
    val title: String? = null,
    val annotations: Annotations? = null
)