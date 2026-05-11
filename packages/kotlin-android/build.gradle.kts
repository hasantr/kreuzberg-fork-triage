plugins {
    id("com.android.library") version "8.3.0"
    id("org.jetbrains.kotlin.android") version "1.9.23"
    id("maven-publish")
}

android {
    namespace = "dev.kreuzberg.android"
    compileSdk = 35

    defaultConfig {
        minSdk = 21
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
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

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = "dev.kreuzberg"
            artifactId = "kreuzberg-android"
            version = project.findProperty("version")?.toString() ?: "0.0.1"

            afterEvaluate {
                from(components["release"])
            }

            pom {
                name.set("kreuzberg-android")
                description.set("Kreuzberg document intelligence library — Android native bindings")
                url.set("https://kreuzberg.dev")
                licenses {
                    license {
                        name.set("Elastic License 2.0")
                        url.set("https://www.elastic.co/licensing/elastic-license")
                    }
                }
            }
        }
    }
}

dependencies {
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
}
