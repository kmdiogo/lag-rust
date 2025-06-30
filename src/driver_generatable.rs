pub trait DriverGeneratable {
    fn get_token_entries(token_ids: &Vec<String>) -> String;
    fn get_state_token_mapping(token_ids: &Vec<String>) -> String;
}

pub struct PythonDriverGenerator {}
impl DriverGeneratable for PythonDriverGenerator {
    fn get_token_entries(token_ids: &Vec<String>) -> String {
        let mut entries = String::new();
        for token in token_ids {
            entries.push_str(format!("    {} = auto()\n", token.to_uppercase()).as_str());
        }
        entries
    }

    fn get_state_token_mapping(token_ids: &Vec<String>) -> String {
        let mut mapping = "{\n".to_string();
        for token in token_ids {
            mapping
                .push_str(format!("    \"{}\": Token.{},\n", token, token.to_uppercase()).as_str());
        }
        mapping.push_str("\n}".to_string().as_str());
        mapping
    }
}
