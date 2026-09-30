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
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    packaging {
        jniLibs {
            // cargo already strips release builds; keep symbols in debug builds.
            keepDebugSymbols += listOf("**/libgpui_mobile_lab.so")
        }
    }

    lint {
        abortOnError = false
        checkReleaseBuilds = false
    }
}
