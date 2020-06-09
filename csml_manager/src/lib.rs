pub mod data;
pub use csmlinterpreter::data::Client;

mod db_interactions;
mod encrypt;
mod init;
mod interpreter_actions;
mod send;
mod tools;

use data::ManagerError;
use data::*;
use db_interactions::{conversation::*, messages::*, state::*};
use init::*;
use interpreter_actions::interpret_step;
use tools::*;

use csmlinterpreter::data::{
    csml_bot::CsmlBot, csml_flow::CsmlFlow, csml_result::CsmlResult, error_info::ErrorInfo,
    ContextJson, Hold, Memories, Message,
};
use md5::{Digest, Md5};
use serde_json::{map::Map, Value};
use std::{env, time::SystemTime};

pub fn start_conversation(
    json_event: Value,
    csmldata: CsmlData,
) -> Result<Map<String, Value>, ManagerError> {
    let now = SystemTime::now();

    let event = format_event(json_event.clone())?;
    let mut data = init_conversation_info(
        get_default_flow(&csmldata.bot)?.name.to_owned(),
        &event,
        &csmldata,
    )?;

    // save event in db as message RECEIVE
    let event_receive = format_event_message(&mut data, json_event)?;
    add_messages_bulk(&mut data, vec![event_receive])?;

    let flow = get_flow_by_id(&data.context.flow, &csmldata.bot.flows)?;
    check_for_hold(&mut data, flow)?;

    let res = interpret_step(&mut data, event.to_owned(), &csmldata);

    if let Ok(var) = env::var(DEBUG) {
        if var == "true" {
            let el = now.elapsed()?;
            println!("Total time Manager - {}.{}", el.as_secs(), el.as_millis());
        }
    }
    res
}

pub fn get_open_conversation(client: &Client) -> Result<Option<Conversation>, ManagerError> {
    let db = init_db()?;

    get_latest_open(client, &db)
}

pub fn get_steps_from_flow(bot: CsmlBot, flow_name: String) -> Vec<String> {
    match csmlinterpreter::get_steps_from_flow(bot, flow_name) {
        Some(vec) => vec,
        None => vec![],
    }
}

pub fn validate_bot(bot: CsmlBot) -> Result<bool, Vec<ErrorInfo>> {
    match csmlinterpreter::validate_bot(bot) {
        CsmlResult {
            flows: _,
            warnings: _,
            errors: None,
        } => Ok(true),
        CsmlResult {
            flows: _,
            warnings: _,
            errors: Some(e),
        } => Err(e),
    }
}

pub fn user_close_all_conversations(client: Client) -> Result<(), ManagerError> {
    let db = init_db()?;

    close_all_conversations(&client, &db)
}

// reset memory if flow hash is different or see if there are some save tmp memories
fn check_for_hold(data: &mut ConversationInfo, flow: &CsmlFlow) -> Result<(), ManagerError> {
    match get_state_key(&data.client, "hold", "position", &data.db) {
        Ok(Some(string)) => {
            let hold = serde_json::to_value(string)?;
            let mut hash = Md5::new();

            hash.input(flow.content.as_bytes());
            let new_hash = format!("{:x}", hash.result());

            if new_hash != hold["hash"] {
                data.context.step = "start".to_owned();
                delete_state_key(&data.client, "hold", "position", &data.db)?;
                data.context.hold = None;
                return Ok(());
            }
            data.context.hold = Some(Hold {
                index: hold["index"].as_u64().ok_or(ManagerError::Interpreter(
                    "hold index bad format".to_owned(),
                ))? as usize,
                step_vars: hold["step_vars"].clone(),
            });
            delete_state_key(&data.client, "hold", "position", &data.db)?;
        }
        Ok(None) => (),
        Err(_) => (),
    };
    Ok(())
}
