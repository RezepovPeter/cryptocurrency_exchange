use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize)]
pub struct RegisterData {
    pub username: String,
}

#[derive(Deserialize, Serialize)]
pub struct OrderData {
    pub pair_id: i32,
    pub quantity: f32,
    pub price: f32,
    pub order_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct OrderPubData {
    pub order_id: i32,
    pub user_id: i32,
    pub pair_id: i32,
    pub quantity: f32,
    pub order_type: String,
    pub price: f32,
    pub closed: String,
}

#[derive(Deserialize, Serialize)]
pub struct OrderId {
    pub order_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct PairPubData {
    pub pair_id: i32,
    pub sale_lot_id: String,
    pub buy_lot_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct LotPubData {
    pub lot_id: i32,
    pub name: String,
}
