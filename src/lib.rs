#[macro_use]
extern crate redis_module;

use redis_module::native_types::RedisType;
use redis_module::{Context, NextArg, RedisError, RedisResult, REDIS_OK};
use std::os::raw::c_void;
use crate::machine::SMachine;
use std::collections::HashMap;
pub mod machine;




static SM_REDIS_TYPE: RedisType = RedisType::new(
    "smrd_type",
    0,
    raw::RedisModuleTypeMethods {
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        rdb_load: Some(machine::RedisMachineRdbLoad),
        rdb_save: Some(machine::RedisMachineRdbSave),
        aof_rewrite: None,
        free: Some(machine::Free),

        // Currently unused by Redis
        mem_usage: None,
        digest: None,

        // Aux data
        aux_load: None,
        aux_save: None,
        aux_save_triggers: 0,
    },
);

fn sm_new(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let name = args.next_string()?;
    let initial = args.next_string()?;
    ctx.log_debug(format!("key: {}, init: {}", name, initial).as_str());
    let key = ctx.open_key_writable(&name);

    match key.get_value::<machine::SMachine>(&SM_REDIS_TYPE)? {
        Some(value) => {
            Err("machine exists".into())
        }
        None => {
            let value = machine::SMachine::new(initial, name);
            key.set_value(&SM_REDIS_TYPE, value)?;
            REDIS_OK
        }
    }
}

fn sm_get(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let name = args.next_string()?;
    ctx.log_debug(format!("key: {}", name).as_str());
    let key = ctx.open_key_writable(&name);

    match key.get_value::<machine::SMachine>(&SM_REDIS_TYPE)? {
        Some(value) => {
            let state = value.get_state();
            ctx.log_debug(format!("state: {}", state).as_str());
            Ok(state.into())
        }
        None => {
            Err("machine doesn't exists".into())
        }
    }
}

fn sm_add(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let name = args.next_string()?;
    let event = args.next_string()?;
    let start = args.next_string()?;
    let end = args.next_string()?;
    let action = args.next_string()?;
    ctx.log_debug(format!("name: {} event: {} start: {} end: {} action: {} ", name, event, start, end, action).as_str());
    let key = ctx.open_key_writable(&name);

    match key.get_value::<machine::SMachine>(&SM_REDIS_TYPE)? {
        Some(value) => {
            value.add_transition(start, event, end, action);
            ctx.log_debug(format!("added transition").as_str());
            REDIS_OK
        }
        None => {
            Err("machine doesn't exists".into())
        }
    }
}

fn sm_event(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let name = args.next_string()?;
    let event = args.next_string()?;
    ctx.log_debug(format!("key: {} name: {}", name, event).as_str());
    let key = ctx.open_key_writable(&name);

    match key.get_value::<machine::SMachine>(&SM_REDIS_TYPE)? {
        Some(value) => {
            let action = value.on_event(event);
            ctx.log_debug(format!("action: {}", action).as_str());
            Ok(action.into())
        }
        None => {
            Err("machine doesn't exists".into())
        }
    }
}

redis_module! {
    name: "statemachine",
    version: 1,
    data_types: [
        SM_REDIS_TYPE,
    ],
    commands: [
        ["sm.new", sm_new, "write"],
        ["sm.add", sm_add, "write"],
        ["sm.event", sm_event, "write"],
        ["sm.get", sm_get, "readonly"],
    ],
}