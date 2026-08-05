#!/bin/sh
set -eu

APP_HOME=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
WRAPPER_JAR="$APP_HOME/gradle/wrapper/gradle-wrapper.jar"
DOWNLOADER="$APP_HOME/gradle/wrapper/WrapperDownloader.java"

if [ ! -f "$WRAPPER_JAR" ]; then
    echo "Gradle wrapper JAR is missing; downloading the pinned, checksum-verified copy." >&2
    java "$DOWNLOADER" "$WRAPPER_JAR"
fi

exec java ${JAVA_OPTS:-} ${GRADLE_OPTS:-} \
    -Dorg.gradle.appname=gradlew \
    -classpath "$WRAPPER_JAR" \
    org.gradle.wrapper.GradleWrapperMain "$@"
