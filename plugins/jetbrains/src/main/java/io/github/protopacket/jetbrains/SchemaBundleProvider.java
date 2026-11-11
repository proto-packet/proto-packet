package io.github.protopacket.jetbrains;

import com.intellij.ide.plugins.IdeaPluginDescriptor;
import com.intellij.ide.plugins.PluginManagerCore;
import com.intellij.openapi.extensions.PluginId;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.plugins.textmate.api.TextMateBundleProvider;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/**
 * Provides the TextMate bundle with the syntax of schema files.
 *
 * <p>The bundle is the {@code textmate} folder of the installed plugin, since the IDE reads a
 * bundle from a folder and not from a jar.
 */
public final class SchemaBundleProvider implements TextMateBundleProvider {

    /** The id of this plugin. */
    private static final PluginId PLUGIN_ID = PluginId.getId("io.github.protopacket");

    /** The name of the bundle in the settings of the IDE. */
    private static final String BUNDLE_NAME = "ProtoPacket";

    /** The folder of the bundle in the folder of the plugin. */
    private static final String BUNDLE_FOLDER = "textmate";

    @Override
    public @NotNull List<PluginBundle> getBundles() {
        IdeaPluginDescriptor plugin = PluginManagerCore.getPlugin(PLUGIN_ID);
        if (plugin == null) {
            return List.of();
        }
        Path bundle = plugin.getPluginPath().resolve(BUNDLE_FOLDER);
        if (!Files.isDirectory(bundle)) {
            return List.of();
        }
        return List.of(new PluginBundle(BUNDLE_NAME, bundle));
    }
}
