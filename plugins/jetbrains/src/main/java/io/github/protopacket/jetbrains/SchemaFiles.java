package io.github.protopacket.jetbrains;

import com.intellij.openapi.vfs.VirtualFile;
import org.jetbrains.annotations.NotNull;

/** The schema files of ProtoPacket. */
final class SchemaFiles {

    /** The file extension of a schema file. */
    private static final String EXTENSION = "pps";

    private SchemaFiles() {
    }

    /** Checks if the {@code file} is a schema file. */
    static boolean isSchemaFile(@NotNull VirtualFile file) {
        return EXTENSION.equals(file.getExtension());
    }
}
