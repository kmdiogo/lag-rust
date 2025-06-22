use crate::regex_ast::{NodeRef, DFA};
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Write;

pub fn serialize_dfa(file: &mut File, dfa: &DFA, entry_state: &BTreeSet<NodeRef>) {
    let mut state_ids: HashMap<BTreeSet<NodeRef>, usize> = HashMap::new();
    for (i, state) in dfa.state_table.keys().enumerate() {
        state_ids.insert(state.clone(), i + 1);
    }
    if !state_ids.contains_key(entry_state) {
        panic!(
            "Provided entry state {:?} not found in DFA state table",
            entry_state
        )
    }

    // Manually construct the JSON object
    let mut states_obj = Map::new();

    for (outer_key, inner_map) in &dfa.state_table {
        let mut inner_json = Map::new();
        for (inner_key, value) in inner_map {
            inner_json.insert(
                (*inner_key).to_string(),
                Value::from(state_ids.get(value).unwrap().to_string()),
            );
        }
        states_obj.insert(
            state_ids.get(outer_key).unwrap().to_string(),
            Value::Object(inner_json),
        );
    }

    let mut json_obj = Map::new();
    let accepting_states: Vec<usize> = dfa
        .accepting_states
        .iter()
        .map(|state| *state_ids.get(state).unwrap())
        .collect();
    json_obj.insert("accepting".to_string(), Value::from(accepting_states));
    json_obj.insert("states".to_string(), Value::Object(states_obj));
    json_obj.insert(
        "entry".to_string(),
        Value::from(state_ids.get(entry_state).unwrap().to_string()),
    );
    let json_string = serde_json::to_string_pretty(&Value::Object(json_obj)).unwrap();
    match file.write_all(json_string.as_bytes()) {
        Ok(_) => {}
        Err(why) => panic!("Error writing serialized DFA to JSON file: {}", why),
    };
}
