plugins {
    id("acp.multiplatform")
    id("acp.publishing")
    alias(libs.plugins.kotlinx.binary.compatibility.validator)
}

kotlin {
    sourceSets {
        commonMain {
            dependencies {
                api(libs.kotlinx.serialization.json)
                api(libs.kotlinx.coroutines.core)
                api(libs.kotlinx.io.core)
                api(libs.kotlinx.collections.immutable)
                api(libs.kotlin.logging)
            }
        }

        commonTest {
            dependencies {
                implementation(kotlin("test"))
                implementation(libs.kotest.assertions.json)
            }
        }
    }
}