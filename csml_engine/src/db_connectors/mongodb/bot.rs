use crate::{db_connectors::DbBot, CsmlBot, EngineError, SerializeCsmlBot};
use bson::{doc, Bson};
use chrono::SecondsFormat;

fn format_bot_struct(bot: bson::ordered::OrderedDocument) -> Result<DbBot, EngineError> {
    Ok(DbBot {
        id: bot.get_object_id("_id").unwrap().to_hex(),
        bot_id: bot.get_str("bot_id").unwrap().to_owned(),
        bot: bot.get_str("bot").unwrap().to_owned(),
        engine_version: bot.get_str("engine_version").unwrap().to_owned(),
        created_at: bot
            .get_utc_datetime("created_at")
            .unwrap()
            .to_rfc3339_opts(SecondsFormat::Millis, true),
    })
}

pub fn create_bot_version(
    bot_id: String,
    bot: String,
    db: &mongodb::Database,
) -> Result<String, EngineError> {
    let collection = db.collection("bot");
    let time = Bson::UtcDatetime(chrono::Utc::now());

    let bot = doc! {
        "bot_id": bot_id,
        "bot": bot,
        "engine_version": env!("CARGO_PKG_VERSION").to_owned(),
        "created_at": &time
    };

    let inserted = collection.insert_one(bot.clone(), None)?;

    let id = inserted.inserted_id.as_object_id().unwrap();

    Ok(id.to_hex())
}

pub fn get_bot_versions(
    bot_id: &str,
    last_key: Option<String>,
    db: &mongodb::Database,
) -> Result<serde_json::Value, EngineError> {
    let collection = db.collection("bot");

    let filter = match last_key {
        Some(key) => {
            doc! {
                "bot_id": bot_id,
                "_id": {"$gt": bson::oid::ObjectId::with_string(&key).unwrap() }
            }
        }
        None => doc! {"bot_id": bot_id },
    };

    let find_options = mongodb::options::FindOptions::builder()
        .sort(doc! { "$natural": -1, })
        .batch_size(10)
        .limit(10)
        .build();

    let cursor = collection.find(filter, find_options)?;
    let mut bots = vec![];
    let mut last_key = None;

    for doc in cursor {
        match doc {
            Ok(bot) => {
                let bot = format_bot_struct(bot)?;

                let base64decoded = base64::decode(&bot.bot).unwrap();
                let csml_bot: SerializeCsmlBot = bincode::deserialize(&base64decoded[..]).unwrap();

                last_key = Some(bot.id.clone());

                let json = serde_json::json!({
                    "id": bot.id,
                    "bot": csml_bot.to_bot(),
                    "engine_version": bot.engine_version,
                    "created_at": bot.created_at
                });

                bots.push(json);
            }
            Err(_) => (),
        };
    }

    Ok(serde_json::json!({"bots": bots, "last_key": last_key}))
}

pub fn get_bot_by_version_id(id: &str, db: &mongodb::Database) -> Result<Option<CsmlBot>, EngineError> {
    let collection = db.collection("bot");

    let filter = doc! {
        "_id": bson::oid::ObjectId::with_string(id).unwrap()
    };

    let find_options = mongodb::options::FindOneOptions::builder()
        .sort(doc! { "$natural": -1, })
        .build();

    let result = collection.find_one(filter, find_options)?;

    match result {
        Some(bot) => {
            let bot = format_bot_struct(bot)?;

            let base64decoded = base64::decode(&bot.bot).unwrap();
            let csml_bot: SerializeCsmlBot = bincode::deserialize(&base64decoded[..]).unwrap();

            Ok(Some(csml_bot.to_bot()))
        }
        None => Ok(None),
    }
}

pub fn get_last_bot_version(
    bot_id: &str,
    db: &mongodb::Database,
) -> Result<Option<CsmlBot>, EngineError> {
    let collection = db.collection("bot");

    let filter = doc! {
        "bot_id": bot_id,
    };

    let find_options = mongodb::options::FindOneOptions::builder()
        .sort(doc! { "$natural": -1,  })
        .build();

    let result = collection.find_one(filter, find_options)?;

    match result {
        Some(bot) => {
            let bot = format_bot_struct(bot)?;

            let base64decoded = base64::decode(&bot.bot).unwrap();
            let csml_bot: SerializeCsmlBot = bincode::deserialize(&base64decoded[..]).unwrap();

            Ok(Some(csml_bot.to_bot()))
        }
        None => Ok(None),
    }
}
