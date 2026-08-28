allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

val newBuildDir: Directory =
    rootProject.layout.buildDirectory
        .dir("../../build")
        .get()
rootProject.layout.buildDirectory.value(newBuildDir)

subprojects {
    val newSubprojectBuildDir: Directory = newBuildDir.dir(project.name)
    project.layout.buildDirectory.value(newSubprojectBuildDir)

    // Legacy-plugin workaround. Runs after each subproject evaluates so we can
    // (a) inject a missing `namespace` (isar_flutter_libs predates AGP 8) and
    // (b) raise compileSdk for old plugins (sensors_plus) whose transitive
    // dependencies require API 34+. Registered before evaluationDependsOn so
    // the callback is in place when evaluation is forced.
    project.afterEvaluate {
        extensions.findByName("android")?.let { android ->
            if (android is com.android.build.gradle.BaseExtension) {
                if (android.namespace.isNullOrBlank()) {
                    android.namespace =
                        "com.eel.${project.name.replace('-', '_').replace('.', '_')}"
                }
                android.compileSdkVersion(36)
            }
        }
    }
}

subprojects {
    project.evaluationDependsOn(":app")
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
