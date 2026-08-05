import java.io.InputStream;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.util.HexFormat;

public final class WrapperDownloader {
    private static final URI WRAPPER_URI = URI.create(
        "https://services.gradle.org/distributions/gradle-9.6.0-wrapper.jar"
    );
    private static final String EXPECTED_SHA256 =
        "497c8c2a7e5031f6aa847f88104aa80a93532ec32ee17bdb8d1d2f67a194a9c7";

    private WrapperDownloader() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("Expected destination path");
        }

        Path destination = Path.of(args[0]).toAbsolutePath();
        Files.createDirectories(destination.getParent());
        Path temporary = Files.createTempFile(destination.getParent(), "gradle-wrapper", ".jar.tmp");

        try {
            HttpClient client = HttpClient.newBuilder()
                .followRedirects(HttpClient.Redirect.NORMAL)
                .build();
            HttpRequest request = HttpRequest.newBuilder(WRAPPER_URI).GET().build();
            HttpResponse<InputStream> response = client.send(
                request,
                HttpResponse.BodyHandlers.ofInputStream()
            );

            if (response.statusCode() != 200) {
                throw new IllegalStateException(
                    "Gradle wrapper download failed with HTTP " + response.statusCode()
                );
            }

            try (InputStream body = response.body()) {
                Files.copy(body, temporary, StandardCopyOption.REPLACE_EXISTING);
            }

            String actualSha256 = sha256(temporary);
            if (!EXPECTED_SHA256.equalsIgnoreCase(actualSha256)) {
                throw new SecurityException(
                    "Gradle wrapper checksum mismatch. Expected " + EXPECTED_SHA256
                        + " but received " + actualSha256
                );
            }

            Files.move(
                temporary,
                destination,
                StandardCopyOption.REPLACE_EXISTING,
                StandardCopyOption.ATOMIC_MOVE
            );
            System.out.println("Downloaded and verified " + destination);
        } finally {
            Files.deleteIfExists(temporary);
        }
    }

    private static String sha256(Path file) throws Exception {
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        try (InputStream input = Files.newInputStream(file)) {
            byte[] buffer = new byte[8192];
            int read;
            while ((read = input.read(buffer)) >= 0) {
                digest.update(buffer, 0, read);
            }
        }
        return HexFormat.of().formatHex(digest.digest());
    }
}
