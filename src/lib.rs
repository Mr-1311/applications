use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static APP_PATHS: RefCell<HashMap<String, (String, Vec<String>)>> = RefCell::new(HashMap::new());
}

#[host_fn]
extern "ExtismHost" {
    fn cli_run(command: String, args: Json<Vec<String>>) -> String;
}

#[derive(Serialize)]
struct PluginCommand {
    name: String,
    description: String,
    icon: String,
}

#[plugin_fn]
pub fn init() -> FnResult<Json<Vec<PluginCommand>>> {
    let platform = config::get("platform")
        .unwrap_or(Some("".to_owned()))
        .unwrap_or("".to_owned());

    if platform.eq("macos") {
        return get_applications_macos();
    }
    if platform.eq("linux") {
        return get_applications_linux();
    }

    Ok(Json(vec![]))
}

#[plugin_fn]
pub fn filter(_query: String) -> FnResult<Json<Vec<PluginCommand>>> {
    Ok(Json(vec![]))
}

#[plugin_fn]
pub fn on_select(selected: String) -> FnResult<()> {
    APP_PATHS.with(|paths| {
        if let Some((command, args)) = paths.borrow().get(&selected) {
            unsafe {
                cli_run(command.clone(), Json(args.clone()))?;
            }
        }
        Ok(())
    })
}

fn get_applications_macos() -> FnResult<Json<Vec<PluginCommand>>> {
    use std::fs;
    use std::path::PathBuf;

    let mut applications = Vec::new();

    // Get HOME directory using cli_run
    let mut scan_dirs = vec![PathBuf::from("/Applications")];

    // Try to get home directory, continue without it if fails
    if let Ok(home_path) =
        unsafe { cli_run("printenv".to_string(), Json(vec!["HOME".to_string()])) }
    {
        let home_apps = PathBuf::from(format!("{}/Applications", home_path.trim()));
        scan_dirs.push(home_apps);
    }

    // Get additional paths from config
    let additional_paths: Vec<String> = config::get("additional paths")
        .unwrap_or(Some("".to_owned()))
        .unwrap_or("".to_owned())
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .collect();

    // Get applications filter from config
    let application_filter: Vec<String> = config::get("application filter")
        .unwrap_or(Some("".to_owned()))
        .unwrap_or("".to_owned())
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .collect();

    // Add additional paths
    scan_dirs.extend(additional_paths.into_iter().map(PathBuf::from));

    // Process all directories
    for dir in scan_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check if the entry is an .app bundle
                if path.is_dir() && path.extension().map_or(false, |ext| ext == "app") {
                    let plist_path = path.join("Contents").join("Info.plist");

                    // Try to read and parse the Info.plist file
                    if let Ok(plist_content) = fs::read_to_string(&plist_path) {
                        if let Ok(plist_value) =
                            plist::from_bytes::<plist::Value>(plist_content.as_bytes())
                        {
                            let plist_dict = match plist_value.as_dictionary() {
                                Some(dict) => dict,
                                None => continue,
                            };

                            // Get app name from path if not in plist
                            let default_name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .trim_end_matches(".app")
                                .to_string();

                            // Extract name
                            let name = plist_dict
                                .get("CFBundleName")
                                .and_then(|v| v.as_string())
                                .unwrap_or(&default_name)
                                .to_string();

                            if !application_filter.is_empty()
                                && !application_filter
                                    .iter()
                                    .any(|f| f.to_lowercase() == name.to_lowercase())
                            {
                                continue;
                            }

                            // Extract description (optional)
                            let description = plist_dict
                                .get("NSHumanReadableDescription")
                                .and_then(|v| v.as_string())
                                .unwrap_or("")
                                .to_string();

                            // Extract icon file and construct full path
                            let icon_file = plist_dict
                                .get("CFBundleIconFile")
                                .and_then(|v| v.as_string())
                                .unwrap_or("AppIcon");

                            let icon_path = path.join("Contents").join("Resources");
                            let icon = if icon_file.ends_with(".icns") {
                                icon_path.join(icon_file)
                            } else {
                                icon_path.join(format!("{}.icns", icon_file))
                            };

                            APP_PATHS.with(|paths| {
                                paths.borrow_mut().insert(
                                    name.clone(),
                                    (
                                        "open".to_string(),
                                        vec![path.to_string_lossy().into_owned()],
                                    ),
                                );
                            });

                            // Add the application to the list
                            applications.push(PluginCommand {
                                name,
                                description,
                                icon: icon.to_string_lossy().into_owned(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(Json(applications))
}

#[derive(Deserialize)]
struct AppInfo {
    name: String,
    exec: String,
    description: String,
    icon_path: String,
}

// Function to get applications on Linux
fn get_applications_linux() -> FnResult<Json<Vec<PluginCommand>>> {
    // Get additional paths from config
    let additional_paths: String = config::get("additional paths")
        .unwrap_or(Some("".to_owned()))
        .unwrap_or("".to_owned());

    // Get application filter from config
    let application_filter: Vec<String> = config::get("application filter")
        .unwrap_or(Some("".to_owned()))
        .unwrap_or("".to_owned())
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .collect();

    // Execute /data/linux_list_apps and get JSON output
    let json_output = unsafe {
        cli_run(
            "/data/linux_list_apps".to_string(),
            Json(vec![additional_paths]),
        )
    }?;

    // Parse JSON output into a vector of AppInfo
    let apps: Vec<AppInfo> = serde_json::from_str(&json_output).map_err(Error::from)?;

    let mut applications = Vec::new();

    // Process each application
    for app in apps {
        // Apply filter if present
        if !application_filter.is_empty()
            && !application_filter
                .iter()
                .any(|f| f.to_lowercase() == app.name.to_lowercase())
        {
            continue;
        }

        // Split exec field into command and arguments, removing % placeholders
        let parts = shlex::split(&app.exec).unwrap_or(vec![app.exec.clone()]);
        let filtered_parts: Vec<String> =
            parts.into_iter().filter(|p| !p.starts_with('%')).collect();

        if !filtered_parts.is_empty() {
            let command = filtered_parts[0].clone();
            let args = filtered_parts[1..].to_vec();

            // Store command and args in APP_PATHS for launching
            APP_PATHS.with(|paths| {
                paths.borrow_mut().insert(app.name.clone(), (command, args));
            });

            // Add to applications list
            applications.push(PluginCommand {
                name: app.name,
                description: app.description,
                icon: app.icon_path, // Use absolute icon path directly
            });
        }
    }

    Ok(Json(applications))
}
