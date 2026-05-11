package dev.kreuzberg.android

/**
 * JNI entry point for kreuzberg on Android.
 *
 * Loads the native kreuzberg-ffi library and exposes document extraction.
 * Results are returned as JSON strings for compatibility with all Android versions.
 */
object KreuzbergAndroid {
    init {
        System.loadLibrary("kreuzberg_ffi")
    }

    /**
     * Extract text from a document given its raw bytes and MIME type.
     *
     * @param bytes Raw document bytes
     * @param mimeType MIME type of the document (e.g. "application/pdf")
     * @return JSON-encoded ExtractionResult string
     */
    external fun extractBytes(bytes: ByteArray, mimeType: String): String

    /**
     * Extract text from a document at the given file path.
     *
     * @param path Absolute path to the document file
     * @param mimeType MIME type of the document, or null for auto-detection
     * @return JSON-encoded ExtractionResult string
     */
    external fun extractFile(path: String, mimeType: String?): String
}
