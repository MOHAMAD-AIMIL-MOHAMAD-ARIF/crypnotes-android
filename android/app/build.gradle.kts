plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
}

android {
    namespace = "com.crypnotes.app"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.crypnotes"
        minSdk = 28
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)

    implementation(project(":core:bridge"))
    implementation(project(":core:data"))
    implementation(project(":core:platform:security"))
    implementation(project(":core:platform:media"))
    implementation(project(":core:platform:notifications"))
    implementation(project(":core:ui"))
    implementation(project(":feature:notes"))
    implementation(project(":feature:labels"))
    implementation(project(":feature:reminders"))
    implementation(project(":feature:vault"))
    implementation(project(":feature:settings"))
}