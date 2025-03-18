# Applications Plugin for Puppet

A Puppet plugin that lists and runs applications installed on your system.

## Features

- Lists applications from standard system directories
- Opens applications directly from Puppet
- Supports filtering applications by name
- Allows adding custom search paths
- Cross-platform compatible (macOS implementation currently available)

## Configuration

The plugin accepts two configuration parameters:

1. `application filter` - Comma-separated list of application names to include. If empty, all applications are listed.
```
Example: "Firefox,Chrome,Visual Studio Code"
```

2. `additional paths` - Comma-separated list of additional directories to scan for applications.
```
Example: "/Users/username/CustomApps,/opt/applications"
```

## Default Search Paths

On macOS, the plugin searches for applications in:
- `/Applications`
- `~/Applications`
- Any additional paths specified in the configuration

## Platform Support

- ✅ macOS
- ✅ Windows
- ✅ Linux (gtk+-3.0 required)

