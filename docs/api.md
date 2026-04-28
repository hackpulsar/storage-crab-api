# API Reference

## Authentication

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/users/` | Register new user | No |
| `POST` | `/api/token/get/` | Login | No |
| `POST` | `/api/token/refresh/` | Refresh tokens | No |
| `POST` | `/api/users/greet/` | Test authentication | Yes |

## User

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/users/me/` | Get current user info | Yes |

## Files

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/files/` | List user's files | Yes |
| `POST` | `/api/files/upload/` | Upload a file | Yes |
| `GET` | `/api/files/download/{file_id}/` | Download a file | Yes |
| `POST` | `/api/files/delete/{file_id}/` | Delete a file | Yes |
| `POST` | `/api/files/share/{file_id}/` | Generate share code | Yes |
| `GET` | `/api/files/download/shared/{share_code}/` | Download shared file | Yes |

## Request/Response Formats

### Register User
```json
// POST /api/users/
// Request
{
  "email": "user@example.com",
  "username": "johndoe",
  "password_hash": "hashed_password"
}

// Response
{
  "id": 1,
  "email": "user@example.com",
  "username": "johndoe"
}
```

### Login
```json
// POST /api/token/get/
// Request
{
  "email": "user@example.com",
  "password_hash": "hashed_password"
}

// Response
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ..."
}
```

### Upload File
```
// POST /api/files/upload/
// Content-Type: multipart/form-data
// Form fields:
//   - file: <binary>
//   - json: {"filename": "document.pdf.enc"}
```

### Refresh Token
```json
// POST /api/token/refresh/
// Request
{
  "refresh_token": "eyJ..."
}

// Response
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ..."
}
```

### Greet
```json
// POST /api/users/greet/
// Response (plain text)
Welcome back, johndoe
```

### Get Current User
```json
// GET /api/users/me/
// Response
{
  "id": 1,
  "email": "user@example.com",
  "username": "johndoe"
}
```

### List Files
```json
// GET /api/files/
// Response
[
  {
    "id": 1,
    "filename": "document.pdf.enc",
    "path": "/app/files_storage/<username_hash>/document.pdf.enc",
    "size": 1024000,
    "uploaded_at": "2024-01-15T10:30:00",
    "user_id": 1
  }
]
```

### Download File
```
// GET /api/files/download/{file_id}/
// Response: binary file stream
// Headers:
//   - Content-Disposition: attachment; filename="document.pdf.enc"
//   - Content-Length: 1024000
```

### Delete File
```
// POST /api/files/delete/{file_id}/
// Response: 204 No Content
```

### Download Shared File
```
// GET /api/files/download/shared/{share_code}/
// Response: binary file stream (no auth required)
// Headers:
//   - Content-Disposition: attachment; filename="document.pdf.enc"
//   - Content-Length: 1024000
```

## JWT Tokens

- **Access token**: 10 minute expiry
- **Refresh token**: 30 minute expiry
- **Header format**: `Authorization: Bearer <token>`
- **Share codes**: 5 minute expiry, stored in Redis