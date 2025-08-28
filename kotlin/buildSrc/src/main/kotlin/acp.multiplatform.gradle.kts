import org.jetbrains.kotlin.gradle.dsl.ExplicitApiMode
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    kotlin("multiplatform")
    kotlin("plugin.serialization")
}

// Generation library versions
val generateLibVersion by tasks.registering {
    val outputDir = layout.buildDirectory.dir("generated-sources/libVersion")
    outputs.dir(outputDir)

    doLast {
        val sourceFile = outputDir.get().file("io/agentclientprotocol/kotlin/LibVersion.kt").asFile
        sourceFile.parentFile.mkdirs()
        sourceFile.writeText(
            """
            package io.agentclientprotocol

            public const val LIB_VERSION: String = "${project.version}"
            
            """.trimIndent()
        )
    }
}

kotlin {
    jvm {
        compilerOptions.jvmTarget = JvmTarget.JVM_1_8
    }
    // Future multiplatform targets can be added here without changing the code
    // js { nodejs() }
    // wasmJs { nodejs() }
    // linuxX64(); macosX64(); mingwX64()

    explicitApi = ExplicitApiMode.Strict
    jvmToolchain(21)

    sourceSets {
        commonMain {
            kotlin.srcDir(generateLibVersion)
        }
    }
}