import org.jetbrains.intellij.platform.gradle.tasks.PrepareSandboxTask

plugins {
    id("java")
    id("org.jetbrains.intellij.platform") version "2.20.1"
}

group = "io.github.protopacket"
version = "0.0.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

// The plugin is built against the IDE at the `ide` property, or a downloaded RustRover.
val ide: Provider<String> = providers.gradleProperty("ide")

dependencies {
    intellijPlatform {
        if (ide.isPresent) {
            local(ide.get())
        } else {
            rustRover("2026.1.4")
        }
        bundledPlugin("org.jetbrains.plugins.textmate")
        pluginVerifier()
    }
}

// The code is compiled by the Java that runs Gradle, for the Java 21 of the platform.
java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(JavaVersion.current().majorVersion)
    }
}

tasks.withType<JavaCompile>().configureEach {
    options.release = 21
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
}

intellijPlatform {
    buildSearchableOptions = false
    instrumentCode = false
    pluginConfiguration {
        ideaVersion {
            sinceBuild = "261.26222"
            untilBuild = provider { null }
        }
    }
    pluginVerification {
        ides {
            if (ide.isPresent) {
                local(ide.get())
            } else {
                recommended()
            }
        }
    }
}

// The TextMate bundle is read from a folder, so it ships beside the plugin's jar.
tasks.withType<PrepareSandboxTask>().configureEach {
    from(layout.projectDirectory.dir("../textmate")) {
        into(pluginName.map { "$it/textmate" })
        exclude("README.md")
    }
}
