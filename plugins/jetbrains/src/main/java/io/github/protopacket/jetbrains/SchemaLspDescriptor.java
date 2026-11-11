package io.github.protopacket.jetbrains;

import com.intellij.execution.configurations.GeneralCommandLine;
import com.intellij.execution.configurations.PathEnvironmentVariableUtil;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.vfs.VirtualFile;
import com.intellij.platform.lsp.api.ProjectWideLspClientDescriptor;
import org.jetbrains.annotations.NotNull;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Describes the language server of the {@code packet} compiler for the schema files of a project.
 *
 * <p>The compiler is found on the {@code PATH}, or in the folder that {@code cargo install} puts
 * it in. The IDE reports the server as failed if the compiler isn't installed.
 */
final class SchemaLspDescriptor extends ProjectWideLspClientDescriptor {

    /** The name of the language server in the IDE. */
    private static final String NAME = "ProtoPacket";

    /** The name of the compiler's executable. */
    private static final String COMPILER = "packet";

    /** The command of the compiler that runs the language server. */
    private static final String COMMAND = "lsp";

    SchemaLspDescriptor(@NotNull Project project) {
        super(project, NAME);
    }

    @Override
    public boolean isSupportedFile(@NotNull VirtualFile file) {
        return SchemaFiles.isSchemaFile(file);
    }

    @Override
    public @NotNull GeneralCommandLine createCommandLine() {
        return new GeneralCommandLine(compilerPath(), COMMAND);
    }

    /** Gets the path of the compiler, or its name if it isn't found. */
    private static @NotNull String compilerPath() {
        File onPath = PathEnvironmentVariableUtil.findInPath(COMPILER);
        if (onPath != null) {
            return onPath.getPath();
        }
        Path installed = Path.of(System.getProperty("user.home"), ".cargo", "bin", COMPILER);
        if (Files.isExecutable(installed)) {
            return installed.toString();
        }
        return COMPILER;
    }
}
