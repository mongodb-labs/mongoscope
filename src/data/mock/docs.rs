use crate::data::model::{BsonDoc, BsonVal, Op};
use indexmap::IndexMap;

fn make_doc(coll: &str, i: usize) -> BsonDoc {
    let mut doc: BsonDoc = IndexMap::new();
    doc.insert(
        "_id".into(),
        BsonVal::ObjectId(format!("66f{:02x}c4a8b4e9c2d1f0{:02x}", i + 1, i + 1)),
    );
    match coll {
        "orders" => {
            doc.insert(
                "userId".into(),
                BsonVal::ObjectId("65fe21c3a8b4e9c2d1f04a12".into()),
            );
            doc.insert("total".into(), BsonVal::Float(49.0 + i as f64 * 17.33));
            doc.insert(
                "status".into(),
                BsonVal::Str(["paid", "pending", "shipped"][i % 3].into()),
            );
            doc.insert(
                "createdAt".into(),
                BsonVal::IsoDate(format!("2026-04-{}T09:{:02}:41Z", 20 + i % 10, 12 + i % 48)),
            );
        }
        "products" => {
            let names = [
                "Linen Shirt",
                "Field Jacket",
                "Canvas Sneakers",
                "Wool Coat",
                "Leather Belt",
            ];
            doc.insert(
                "sku".into(),
                BsonVal::Str(format!("SKU-{}-BLK-M", 88421 + i)),
            );
            doc.insert("name".into(), BsonVal::Str(names[i % names.len()].into()));
            doc.insert("price".into(), BsonVal::Int(49 + i as i64 * 20));
            doc.insert("inStock".into(), BsonVal::Bool(!i.is_multiple_of(3)));
        }
        "users" => {
            doc.insert(
                "email".into(),
                BsonVal::Str(format!("user{}@example.com", 1000 + i)),
            );
            doc.insert("name".into(), BsonVal::Str(format!("User {}", 1000 + i)));
            doc.insert(
                "role".into(),
                BsonVal::Str(
                    if i.is_multiple_of(10) {
                        "admin"
                    } else {
                        "customer"
                    }
                    .into(),
                ),
            );
            doc.insert(
                "createdAt".into(),
                BsonVal::IsoDate(format!("2025-0{}T00:00:00Z", 1 + i % 9)),
            );
        }
        "carts" => {
            doc.insert(
                "userId".into(),
                BsonVal::ObjectId(format!("65fe21c3a8b4e9c2d1f0{:04x}", 100 + i)),
            );
            doc.insert(
                "updatedAt".into(),
                BsonVal::IsoDate(format!("2026-04-24T{}:00:00Z", 10 + i % 14)),
            );
        }
        "sessions" => {
            doc.insert(
                "ip".into(),
                BsonVal::Str(format!("192.168.{}.{}", i % 256, (i * 7) % 256)),
            );
            doc.insert(
                "expiresAt".into(),
                BsonVal::IsoDate(format!("2026-05-{}T00:00:00Z", 1 + i % 30)),
            );
        }
        "reviews" => {
            doc.insert(
                "productId".into(),
                BsonVal::ObjectId(format!("66f{:02x}b3a8c4e9d2f1e0{:02x}", i + 1, i + 5)),
            );
            doc.insert("rating".into(), BsonVal::Int(1 + (i % 5) as i64));
            doc.insert(
                "body".into(),
                BsonVal::Str(["Great product!", "As expected", "Would recommend"][i % 3].into()),
            );
        }
        "inventory" => {
            doc.insert(
                "sku".into(),
                BsonVal::Str(format!("SKU-{}-BLK-M", 88421 + i)),
            );
            doc.insert("qty".into(), BsonVal::Int(100 + i as i64 * 13));
            doc.insert(
                "warehouse".into(),
                BsonVal::Str(["east-1", "west-2", "central-3"][i % 3].into()),
            );
        }
        "events" => {
            doc.insert(
                "type".into(),
                BsonVal::Str(["click", "view", "purchase", "add_to_cart"][i % 4].into()),
            );
            doc.insert(
                "ts".into(),
                BsonVal::IsoDate(format!("2026-04-24T{}:{:02}:08Z", 14 + i % 10, i % 60)),
            );
        }
        _ => {
            doc.insert("type".into(), BsonVal::Str("generic".into()));
            doc.insert(
                "ts".into(),
                BsonVal::IsoDate(format!("2026-04-24T{}:22:08Z", 14 + i % 10)),
            );
        }
    }
    doc
}

pub fn gen_response_docs(coll: &str, op: &Op, n: usize) -> Vec<BsonDoc> {
    let is_read = matches!(
        op,
        Op::Find | Op::FindOne | Op::Aggregate | Op::CountDocuments
    );
    if is_read {
        let count = n.min(5);
        (0..count).map(|i| make_doc(coll, i)).collect()
    } else {
        // Write ops return an empty vec; the response is synthesized from model metadata
        vec![]
    }
}
