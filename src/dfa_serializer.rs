use crate::parser::{ClassSetEntry, ClassSetOperator};
use crate::regex_ast::{NodeRef, DFA};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug)]
struct SerializedClassSet {
    chars: HashSet<char>,
    exclude: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SerializedDFA {
    accepting: HashMap<String, Vec<String>>,
    class_sets: HashMap<String, SerializedClassSet>,
    entry: String,
    states: HashMap<String, HashMap<String, String>>,
}

/// Returns all accepting tokens, if any, for a state set
fn get_accepting_tokens(
    end_nodes: &HashMap<NodeRef, String>,
    state: &BTreeSet<NodeRef>,
    token_order: &Vec<String>,
) -> Vec<String> {
    debug!("Getting accepting tokens for state: {:?}", state);
    let mut accepting_tokens = HashSet::new();
    for node_ref in state {
        match end_nodes.get(&node_ref) {
            Some(token_id) => accepting_tokens.insert(token_id.clone()),
            None => {
                continue;
            }
        };
    }
    let mut sorted_tokens: Vec<String> = Vec::new();
    for token in token_order {
        if !accepting_tokens.contains(token) {
            continue;
        }
        sorted_tokens.push(token.clone());
    }
    debug!("  Result: {:?}", sorted_tokens);
    sorted_tokens
}

/// Serializes a DFA and returns it as a JSON string
pub fn serialize_dfa(
    dfa: &DFA,
    entry_state: &BTreeSet<NodeRef>,
    end_nodes: &HashMap<NodeRef, String>,
    token_order: &Vec<String>,
    class_lookup_table: &BTreeMap<String, ClassSetEntry>,
) -> String {
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
    let mut serialized_states: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (state_set, transition_table) in &dfa.state_table {
        let mut serialized_transition_table: HashMap<String, String> = HashMap::new();
        for (input_symbol, transition_state_set) in transition_table {
            serialized_transition_table.insert(
                (*input_symbol).clone(),
                state_ids.get(transition_state_set).unwrap().to_string(),
            );
        }
        serialized_states.insert(
            state_ids.get(state_set).unwrap().to_string(),
            serialized_transition_table,
        );
    }

    let mut accepting_states: HashMap<String, Vec<String>> = HashMap::new();
    for state in dfa.state_table.keys() {
        let accepting_tokens = get_accepting_tokens(end_nodes, state, &token_order);
        if accepting_tokens.len() > 0 {
            accepting_states.insert(
                String::from(state_ids.get(state).unwrap().to_string()),
                accepting_tokens,
            );
        }
    }

    let class_sets: HashMap<_, _> = class_lookup_table
        .iter()
        .map(|(class_id, class_set)| {
            (
                format!("[{}]", class_id),
                SerializedClassSet {
                    chars: class_set.chars.clone(),
                    exclude: match class_set.operator {
                        ClassSetOperator::Negate => true,
                        _ => false,
                    },
                },
            )
        })
        .collect();
    let serialized_dfa = SerializedDFA {
        entry: state_ids.get(entry_state).unwrap().to_string(),
        accepting: accepting_states,
        states: serialized_states,
        class_sets: class_sets,
    };
    let json_string = serde_json::to_string_pretty(&serialized_dfa).unwrap();
    json_string
}
