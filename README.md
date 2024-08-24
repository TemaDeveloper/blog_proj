# OAuth2 Authentication with Rust (Axum Framework)

![Rust](https://img.shields.io/badge/language-Rust-orange.svg)

This Rust project utilizes the Axum framework and Sea-ORM for building a modern, secure, and efficient authentication system. The system integrates OAuth2 and Google Authorization Services to manage user authentication.

## Authentication Workflow

![OAuth2 Authentication Flow](https://github.com/user-attachments/assets/a4867e71-6286-493f-9f22-c351dab76a1f)

1. **Authorization Code Retrieval**  
   The client application initiates the OAuth2 flow by redirecting the user to the Authorization Server, where the user consents to sharing their information. Upon successful authorization, the server returns an authorization code to the client.

2. **Access Token Exchange**  
   The client exchanges the authorization code for an access token by making a request to the Authorization Server's token endpoint.

3. **User Information Retrieval**  
   With the access token, the client retrieves user information from the Resource Server using two scopes:
   - `/userinfo.profile`
   - `/userinfo.email`

4. **Email Verification and Session Management**  
   The client compares the email received from the Resource Server with the email stored in the database. If the emails match:
   - A new session is created in the database.
   - The session ID is stored in a cookie on the client.

5. **Logout Handling**  
   The logout endpoint invalidates the user's session by deleting the session record from the database and clearing all related cookies.

## Technology Stack

- **Sea-ORM**: Version `1.0.0`  
   An async and dynamic ORM framework for Rust that enables efficient database interactions.
- **Axum**: Version `0.7.5`  
   A web framework for Rust that is focused on ergonomics and modularity.
- **OAuth2**: Version `4.4.2`  
   A complete, modern, and secure OAuth2 client library for Rust, designed to handle various OAuth2 flows and grant types.
- **Redis**: Version `0.26.1`  
- **Amazon S3**: libs => aws-config = `1.5.5` aws-sdk-s3 = `1.46.0`
  
## Session Storage with Redis

To enhance the performance of session management, Redis is employed for quick session retrieval and storage. Each session is associated with a unique `session_id`, which is an integer. Sessions have a one-hour expiration time, ensuring that stale sessions are automatically removed.

**Redis Key Structure**:
- `"session_id" : integer`

This approach ensures that user sessions are efficiently managed, improving the overall responsiveness of the application.

## File Uploads using Amazon S3

- **File Upload**: Supports uploading files via multipart form data.
- **AWS S3 Integration**: Uses the AWS SDK for Rust to handle file storage on Amazon S3.
- **Asynchronous Processing**: The application is built using asynchronous Rust to handle multiple concurrent uploads efficiently.

## Getting Started

### Running the Application

**Start Docker Services**  
Ensure all required services (e.g., PostgreSQL, Redis) are up and running.
```bash
docker-compose up
```
**Start Server**
```
cargo watch -q -c -w src/ -x run
```
