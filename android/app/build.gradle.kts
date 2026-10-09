plugins {
    id("com.android.application")
}

android {
    namespace = "dev.gpui.mobile.lab"
    compileSdk = 35

    defaultConfig {
        applicationId = "dev.gpui.mobile.lab"
        minSdk = 26
        // 34 keeps adjustResize semantics (35 forces edge-to-edge).
        targetSdk = 34
        versionCode = 3
        versionName = "0.1.2"
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    signingConfigs {
        // Sideload-only test app: release is signed with the local debug key so the
        // APK installs without a keystore. Not for store distribution.
        getByName("debug")
    }

    buildTypes {
        release {
            // R8 drops the unused parts of AndroidX and the Kotlin stdlib; the classes
            // Rust reaches through JNI are kept by proguard-rules.pro.
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    packaging {
        jniLibs {
            // Store the native library compressed: a smaller APK to download and push,
            // at the cost of extracting it on install (minSdk 23+ defaults to uncompressed).
            useLegacyPackaging = true
            // cargo already strips release builds; keep symbols in debug builds.
            keepDebugSymbols += listOf("**/libgpui_mobile_lab.so")
        }
    }

    sourceSets {
        // gpui-mobile's Java helpers, copied by build.sh from the pinned checkout.
        getByName("main").java.srcDir("build/generated/gpui-helpers")
    }

    lint {
        abortOnError = false
        checkReleaseBuilds = false
    }
}

dependencies {
    // Needed by gpui-mobile's Java helpers (notifications, biometric prompt, media session).
    implementation("androidx.core:core:1.12.0")
    implementation("androidx.biometric:biometric:1.1.0")
    implementation("androidx.media:media:1.7.1")
}
