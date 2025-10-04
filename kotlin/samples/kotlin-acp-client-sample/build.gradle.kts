plugins {
    kotlin("jvm")
    kotlin("plugin.serialization")
    application
}

dependencies {
    implementation(project(":kotlin-acp"))
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.kotlin.logging)
    implementation(libs.kotlinx.io.core)
    implementation("ch.qos.logback:logback-classic:1.5.13")
}

application {
    mainClass.set("io.agentclientprotocol.samples.client.ClientSampleKt")
}

kotlin {
    jvmToolchain(21)
}