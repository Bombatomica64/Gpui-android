// The Rust library is built by ../build.sh (cargo-ndk) into app/src/main/jniLibs
// before Gradle packages it; Gradle itself never invokes cargo.
buildscript {
    repositories {
        google()
        mavenCentral()
    }
    dependencies {
        classpath("com.android.tools.build:gradle:9.1.0")
    }
}
