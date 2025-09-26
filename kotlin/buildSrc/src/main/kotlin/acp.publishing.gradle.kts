plugins {
    `maven-publish`
    signing
}

publishing {
    publications {
        withType<MavenPublication> {
            pom {
                name.set("ACP Kotlin SDK")
                description.set("Kotlin implementation of the Agent Client Protocol (ACP)")
                url.set("https://github.com/agentclientprotocol/agent-client-protocol")
                
                licenses {
                    license {
                        name.set("MIT")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                }
                
                developers {
                    developer {
                        id.set("acp")
                        name.set("Agent Client Protocol Team")
                        email.set("team@agentclientprotocol.com")
                    }
                }
                
                scm {
                    connection.set("scm:git:git://github.com/agentclientprotocol/agent-client-protocol.git")
                    developerConnection.set("scm:git:ssh://github.com/agentclientprotocol/agent-client-protocol.git")
                    url.set("https://github.com/agentclientprotocol/agent-client-protocol")
                }
            }
        }
    }
}

signing {
    useGpgCmd()
    sign(publishing.publications)
}