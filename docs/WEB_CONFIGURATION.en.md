# Web Real-time Configuration

MCP-ANY-REST provides a web-based real-time configuration interface that allows you to dynamically modify server configuration, manage modules, and apply presets at runtime.

## Access

The web configuration interface is integrated into the server by default and can be accessed via the browser at the server's root path:

```
http://localhost:<port>/
```

Where `<port>` is the server port configured in your configuration file (default is 3000).

## Key Features

### 1. Overview & Status
- View the current running status of the server.
- View basic configuration information (Base URL, Log Level, etc.).

### 2. Module Management
- **Enable/Disable Modules**: Enable or disable specific functional modules in real-time.
- **Access Control**: Set access levels for modules (Public, Protected, Private).
- **Rate Limiting**: Configure request limits per minute/hour and burst capacity.
- **Method-Level Control**: Granular configuration down to specific methods.

### 3. Preset Management
- **Apply Presets**: Quickly switch between different configuration presets (e.g., "Full", "Minimal").
- **Save Presets**: Save the current configuration as a new preset.
- **Load/Delete**: Manage saved configuration snapshots.

### 4. Server Configuration
- Modify server port.
- Adjust log levels.
- Update API Base URL.

### 5. Authentication Configuration
- Switch authentication modes (Direct / Login).
- Configure authentication strategies (Bearer Token, API Key, OAuth2, etc.).
- Set token expiry times and refresh policies.

## Usage Instructions

All configuration changes take effect immediately (Dynamic Configuration) and typically do not require a server restart. Configuration changes are automatically persisted to the configuration file to ensure consistency on the next startup.
