use actix_web::{ web, HttpResponse, HttpRequest };
use uuid::Uuid;
use crate::{
    db::execute_query,
    models::{ OrderData, OrderId, OrderPubData, PairPubData, LotPubData, RegisterData },
    utils::{ update_orders_buy, update_orders_sell, set_start_balance },
};

//POST
pub async fn create_user(user_data: web::Json<RegisterData>) -> HttpResponse {
    let username = &user_data.username;
    if username.trim().is_empty() {
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "Username cannot be empty"})
        );
    }

    let user_key = Uuid::new_v4().to_string();

    let query = format!("INSERT INTO users VALUES ({}, {})", username.replace("'", ""), user_key);

    match execute_query(query).await {
        Ok(_) => {
            set_start_balance(username).await.unwrap();
            HttpResponse::Created().json(serde_json::json!({"key": user_key}))
        }
        Err(e) =>
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

//POST
pub async fn create_order(
    current_order_data: web::Json<OrderData>,
    req: HttpRequest
) -> HttpResponse {
    let user_key = match
        req
            .headers()
            .get("X-USER-KEY")
            .and_then(|key| key.to_str().ok())
    {
        Some(key) => key,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Missing or invalid X-USER-KEY"})
            );
        }
    };

    let query_check_user = format!(
        "SELECT users.user_id FROM users WHERE users.auth_key = '{}'",
        user_key.replace("'", "")
    );

    let user_id: i32 = match execute_query(query_check_user).await {
        Ok(user_id) =>
            match user_id.trim().parse() {
                Ok(user_id) => user_id,
                Err(_) => {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Parse int error"})
                    );
                }
            }
        Err(_) => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Invalid user key"})
            );
        }
    };

    let query_create_order = format!(
        "INSERT INTO orders VALUES ({}, {}, {}, {}, '{}', NULL)",
        user_id,
        current_order_data.pair_id,
        current_order_data.quantity,
        current_order_data.price,
        current_order_data.order_type.replace("'", "")
    );

    let current_order_id = match execute_query(query_create_order).await {
        Ok(_) => {
            let query_get_order_id = format!(
                "SELECT orders.order_id FROM orders WHERE orders.user_id = {} AND orders.pair_id = {} AND orders.quantity = {} AND orders.price = {} AND orders.order_type = '{}'",
                user_id,
                current_order_data.pair_id,
                current_order_data.quantity,
                current_order_data.price,
                current_order_data.order_type.replace("'", "")
            );

            let current_order_id = match execute_query(query_get_order_id).await {
                Ok(order_id) =>
                    match order_id.trim().parse::<i32>() {
                        Ok(order_id) => order_id,
                        Err(_) => {
                            return HttpResponse::InternalServerError().json(
                                serde_json::json!({"error": "Parse int error"})
                            );
                        }
                    }
                Err(e) => {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": e.to_string()})
                    );
                }
            };

            current_order_id
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": e.to_string()})
            );
        }
    };

    let current_order = OrderData {
        pair_id: current_order_data.pair_id,
        quantity: current_order_data.quantity,
        price: current_order_data.price,
        order_type: current_order_data.order_type.clone(),
    };
    if current_order.order_type == "buy" {
        match update_orders_buy(current_order, current_order_id, user_id).await {
            Ok(current_order_id) => {
                return HttpResponse::Ok().json(serde_json::json!({"order_id": current_order_id}));
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": e.to_string()})
                );
            }
        };
    } else {
        match update_orders_sell(current_order, current_order_id, user_id).await {
            Ok(current_order_id) => {
                return HttpResponse::Ok().json(serde_json::json!({"order_id": current_order_id}));
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": e.to_string()})
                );
            }
        };
    }
}

//GET
pub async fn get_orders() -> HttpResponse {
    let query_get_orders = String::from(
        "SELECT orders.order_id, orders.user_id, orders.pair_id, orders.quantity, orders.order_type, orders.price, orders.closed FROM orders"
    );
    match execute_query(query_get_orders).await {
        Ok(orders) => {
            let mut parsed_orders: Vec<OrderPubData> = Vec::new();
            for order in orders.lines() {
                let order_parts: Vec<&str> = order.split_whitespace().collect();
                if order_parts.len() == 7 {
                    let order_data = OrderPubData {
                        order_id: order_parts[0].parse().unwrap_or_default(),
                        user_id: order_parts[1].parse().unwrap_or_default(),
                        pair_id: order_parts[2].parse().unwrap_or_default(),
                        quantity: order_parts[3].parse().unwrap_or_default(),
                        order_type: order_parts[4].to_string(),
                        price: order_parts[5].parse().unwrap_or_default(),
                        closed: order_parts[6].parse().unwrap_or_default(),
                    };
                    parsed_orders.push(order_data);
                }
            }
            HttpResponse::Ok().json(parsed_orders)
        }
        Err(e) =>
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

//DELETE
pub async fn delete_order(order_id: web::Json<OrderId>, req: HttpRequest) -> HttpResponse {
    let user_key = match
        req
            .headers()
            .get("X-USER-KEY")
            .and_then(|key| key.to_str().ok())
    {
        Some(key) => key,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Missing or invalid X-USER-KEY"})
            );
        }
    };
    let query_check_user = format!(
        "SELECT users.user_id FROM users WHERE users.auth_key = {}",
        user_key.replace("'", "")
    );

    let user_id: i32 = match execute_query(query_check_user).await {
        Ok(user_id) =>
            match user_id.trim().parse() {
                Ok(user_id) => user_id,
                Err(_) => {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Parse int error"})
                    );
                }
            }
        Err(_) => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Invalid user key"})
            );
        }
    };
    let query_delete_order = format!(
        "DELETE FROM orders WHERE orders.order_id = {} AND orders.user_id = {}",
        order_id.order_id,
        user_id
    );
    match execute_query(query_delete_order).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) =>
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

//GET
pub async fn get_pairs() -> HttpResponse {
    let query_get_pairs = String::from(
        "SELECT pair.pair_id, pair.sale_lot_id, pair.buy_lot_id FROM pair"
    );

    match execute_query(query_get_pairs).await {
        Ok(pairs) => {
            let mut parsed_pairs: Vec<PairPubData> = Vec::new();
            for pair in pairs.lines() {
                let pair_parts: Vec<&str> = pair.split_whitespace().collect();
                if pair_parts.len() == 3 {
                    let pair_data = PairPubData {
                        pair_id: pair_parts[0].parse().unwrap_or_default(),
                        sale_lot_id: pair_parts[1].parse().unwrap_or_default(),
                        buy_lot_id: pair_parts[2].parse().unwrap_or_default(),
                    };
                    parsed_pairs.push(pair_data);
                }
            }
            HttpResponse::Ok().json(parsed_pairs)
        }
        Err(e) =>
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

//GET
pub async fn get_lots() -> HttpResponse {
    let query_get_lots = String::from("SELECT lot.lot_id, lot.name FROM lot");

    match execute_query(query_get_lots).await {
        Ok(lots) => {
            let mut parsed_lots: Vec<LotPubData> = Vec::new();
            for lot in lots.lines() {
                let lot_parts: Vec<&str> = lot.split_whitespace().collect();
                if lot_parts.len() == 2 {
                    let lot_data = LotPubData {
                        lot_id: lot_parts[0].parse().unwrap_or_default(),
                        name: lot_parts[1].parse().unwrap_or_default(),
                    };
                    parsed_lots.push(lot_data);
                }
            }
            HttpResponse::Ok().json(parsed_lots)
        }
        Err(e) =>
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

//GET
pub async fn get_balance() -> HttpResponse {
    HttpResponse::Ok().finish()
}
