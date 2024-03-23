use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::{
    db::bundlesdb::{self, AppData, DefaultReturn, FullUser, UserFollow, UserMetadata},
    utility,
};

#[derive(serde::Deserialize)]
struct RegisterInfo {
    username: String,
}

#[derive(serde::Deserialize)]
struct LoginInfo {
    uid: String,
}

#[derive(serde::Deserialize)]
struct UpdateAboutInfo {
    about: String,
}

#[post("/api/auth/register")]
pub async fn register(body: web::Json<RegisterInfo>, data: web::Data<AppData>) -> impl Responder {
    // if server disabled registration, return
    let disabled = crate::config::get_var("REGISTRATION_DISABLED");

    if disabled.is_some() {
        return HttpResponse::NotAcceptable()
            .body("This server requires has registration disabled");
    }

    // ...
    let username = &body.username.trim();
    let res = data.db.create_user(username.to_string()).await;

    let c = res.clone();
    let set_cookie = if res.success && res.payload.is_some() {
        format!("__Secure-Token={}; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}", c.message, 60 * 60 * 24 * 365)
    } else {
        String::new()
    };

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", if res.success { &set_cookie } else { "" }))
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/login")]
pub async fn login(body: web::Json<LoginInfo>, data: web::Data<AppData>) -> impl Responder {
    let id = body.uid.trim();
    let id_hashed = utility::hash(id.to_string());

    let res = data
        .db
        .get_user_by_hashed(id_hashed) // if the user is returned, that means the ID is valid
        .await;

    let set_cookie = if res.success && res.payload.is_some() {
        format!("__Secure-Token={}; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}", body.uid, 60 * 60 * 24 * 365)
    } else {
        String::new()
    };

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", if res.success { &set_cookie } else { "" }))
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/login-st")]
pub async fn login_secondary_token(
    body: web::Json<LoginInfo>,
    data: web::Data<AppData>,
) -> impl Responder {
    let id = body.uid.trim();
    let id_unhashed = id.to_string();

    let res = data
        .db
        .get_user_by_unhashed_st(id_unhashed) // if the user is returned, that means the token is valid
        .await;

    let set_cookie = if res.success && res.payload.is_some() {
        format!("__Secure-Token={}; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}", body.uid, 60 * 60 * 24 * 365)
    } else {
        String::new()
    };

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", if res.success { &set_cookie } else { "" }))
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[get("/api/auth/logout")]
pub async fn logout(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let cookie = req.cookie("__Secure-Token");

    if cookie.is_none() {
        return HttpResponse::NotAcceptable().body("Missing token");
    }

    let res = data
        .db
        .get_user_by_unhashed(cookie.unwrap().value().to_string()) // if the user is returned, that means the ID is valid
        .await;

    if !res.success {
        return HttpResponse::NotAcceptable().body("Invalid token");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0"))
        .append_header(("Content-Type", "text/plain"))
        .body("You have been signed out. You can now close this tab.");
}

#[post("/api/auth/users/{name:.*}/about")]
pub async fn edit_about_request(
    req: HttpRequest,
    body: web::Json<UpdateAboutInfo>,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get token user
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    let token_user = token_user.unwrap().payload.unwrap();

    // make sure profile exists
    let profile: DefaultReturn<Option<FullUser<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if !profile.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Profile does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let profile = profile.payload.unwrap();
    let mut user = serde_json::from_str::<UserMetadata>(&profile.user.metadata).unwrap();

    // check if we can update this user
    // must be authenticated AND same user OR staff
    let can_update: bool = (token_user.user.username == profile.user.username)
        | (token_user
            .level
            .permissions
            .contains(&String::from("ManageUsers")));

    if can_update == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this user's contents.");
    }

    // (check length)
    if (body.about.len() < 2) | (body.about.len() > 200_000) {
        return HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Content is invalid"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    // update about
    user.about = body.about.clone();

    // ...
    let res = data.db.edit_user_metadata_by_name(name, user).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/users/{name:.*}/secondary-token")]
pub async fn refresh_secondary_token_request(
    req: HttpRequest,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get token user
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    let token_user = token_user.unwrap().payload.unwrap();

    // make sure profile exists
    let profile: DefaultReturn<Option<FullUser<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if !profile.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Profile does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let profile = profile.payload.unwrap();
    let mut user = serde_json::from_str::<UserMetadata>(&profile.user.metadata).unwrap();

    // check if we can update this user
    // must be authenticated AND same user OR staff
    let can_update: bool = (token_user.user.username == profile.user.username)
        | (token_user
            .level
            .permissions
            .contains(&String::from("ManageUsers")));

    if can_update == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this user's contents.");
    }

    // update secondary token
    let token = crate::utility::uuid();
    user.secondary_token = Option::Some(crate::utility::hash(token.clone())); // this is essentially just a second ID the user can signin with

    // ...
    let res = data.db.edit_user_metadata_by_name(name, user).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<DefaultReturn<String>>(&DefaultReturn {
                success: res.success,
                message: res.message,
                payload: token,
            })
            .unwrap(),
        );
}

#[post("/api/auth/users/{name:.*}/follow")]
pub async fn follow_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get token user
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    let token_user = token_user.unwrap().payload.unwrap();

    // ...
    let res = data
        .db
        .toggle_user_follow(&mut UserFollow {
            user: token_user.user.username,
            is_following: name,
        })
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/users/{name:.*}/update")]
pub async fn update_request(
    req: HttpRequest,
    body: web::Json<UserMetadata>,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get token user
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    // make sure profile exists
    let profile: DefaultReturn<Option<FullUser<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if !profile.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Profile does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let token_user = token_user.unwrap().payload.unwrap();
    let profile = profile.payload.unwrap();

    // check if we can update this user
    // must be authenticated AND same user OR staff
    let can_update: bool = (token_user.user.username == profile.user.username)
        | (token_user
            .level
            .permissions
            .contains(&String::from("ManageUsers")));

    if can_update == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this user's contents.");
    }

    // ...
    let res = data
        .db
        .edit_user_metadata_by_name(
            name,            // select user
            body.to_owned(), // new metadata
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/users/{name:.*?}/ban")]
/// Ban user
pub async fn ban_request(req: HttpRequest, data: web::Data<bundlesdb::AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get token user
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    // make sure token_user is of role "staff"
    if !token_user
        .unwrap()
        .payload
        .unwrap()
        .level
        .permissions
        .contains(&String::from("ManageUsers"))
    {
        return HttpResponse::NotAcceptable().body("Only staff can do this");
    }

    // ban user
    let res: bundlesdb::DefaultReturn<Option<String>> = data.db.ban_user_by_name(name).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string::<bundlesdb::DefaultReturn<Option<String>>>(&res).unwrap());
}

#[get("/api/auth/users/{name:.*}/followers")]
pub async fn followers_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<crate::pages::boards::ViewBoardQueryProps>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get followers
    let res: DefaultReturn<Option<Vec<bundlesdb::Log>>> = data
        .db
        .get_user_followers(name.to_owned(), info.offset)
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string::<DefaultReturn<Option<Vec<bundlesdb::Log>>>>(&res).unwrap());
}

#[get("/api/auth/users/{name:.*}/following")]
pub async fn following_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<crate::pages::boards::ViewBoardQueryProps>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get following
    let res: DefaultReturn<Option<Vec<bundlesdb::Log>>> = data
        .db
        .get_user_following(name.to_owned(), info.offset)
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string::<DefaultReturn<Option<Vec<bundlesdb::Log>>>>(&res).unwrap());
}

#[get("/api/auth/users/{name:.*}/avatar")]
pub async fn avatar_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // make sure profile exists
    let profile: DefaultReturn<Option<FullUser<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if !profile.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Profile does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let profile = profile.payload.unwrap();
    let user = serde_json::from_str::<UserMetadata>(&profile.user.metadata).unwrap();

    if user.avatar_url.is_none() {
        return HttpResponse::NotFound().body("User does not have an avatar set");
    }

    let avatar = user.avatar_url.unwrap();

    // fetch avatar
    let res = data
        .http_client
        .get(avatar)
        .timeout(std::time::Duration::from_millis(5_000))
        .insert_header(("User-Agent", "stellular-bundlrs/1.0"))
        .send()
        .await;

    if res.is_err() {
        return HttpResponse::NotFound().body(format!(
            "Failed to fetch avatar on server: {}",
            res.err().unwrap()
        ));
    }

    // ...
    let mut res = res.unwrap();
    let body = res.body().limit(10_000_000).await;

    if body.is_err() {
        return HttpResponse::NotFound().body(
            "Failed to fetch avatar on server (image likely too large, please keep under 10 MB)",
        );
    }

    let body = body.unwrap();

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", res.content_type()))
        .body(body);
}

#[get("/api/auth/users/{name:.*?}/pastes")]
/// Get all pastes by owner
pub async fn get_from_owner_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
    info: web::Query<crate::pages::boards::ViewBoardQueryProps>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get pastes
    let res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::PasteIdentifier>>> =
        data.db.get_pastes_by_owner_limited(name, info.offset).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<
                bundlesdb::DefaultReturn<Option<Vec<bundlesdb::PasteIdentifier>>>,
            >(&res)
            .unwrap(),
        );
}

#[get("/api/auth/users/{name:.*}/level")]
pub async fn level_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get user
    let res: DefaultReturn<Option<FullUser<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if res.success == false {
        return HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<bundlesdb::RoleLevel>(&bundlesdb::RoleLevel {
                    elevation: -1000,
                    name: String::from("anonymous"),
                    permissions: Vec::new(),
                })
                .unwrap(),
            );
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string::<bundlesdb::RoleLevel>(&res.payload.unwrap().level).unwrap());
}
