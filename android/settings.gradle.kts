pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.10.0"
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "CrypNotesAndroid"

include(
    ":app",
    ":core:bridge",
    ":core:data",
    ":core:platform:security",
    ":core:platform:media",
    ":core:platform:notifications",
    ":core:ui",
    ":feature:notes",
    ":feature:labels",
    ":feature:reminders",
    ":feature:vault",
    ":feature:settings"
)