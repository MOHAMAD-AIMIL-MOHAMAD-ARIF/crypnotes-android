pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
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