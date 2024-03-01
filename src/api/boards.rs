use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};

use crate::db::bundlesdb::{AppData, Board, BoardMetadata, BoardPostLog, DefaultReturn};

#[derive(serde::Deserialize)]
struct CreateInfo {
    name: String,
}

#[derive(serde::Deserialize)]
struct CreatePostInfo {
    content: String,
    reply: Option<String>,
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
        return HttpResponse::NotAcceptable().body("An account is required to do this");
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

#[get("/api/board/{name:.*}/posts")]
pub async fn get_posts_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    let board: DefaultReturn<Option<Board<String>>> = data.db.get_board_by_name(name.clone()).await;

    // get
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

    // check if board is private
    // if it is, only the owner and users with the "staff" role can view it
    if board.payload.is_some() {
        let metadata =
            serde_json::from_str::<BoardMetadata>(&board.payload.as_ref().unwrap().metadata)
                .unwrap();

        if metadata.is_private == String::from("yes") {
            // anonymous
            if token_user.is_none() {
                return HttpResponse::NotFound()
                    .body("You do not have permission to view this board's contents.");
            }

            // not owner
            let user = token_user.unwrap().payload.unwrap();

            if (user.username != metadata.owner) && (user.role != String::from("staff")) {
                return HttpResponse::NotFound()
                    .body("You do not have permission to view this board's contents.");
            }
        }
    }

    // ...
    let res = data.db.get_board_posts(name).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/board/{name:.*}/posts")]
pub async fn create_post_request(
    req: HttpRequest,
    body: web::Json<CreatePostInfo>,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

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
    }

    // ...
    let res = data
        .db
        .create_board_post(
            &mut BoardPostLog {
                author: String::new(),
                content: body.content.clone(), // use given content
                content_html: String::new(),
                board: name,
                is_hidden: false,
                reply: if body.reply.is_some() {
                    Option::Some(body.reply.as_ref().unwrap().to_string())
                } else {
                    Option::None
                },
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

#[delete("/api/board/{name:.*}/posts/{id:.*}")]
pub async fn delete_post_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();
    let id: String = req.match_info().get("id").unwrap().to_string();

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
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    // check if we can delete this post
    let board: DefaultReturn<Option<Board<String>>> =
        data.db.get_board_by_name(name.to_owned()).await;

    if !board.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Board does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let board = serde_json::from_str::<BoardMetadata>(&board.payload.unwrap().metadata).unwrap();

    // get post
    let p = data.db.get_log_by_id(id.to_owned()).await;
    let post = serde_json::from_str::<BoardPostLog>(&p.payload.unwrap().content).unwrap();

    // check if we can delete this post
    // must be authenticated AND board owner OR staff OR post author
    let user = token_user.unwrap().payload.unwrap();

    let can_delete: bool = (user.username != String::from("Anonymous"))
        && ((user.username == board.owner)
            | (user.role == String::from("staff"))
            | (user.username == post.author));

    if can_delete == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this board's contents.");
    }

    // ...
    let res = data.db.delete_log(id).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/board/{name:.*}/update")]
pub async fn metadata_request(
    req: HttpRequest,
    body: web::Json<BoardMetadata>,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get board
    let board: DefaultReturn<Option<Board<String>>> = data.db.get_board_by_name(name.clone()).await;

    if board.success == false {
        return HttpResponse::NotFound().body(board.message);
    }

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
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    // make sure we have permission to do this
    let metadata =
        serde_json::from_str::<BoardMetadata>(&board.payload.as_ref().unwrap().metadata).unwrap();

    let user = token_user.as_ref().unwrap().payload.as_ref().unwrap();
    let can_edit: bool = (user.username == metadata.owner) | (user.role == String::from("staff"));

    if can_edit == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this board's contents.");
    }

    // ...
    let res = data
        .db
        .edit_board_metadata_by_name(
            name,            // select board
            body.to_owned(), // new metadata
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