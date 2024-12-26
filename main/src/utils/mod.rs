use std::error::Error;
use crate::{ db::execute_query, models::{ OrderData, OrderPubData } };
use std::fs;
use serde_json::Value;

pub async fn update_orders_buy(
    mut current_order: OrderData,
    mut current_order_id: i32,
    user_id: i32
) -> Result<i32, Box<dyn Error>> {
    let query_get_all_orders = format!(
        "SELECT orders.order_id, orders.user_id, orders.pair_id, orders.quantity, orders.price, orders.order_type, orders.closed FROM orders WHERE orders.pair_id = {} AND orders.closed IS NULL AND orders.order_type = 'sell'",
        current_order.pair_id
    );

    let query_get_lots_ids = format!(
        "SELECT pair.sale_lot_id, pair.buy_lot_id FROM pair WHERE pair.pair_id = {}",
        current_order.pair_id
    );

    let result = execute_query(query_get_lots_ids).await?;
    let lot_ids = result
        .trim()
        .split_whitespace()
        .map(|x| x.parse::<i32>())
        .collect::<Result<Vec<i32>, _>>()?;

    if lot_ids.len() != 2 {
        return Err(
            format!(
                "Invalid pair_id: {}. Expected 2 lot IDs, found {}.",
                current_order.pair_id,
                lot_ids.len()
            ).into()
        );
    }

    let mut matched_orders: Vec<OrderPubData> = Vec::new();
    let orders = execute_query(query_get_all_orders).await?;
    for order in orders.lines() {
        let order_parts: Vec<&str> = order.split_whitespace().collect();
        if order_parts.len() == 7 {
            let order_data = OrderPubData {
                order_id: order_parts[0].parse().unwrap_or_default(),
                user_id: order_parts[1].parse().unwrap_or_default(),
                pair_id: order_parts[2].parse().unwrap_or_default(),
                quantity: order_parts[3].parse().unwrap_or_default(),
                price: order_parts[4].parse().unwrap_or_default(),
                order_type: order_parts[5].to_string(),
                closed: order_parts[6].parse().unwrap_or_default(),
            };
            matched_orders.push(order_data);
        }
    }

    for order in matched_orders {
        if order.price <= current_order.price {
            if order.quantity == current_order.quantity {
                update_balance(user_id, lot_ids[1], current_order.quantity).await?;
                update_balance(user_id, lot_ids[0], -current_order.quantity * order.price).await?;
                update_balance(order.user_id, lot_ids[1], -current_order.quantity).await?;
                update_balance(
                    order.user_id,
                    lot_ids[0],
                    current_order.quantity * order.price
                ).await?;
                // update(покупаемый у текущего пользователя на +значение(current))
                // update(продаваемый у текущего пользователя на -значение(current))
                // update(покупаемый у продавца на -значение (current))
                // update(продаваемый у продавца на + значение (current)))
                let query_delete_order = format!(
                    "DELETE FROM orders WHERE orders.order_id = {} OR orders.order_id = {}",
                    order.order_id,
                    current_order_id
                );
                let query_insert_updated_order = format!(
                    "INSERT INTO orders VALUES ({}, {}, {}, {}, '{}', 'closed'), ({}, {}, {}, {}, '{}', 'closed')",
                    order.user_id,
                    order.pair_id,
                    order.quantity,
                    order.price,
                    order.order_type,
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );
                execute_query(query_delete_order).await?;
                execute_query(query_insert_updated_order).await?;
                let query_get_order_id = format!(
                    "SELECT orders.order_id FROM orders WHERE orders.user_id = {} AND orders.pair_id = {} AND orders.quantity = {} AND orders.price = {} AND orders.order_type = '{}'",
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );

                current_order_id = execute_query(query_get_order_id).await?
                    .trim()
                    .parse()
                    .map_err(|_| "Parse int error")?;
                return Ok(current_order_id);
            } else if order.quantity > current_order.quantity {
                update_balance(user_id, lot_ids[1], current_order.quantity).await?;
                update_balance(user_id, lot_ids[0], -current_order.quantity * order.price).await?;
                update_balance(order.user_id, lot_ids[1], -current_order.quantity).await?;
                update_balance(
                    order.user_id,
                    lot_ids[0],
                    current_order.quantity * order.price
                ).await?;
                // update(покупаемый у текущего пользователя на +значение(current))
                // update(продаваемый у текущего пользователя на -значение(current))
                // update(покупаемый у продавца на -значение (current))
                // update(продаваемый у продавца на + значение (current)))
                let query_delete_order = format!(
                    "DELETE FROM orders WHERE orders.order_id = {} OR orders.order_id = {}",
                    order.order_id,
                    current_order_id
                );
                let query_insert_updated_order = format!(
                    "INSERT INTO orders VALUES ({}, {}, {}, {}, '{}', NULL), ({}, {}, {}, {}, '{}', 'closed')",
                    order.user_id,
                    order.pair_id,
                    order.quantity - current_order.quantity,
                    order.price,
                    order.order_type,
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );
                execute_query(query_delete_order).await?;
                execute_query(query_insert_updated_order).await?;
                let query_get_order_id = format!(
                    "SELECT orders.order_id FROM orders WHERE orders.user_id = {} AND orders.pair_id = {} AND orders.quantity = {} AND orders.price = {} AND orders.order_type = '{}'",
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );

                current_order_id = execute_query(query_get_order_id).await?
                    .trim()
                    .parse()
                    .map_err(|_| "Parse int error")?;
                return Ok(current_order_id);
            } else if order.quantity < current_order.quantity {
                update_balance(user_id, lot_ids[1], order.quantity).await?;
                update_balance(user_id, lot_ids[0], -order.quantity * order.price).await?;
                update_balance(order.user_id, lot_ids[1], -order.quantity).await?;
                update_balance(order.user_id, lot_ids[0], order.quantity * order.price).await?;
                // update(покупаемый у текущего пользователя на +значение(order))
                // update(продаваемый у текущего пользователя на -значение(order))
                // update(покупаемый у продавца на -значение (order))
                // update(продаваемый у продавца на + значение (order)))
                let query_delete_order = format!(
                    "DELETE FROM orders WHERE orders.order_id = {} OR orders.order_id = {}",
                    order.order_id,
                    current_order_id
                );
                let query_insert_updated_order = format!(
                    "INSERT INTO orders VALUES ({}, {}, {}, {}, '{}', 'closed'), ({}, {}, {}, {}, '{}', NULL)",
                    order.user_id,
                    order.pair_id,
                    order.quantity,
                    order.price,
                    order.order_type,
                    user_id,
                    current_order.pair_id,
                    current_order.quantity - order.quantity,
                    current_order.price,
                    current_order.order_type
                );
                current_order.quantity = current_order.quantity - order.quantity;
                execute_query(query_delete_order).await?;
                execute_query(query_insert_updated_order).await?;

                let query_get_order_id = format!(
                    "SELECT orders.order_id FROM orders WHERE orders.user_id = {} AND orders.pair_id = {} AND orders.quantity = {} AND orders.price = {} AND orders.order_type = '{}'",
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );

                current_order_id = execute_query(query_get_order_id).await?
                    .trim()
                    .parse()
                    .map_err(|_| "Parse int error")?;
            }
        }
    }
    Ok(current_order_id)
}
pub async fn update_orders_sell(
    mut current_order: OrderData,
    mut current_order_id: i32,
    user_id: i32
) -> Result<i32, Box<dyn Error>> {
    let query_get_all_orders = format!(
        "SELECT orders.order_id, orders.user_id, orders.pair_id, orders.quantity, orders.price, orders.order_type, orders.closed FROM orders WHERE orders.pair_id = {} AND orders.closed IS NULL AND orders.order_type = 'buy'",
        current_order.pair_id
    );
    let query_get_lots_ids = format!(
        "SELECT pair.sale_lot_id, pair.buy_lot_id FROM pair WHERE pair.pair_id = {}",
        current_order.pair_id
    );

    let result = execute_query(query_get_lots_ids).await?;
    let lot_ids = result
        .trim()
        .split_whitespace()
        .map(|x| x.parse::<i32>())
        .collect::<Result<Vec<i32>, _>>()?;

    if lot_ids.len() != 2 {
        return Err(
            format!(
                "Invalid pair_id: {}. Expected 2 lot IDs, found {}.",
                current_order.pair_id,
                lot_ids.len()
            ).into()
        );
    }

    let mut matched_orders: Vec<OrderPubData> = Vec::new();
    let orders = execute_query(query_get_all_orders).await?;
    for order in orders.lines() {
        let order_parts: Vec<&str> = order.split_whitespace().collect();
        if order_parts.len() == 7 {
            let order_data = OrderPubData {
                order_id: order_parts[0].parse().unwrap_or_default(),
                user_id: order_parts[1].parse().unwrap_or_default(),
                pair_id: order_parts[2].parse().unwrap_or_default(),
                quantity: order_parts[3].parse().unwrap_or_default(),
                price: order_parts[4].parse().unwrap_or_default(),
                order_type: order_parts[5].to_string(),
                closed: order_parts[6].parse().unwrap_or_default(),
            };
            matched_orders.push(order_data);
        }
    }

    for order in matched_orders {
        if order.price >= current_order.price {
            if order.quantity == current_order.quantity {
                update_balance(user_id, lot_ids[1], -current_order.quantity * order.price).await?;
                update_balance(user_id, lot_ids[0], current_order.quantity).await?;
                update_balance(
                    order.user_id,
                    lot_ids[1],
                    current_order.quantity * order.price
                ).await?;
                update_balance(order.user_id, lot_ids[0], -current_order.quantity).await?;
                // update(покупаемый у текущего пользователя на -значение(current))
                // update(продаваемый у текущего пользователя на +значение(current))
                // update(покупаемый у продавца на +значение (current))
                // update(продаваемый у продавца на -значение (current)))
                let query_delete_order = format!(
                    "DELETE FROM orders WHERE orders.order_id = {} OR orders.order_id = {}",
                    order.order_id,
                    current_order_id
                );
                let query_insert_updated_order = format!(
                    "INSERT INTO orders VALUES ({}, {}, {}, {}, '{}', 'closed'), ({}, {}, {}, {}, '{}', 'closed')",
                    order.user_id,
                    order.pair_id,
                    order.quantity,
                    order.price,
                    order.order_type,
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );
                execute_query(query_delete_order).await?;
                execute_query(query_insert_updated_order).await?;
                let query_get_order_id = format!(
                    "SELECT orders.order_id FROM orders WHERE orders.user_id = {} AND orders.pair_id = {} AND orders.quantity = {} AND orders.price = {} AND orders.order_type = '{}'",
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );

                current_order_id = execute_query(query_get_order_id).await?
                    .trim()
                    .parse()
                    .map_err(|_| "Parse int error")?;
                return Ok(current_order_id);
            } else if order.quantity > current_order.quantity {
                update_balance(user_id, lot_ids[1], -current_order.quantity * order.price).await?;
                update_balance(user_id, lot_ids[0], current_order.quantity).await?;
                update_balance(
                    order.user_id,
                    lot_ids[1],
                    current_order.quantity * order.price
                ).await?;
                update_balance(order.user_id, lot_ids[0], -current_order.quantity).await?;
                // update(покупаемый у текущего пользователя на -значение(current))
                // update(продаваемый у текущего пользователя на +значение(current))
                // update(покупаемый у продавца на +значение (current))
                // update(продаваемый у продавца на -значение (current)))
                let query_delete_order = format!(
                    "DELETE FROM orders WHERE orders.order_id = {} OR orders.order_id = {}",
                    order.order_id,
                    current_order_id
                );
                let query_insert_updated_order = format!(
                    "INSERT INTO orders VALUES ({}, {}, {}, {}, '{}', NULL), ({}, {}, {}, {}, '{}', 'closed')",
                    order.user_id,
                    order.pair_id,
                    order.quantity - current_order.quantity,
                    order.price,
                    order.order_type,
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );
                execute_query(query_delete_order).await?;
                execute_query(query_insert_updated_order).await?;
                let query_get_order_id = format!(
                    "SELECT orders.order_id FROM orders WHERE orders.user_id = {} AND orders.pair_id = {} AND orders.quantity = {} AND orders.price = {} AND orders.order_type = '{}'",
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );

                current_order_id = execute_query(query_get_order_id).await?
                    .trim()
                    .parse()
                    .map_err(|_| "Parse int error")?;
                return Ok(current_order_id);
            } else if order.quantity < current_order.quantity {
                update_balance(user_id, lot_ids[1], -order.quantity * order.price).await?;
                update_balance(user_id, lot_ids[0], order.quantity).await?;
                update_balance(order.user_id, lot_ids[1], order.quantity * order.price).await?;
                update_balance(order.user_id, lot_ids[0], -order.quantity).await?;
                // update(покупаемый у текущего пользователя на -значение(order))
                // update(продаваемый у текущего пользователя на +значение(order))
                // update(покупаемый у продавца на +значение (order))
                // update(продаваемый у продавца на -значение (order)))
                let query_delete_order = format!(
                    "DELETE FROM orders WHERE orders.order_id = {} OR orders.order_id = {}",
                    order.order_id,
                    current_order_id
                );
                let query_insert_updated_order = format!(
                    "INSERT INTO orders VALUES ({}, {}, {}, {}, '{}', 'closed'), ({}, {}, {}, {}, '{}', NULL)",
                    order.user_id,
                    order.pair_id,
                    order.quantity,
                    order.price,
                    order.order_type,
                    user_id,
                    current_order.pair_id,
                    current_order.quantity - order.quantity,
                    current_order.price,
                    current_order.order_type
                );
                current_order.quantity = current_order.quantity - order.quantity;
                execute_query(query_delete_order).await?;
                execute_query(query_insert_updated_order).await?;

                let query_get_order_id = format!(
                    "SELECT orders.order_id FROM orders WHERE orders.user_id = {} AND orders.pair_id = {} AND orders.quantity = {} AND orders.price = {} AND orders.order_type = '{}'",
                    user_id,
                    current_order.pair_id,
                    current_order.quantity,
                    current_order.price,
                    current_order.order_type
                );

                current_order_id = execute_query(query_get_order_id).await?
                    .trim()
                    .parse()
                    .map_err(|_| "Parse int error")?;
            }
        }
    }
    Ok(current_order_id)
}

pub async fn init_db() -> Result<(), Box<dyn Error>> {
    // Читаем содержимое config.json
    let config_content = fs::read_to_string(
        "/home/kali/Desktop/VSCode_files/STUDY/prak_3/config.json"
    )?;
    let config: Value = serde_json::from_str(&config_content)?;

    // Получаем список лотов из конфигурации
    let lots = config["lots"].as_array().ok_or("Lots not found in config")?;

    // Получаем все существующие пары из базы данных
    let query_get_all_pairs = "SELECT pair.sale_lot_id, pair.buy_lot_id FROM pair".to_string();
    let existing_pairs: Vec<(i32, i32)> = execute_query(query_get_all_pairs).await?
        .lines()
        .map(|pair| {
            let parts: Vec<&str> = pair.split_whitespace().collect();
            if parts.len() == 2 {
                (parts[0].parse().unwrap_or_default(), parts[1].parse().unwrap_or_default())
            } else {
                (0, 0) // Заменяем ошибочные данные на 0, 0
            }
        })
        .collect();

    // Получаем все существующие лоты из базы данных
    let query_get_all_lots = "SELECT lot.name, lot.lot_id FROM lot".to_string();
    let existing_lots: Vec<(String, i32)> = execute_query(query_get_all_lots).await?
        .lines()
        .map(|lot| {
            let parts: Vec<&str> = lot.split_whitespace().collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].parse().unwrap_or_default())
            } else {
                (String::new(), 0) // Заменяем ошибочные данные на пустую строку и 0
            }
        })
        .collect();

    // Создаем уникальные лоты
    for lot in lots {
        let lot_name = lot.as_str().ok_or("Invalid lot name")?;
        if !existing_lots.iter().any(|(name, _)| name == lot_name) {
            let query_insert_lot = format!("INSERT INTO lot (name) VALUES ('{}')", lot_name);
            execute_query(query_insert_lot).await?;
        }
    }

    // Обновляем список лотов после возможного добавления новых
    let query_get_all_lots = "SELECT lot.name, lot.lot_id FROM lot".to_string();
    let updated_lots: Vec<(String, i32)> = execute_query(query_get_all_lots).await?
        .lines()
        .map(|lot| {
            let parts: Vec<&str> = lot.split_whitespace().collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].parse().unwrap_or_default())
            } else {
                (String::new(), 0)
            }
        })
        .collect();

    // Создаем все возможные пары
    for i in 0..updated_lots.len() {
        for j in i + 1..updated_lots.len() {
            let (_, lot1_id) = &updated_lots[i];
            let (_, lot2_id) = &updated_lots[j];

            if !existing_pairs.contains(&(*lot1_id, *lot2_id)) {
                let query = format!(
                    "INSERT INTO pair (sale_lot_id, buy_lot_id) VALUES ({}, {})",
                    lot1_id,
                    lot2_id
                );
                execute_query(query).await?;
            }
            if !existing_pairs.contains(&(*lot2_id, *lot1_id)) {
                let query = format!(
                    "INSERT INTO pair (sale_lot_id, buy_lot_id) VALUES ({}, {})",
                    lot2_id,
                    lot1_id
                );
                execute_query(query).await?;
            }
        }
    }

    Ok(())
}

pub async fn set_start_balance(username: &String) -> Result<(), Box<dyn Error>> {
    // Читаем содержимое config.json
    let config_content = fs::read_to_string(
        "/home/kali/Desktop/VSCode_files/STUDY/prak_3/config.json"
    )?;
    let config: Value = serde_json::from_str(&config_content)?;

    // Получаем id пользователя по имени
    let query_get_user_id =
        format!("SELECT users.user_id FROM users WHERE users.username = '{}'", username);
    let user_id: i32 = execute_query(query_get_user_id).await?
        .trim()
        .parse()
        .map_err(|_| "Failed to parse user_id")?;

    // Получаем список лотов
    let lots = config["lots"].as_array().ok_or("Lots not found in config")?;
    for lot_name in lots {
        let lot_name_str = lot_name.as_str().ok_or("Invalid lot name in config")?;

        // Получаем lot_id по имени lot_name
        let query_get_lot_id =
            format!("SELECT lot.lot_id FROM lot WHERE lot.name = '{}'", lot_name_str);
        let lot_id: i32 = execute_query(query_get_lot_id).await?
            .trim()
            .parse()
            .map_err(|_| format!("Failed to fetch lot_id for lot_name: {}", lot_name_str))?;

        // Вставляем начальный баланс пользователя для найденного lot_id
        let query_insert_balance = format!(
            "INSERT INTO user_lot (user_id, lot_id, quantity) VALUES ({}, {}, {})",
            user_id,
            lot_id,
            1000
        );
        execute_query(query_insert_balance).await?;
    }

    Ok(())
}

pub async fn update_balance(
    user_id: i32,
    lot_id: i32,
    quantity_change: f32
) -> Result<(), Box<dyn Error>> {
    // Получение текущего баланса пользователя для указанного lot_id
    let query_get_balance = format!(
        "SELECT user_lot.quantity FROM user_lot WHERE user_lot.user_id = {} AND user_lot.lot_id = {}",
        user_id,
        lot_id
    );
    print!("ok1");

    let current_balance: f32 = execute_query(query_get_balance).await?
        .trim()
        .parse()
        .unwrap_or(0.0); // Если баланс отсутствует, подразумевается 0.0
    print!("ok2");

    // Вычисляем новый баланс
    let new_balance = current_balance + quantity_change;

    // Проверяем, не уходит ли баланс в минус
    if new_balance < 0.0 {
        return Err(
            format!(
                "Insufficient balance for user_id: {} on lot_id: {}. Current balance: {}, required: {}",
                user_id,
                lot_id,
                current_balance,
                quantity_change.abs()
            ).into()
        );
    }

    // Удаляем старую запись баланса
    let query_delete_balance = format!(
        "DELETE FROM user_lot WHERE user_lot.user_id = {} AND user_lot.lot_id = {}",
        user_id,
        lot_id
    );
    execute_query(query_delete_balance).await?;
    // Добавляем новую запись с обновленным балансом
    let query_insert_balance = format!(
        "INSERT INTO user_lot (user_id, lot_id, quantity) VALUES ({}, {}, {})",
        user_id,
        lot_id,
        new_balance
    );
    execute_query(query_insert_balance).await?;

    Ok(())
}
