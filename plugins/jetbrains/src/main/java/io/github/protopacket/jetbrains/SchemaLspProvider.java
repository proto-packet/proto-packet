package io.github.protopacket.jetbrains;

import com.intellij.openapi.project.Project;
import com.intellij.openapi.vfs.VirtualFile;
import com.intellij.platform.lsp.api.LspIntegrationProvider;
import org.jetbrains.annotations.NotNull;

/** Starts the language server of the {@code packet} compiler when a schema file is opened. */
public final class SchemaLspProvider implements LspIntegrationProvider {

    @Override
    public void fileOpened(
            @NotNull Project project,
            @NotNull VirtualFile file,
            @NotNull LspClientStarter clientStarter
    ) {
        if (SchemaFiles.isSchemaFile(file)) {
            clientStarter.ensureClientStarted(new SchemaLspDescriptor(project));
        }
    }
}
