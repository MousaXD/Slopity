@ECHO OFF
SETLOCAL
SET APP_HOME=%~dp0
SET WRAPPER_JAR=%APP_HOME%gradle\wrapper\gradle-wrapper.jar
SET DOWNLOADER=%APP_HOME%gradle\wrapper\WrapperDownloader.java

IF NOT EXIST "%WRAPPER_JAR%" (
  ECHO Gradle wrapper JAR is missing; downloading the pinned, checksum-verified copy. 1>&2
  java "%DOWNLOADER%" "%WRAPPER_JAR%"
  IF ERRORLEVEL 1 EXIT /B 1
)

java %JAVA_OPTS% %GRADLE_OPTS% -Dorg.gradle.appname=gradlew -classpath "%WRAPPER_JAR%" org.gradle.wrapper.GradleWrapperMain %*
ENDLOCAL
