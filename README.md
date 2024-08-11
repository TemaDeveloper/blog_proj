# Auth System Oauth2 implementation with Rust (Axum)

This is a Rust project that is used Axum framework and Sea-Orm
Modern approach of building auth system, using Oauth2 and Google Authorization Services.
Authentication workflow: 

![3](https://github.com/user-attachments/assets/a4867e71-6286-493f-9f22-c351dab76a1f)

 - After the client gets the code from Authorization Server it exchanges it to access_token 

 - Using the access_token, client gets user information from two scopes

**/userinfo.profile**<br />
**/userinfo.email**

 - Client gets email from the Database and compare it to the one that it gets from the Resource Server
 - If two emails match => create a new session in DB and store session_id in cookie
 - Using the logout endpoint, all cookies of the logged in user will be deleted and also the session. 

 - Sea-Orm - LAST VERSION [1.0.0]
 - Axum - LAST VERSION [0.7.5]
 - Oauth2 - LAST VERSION[4.4.2]

Run Docker:
```
docker-compose up
```

Run Project: 
```
cargo watch -q -c -w src/ -x run
```


