# Apex for Android

Native Kotlin + Jetpack Compose app. Design and roadmap: [`docs/mobile/architecture.md`](../../docs/mobile/architecture.md).

- applicationId: `com.apex.kubemanager.mobile`
- minSdk 28, compile/target SDK 37
- Gradle 9.6.0 (wrapper), AGP 9.4.1, Kotlin 2.2.10, JDK 17

## Layout

| Module | What |
|---|---|
| `:app` | Compose app (placeholder triage screen) |
| `:core:triage` | Pure Kotlin/JVM triage model and rules, plain JUnit tests |

Versions live in `gradle/libs.versions.toml`.

## Build locally

Needs JDK 17+ (Android Studio's bundled JBR works) and the Android SDK. Point Gradle at the SDK with `ANDROID_HOME` or a `local.properties` file (`sdk.dir=...`, not committed).

```bash
cd apps/android
./gradlew assembleDebug                               # APK: app/build/outputs/apk/debug/
./gradlew testDebugUnitTest :core:triage:test         # unit tests
./gradlew installDebug                                # onto a running emulator/device
```

Or open `apps/android` in Android Studio.

CI: `.github/workflows/android.yml`, runs only on changes under `apps/android/**` or `spec/**`.
