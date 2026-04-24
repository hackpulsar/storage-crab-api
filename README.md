# Storage Crab API

A Rust-based backend API for file storage with sharing capabilities.

## Overview
This backend provides a secure, HTTPS-enabled API for managing user accounts and storing files. Built with Rust using the Actix-web framework, it supports user registration, login with JWT-based authentication, file uploads/downloads, and temporary file sharing via share codes.

## Features

- **User Authentication** - Registration and login with JWT tokens
- **Token Refresh** - Secure token refresh with blacklisting via Redis
- **File Storage** - Upload, download, list, and delete files
- **File Sharing** - Generate temporary share codes (5-minute expiry) via Redis
- **HTTPS** - TLS/SSL encrypted communication

## Architecture

```
src/
├── main.rs           # Server setup, TLS config, app state
├── routes/           # HTTP endpoints
│   ├── auth.rs       # Login, register, token refresh
│   ├── files.rs      # File operations
│   └── user.rs       # User profile endpoints
├── services/         # Business logic
├── models/           # Data structures
└── utils/            # Error handling, helpers
```

## Documentation

See the [docs](docs/) directory for detailed documentation:
- [API Reference](docs/api.md)
- [Configuration](docs/config.md)
- [Deployment](docs/deployment.md)