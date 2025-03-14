// gcc linux_list_apps.c -o linux_list_apps $(pkg-config --cflags --libs gtk+-3.0)
#include <gtk/gtk.h>
#include <glib.h>
#include <stdio.h>
#include <string.h>

typedef struct {
    GString *json;
    gboolean first_entry;
} JsonBuilder;

static gchar* escape_json(const gchar *str) {
    if (!str) return g_strdup("");
    
    GString *escaped = g_string_new("");
    for (const gchar *p = str; *p; p++) {
        switch (*p) {
            case '"':  g_string_append(escaped, "\\\""); break;
            case '\\': g_string_append(escaped, "\\\\"); break;
            case '\b': g_string_append(escaped, "\\b"); break;
            case '\f': g_string_append(escaped, "\\f"); break;
            case '\n': g_string_append(escaped, "\\n"); break;
            case '\r': g_string_append(escaped, "\\r"); break;
            case '\t': g_string_append(escaped, "\\t"); break;
            default:
                if (*p < 0x20) {
                    g_string_append_printf(escaped, "\\u%04x", *p);
                } else {
                    g_string_append_c(escaped, *p);
                }
        }
    }
    return g_string_free(escaped, FALSE);
}

static void process_desktop_file(const gchar *path, JsonBuilder *builder, GtkIconTheme *icon_theme) {
    GKeyFile *key_file = g_key_file_new();
    GError *error = NULL;

    if (!g_key_file_load_from_file(key_file, path, G_KEY_FILE_NONE, &error)) {
        g_clear_error(&error);
        g_key_file_free(key_file);
        return;
    }

    if (!g_key_file_has_group(key_file, "Desktop Entry")) {
        g_key_file_free(key_file);
        return;
    }

    gchar *type = g_key_file_get_string(key_file, "Desktop Entry", "Type", NULL);
    if (type == NULL || g_strcmp0(type, "Application") != 0) {
        g_free(type);
        g_key_file_free(key_file);
        return;
    }
    g_free(type);

    gboolean no_display = g_key_file_get_boolean(key_file, "Desktop Entry", "NoDisplay", NULL);
    if (no_display) {
        g_key_file_free(key_file);
        return;
    }

    gchar *exec = g_key_file_get_string(key_file, "Desktop Entry", "Exec", NULL);
    if (!exec) {
        g_key_file_free(key_file);
        return;
    }

    gchar *name = g_key_file_get_locale_string(key_file, "Desktop Entry", "Name", NULL, NULL);
    if (!name) {
        g_free(exec);
        g_key_file_free(key_file);
        return;
    }

    gchar *comment = g_key_file_get_locale_string(key_file, "Desktop Entry", "Comment", NULL, NULL);
    gchar *icon_name = g_key_file_get_string(key_file, "Desktop Entry", "Icon", NULL);

    gchar *icon_path = NULL;
    if (icon_name != NULL) {
        if (g_path_is_absolute(icon_name)) {
            icon_path = g_strdup(icon_name);
        } else {
            GtkIconInfo *icon_info = gtk_icon_theme_lookup_icon(
                icon_theme, icon_name, 48, GTK_ICON_LOOKUP_USE_BUILTIN);
            if (icon_info != NULL) {
                const gchar *filename = gtk_icon_info_get_filename(icon_info);
                if (filename != NULL) {
                    icon_path = g_strdup(filename);
                }
                g_object_unref(icon_info);
            }
        }
    }

    gchar *escaped_name = escape_json(name);
    gchar *escaped_exec = escape_json(exec);
    gchar *escaped_comment = escape_json(comment);
    gchar *escaped_icon = escape_json(icon_path);

    if (!builder->first_entry) {
        g_string_append(builder->json, ",\n");
    } else {
        builder->first_entry = FALSE;
    }

    g_string_append_printf(builder->json,
        "  {\n"
        "    \"name\": \"%s\",\n"
        "    \"exec\": \"%s\",\n"
        "    \"description\": \"%s\",\n"
        "    \"icon_path\": \"%s\"\n"
        "  }",
        escaped_name, escaped_exec, escaped_comment ? escaped_comment : "",
        escaped_icon ? escaped_icon : "");

    g_free(escaped_name);
    g_free(escaped_exec);
    g_free(escaped_comment);
    g_free(escaped_icon);
    g_free(name);
    g_free(comment);
    g_free(exec);
    g_free(icon_name);
    g_free(icon_path);
    g_key_file_free(key_file);
}

static void add_search_paths(GList **app_dirs, const gchar *paths) {
    gchar **tokens = g_strsplit(paths, ",", -1);
    for (gchar **dir = tokens; *dir != NULL; dir++) {
        gchar *clean_dir = g_strstrip(*dir);
        if (*clean_dir) {
            *app_dirs = g_list_append(*app_dirs, g_strdup(clean_dir));
        }
    }
    g_strfreev(tokens);
}

int main(int argc, char **argv) {
    gtk_init(&argc, &argv);

    JsonBuilder builder = { g_string_new("[\n"), TRUE };
    GtkIconTheme *icon_theme = gtk_icon_theme_get_default();
    GList *app_dirs = NULL;

    // Standard XDG paths
    gchar **data_dirs = g_strsplit(g_getenv("XDG_DATA_DIRS") ? 
                                 g_getenv("XDG_DATA_DIRS") : 
                                 "/usr/local/share/:/usr/share/", ":", -1);
    for (gchar **dir = data_dirs; *dir != NULL; dir++) {
        gchar *app_dir = g_build_filename(*dir, "applications", NULL);
        app_dirs = g_list_append(app_dirs, app_dir);
    }

    // User directory
    const gchar *home_dir = g_get_home_dir();
    gchar *user_app_dir = g_build_filename(home_dir, ".local", "share", "applications", NULL);
    app_dirs = g_list_append(app_dirs, user_app_dir);

    // Additional paths from command line
    if (argc > 1) {
        add_search_paths(&app_dirs, argv[1]);
    }

    // Process all directories
    for (GList *l = app_dirs; l != NULL; l = l->next) {
        const gchar *app_dir = l->data;
        GDir *dir = g_dir_open(app_dir, 0, NULL);
        if (!dir) continue;

        const gchar *filename;
        while ((filename = g_dir_read_name(dir)) != NULL) {
            if (g_str_has_suffix(filename, ".desktop")) {
                gchar *path = g_build_filename(app_dir, filename, NULL);
                process_desktop_file(path, &builder, icon_theme);
                g_free(path);
            }
        }
        g_dir_close(dir);
    }

    g_string_append(builder.json, "\n]");
    puts(builder.json->str);

    // Cleanup
    g_string_free(builder.json, TRUE);
    g_list_free_full(app_dirs, g_free);
    g_strfreev(data_dirs);

    return 0;
}
