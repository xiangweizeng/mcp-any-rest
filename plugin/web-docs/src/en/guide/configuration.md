---
title: Web Live Configuration
icon: bolt
article: false
footer: true
---

# Web Live Configuration

MCP-ANY-REST provides a Web-based live configuration interface, allowing you to dynamically modify server configuration, manage modules, and apply presets at runtime.

## Access Method

The Web configuration interface is integrated into the server by default and can be accessed via the browser at the server's root path:

```
http://localhost:<port>/
```

Where `<port>` is the server port set in your configuration file (default is 3000).

## Main Features

### 1. Overview and Status
- View the current running status of the server.
- View basic configuration information (Base URL, Log Level, etc.).

### 2. Module Management (Modules)
- **Enable/Disable Modules**: Enable or disable specific functional modules in real-time.
- **Access Control**: Set the access level of modules (Public, Protected, Private).
- **Rate Limiting**: Configure request limits per minute/hour and burst capacity.
- **Method-Level Control**: Granular configuration down to the specific Method level.

### 3. Preset Management (Presets)
- **Apply Presets**: Quickly switch between different configuration presets (e.g., "Full", "Minimal").
- **Save Presets**: Save the current configuration as a new preset.
- **Load/Delete**: Manage saved configuration snapshots.

### 4. Server Configuration
- Modify server port.
- Adjust log level.
- Update API Base URL.

### 5. Authentication Configuration (Authentication)
- Switch authentication modes (Direct / Login).
- Configure authentication policies (Bearer Token, API Key, OAuth2, etc.).
- Set Token expiration time and refresh policies.

## Usage Instructions

All configuration changes take effect immediately (Dynamic Configuration), usually without restarting the server. Configuration changes are automatically persisted to the configuration file, ensuring consistency on the next startup.
