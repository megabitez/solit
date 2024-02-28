use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::db::bundlesdb::{AppData, Board};

#[derive(serde::Deserialize)]
struct CreateInfo {
    name: String,
}

#[post("/api/board/new")]
pub async fn create_request(
    req: HttpRequest,
    body: web::Json<CreateInfo>,
    data: web::Data<AppData>,
) -> impl Responder {
    // get owner
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_hashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
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
        // if server requires an account, return
        let requires_account = crate::config::get_var("AUTH_REQUIRED");

        if requires_account.is_some() {
            return HttpResponse::NotAcceptable()
                .body("This server requires an account to create pastes");
        }
    }

    // ...
    let res = data
        .db
        .create_board(
            &mut Board {
                name: body.name.clone(),
                timestamp: 0,
                metadata: String::new(),
            },
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[get("/api/board/{name:.*}")]
pub async fn get_posts_request(
    req: HttpRequest,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // ...
    let res = data.db.get_board_posts(name).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}
