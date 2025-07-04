pub trait DriverGeneratable {
    fn get_token_entries(&self, token_ids: &Vec<String>) -> String;
    fn get_state_token_mapping(&self, token_ids: &Vec<String>) -> String;
    fn get_template_content(&self) -> &'static str;
}

static PYTHON_DRIVER_TEMPLATE: &'static str = include_str!("../driver_templates/python.py");
pub struct PythonDriverGenerator {}
impl DriverGeneratable for PythonDriverGenerator {
    fn get_token_entries(&self, token_ids: &Vec<String>) -> String {
        let mut entries = String::new();
        for token in token_ids {
            entries.push_str(format!("    {} = auto()\n", token.to_uppercase()).as_str());
        }
        entries
    }

    fn get_state_token_mapping(&self, token_ids: &Vec<String>) -> String {
        let mut mapping = "{\n".to_string();
        for token in token_ids {
            mapping
                .push_str(format!("    \"{}\": Token.{},\n", token, token.to_uppercase()).as_str());
        }
        mapping.push_str("\n}".to_string().as_str());
        mapping
    }

    fn get_template_content(&self) -> &'static str {
        PYTHON_DRIVER_TEMPLATE
    }
}

static JAVASCRIPT_DRIVER_TEMPLATE: &'static str = include_str!("../driver_templates/javascript.js");
pub struct JavascriptDriverGenerator {}
impl DriverGeneratable for JavascriptDriverGenerator {
    fn get_token_entries(&self, token_ids: &Vec<String>) -> String {
        let mut entries = String::new();
        for token in token_ids {
            entries.push_str(format!("    {0}: Symbol('{0}'),\n", token.to_uppercase()).as_str());
        }
        entries
    }

    fn get_state_token_mapping(&self, token_ids: &Vec<String>) -> String {
        let mut mapping = "{\n".to_string();
        for token in token_ids {
            mapping.push_str(
                format!("    \"{}\": Symbol('{}'),\n", token, token.to_uppercase()).as_str(),
            );
        }
        mapping.push_str("\n}".to_string().as_str());
        mapping
    }

    fn get_template_content(&self) -> &'static str {
        JAVASCRIPT_DRIVER_TEMPLATE
    }
}

pub fn generate_driver_content(
    generatable: &dyn DriverGeneratable,
    user_defined_token_ids: &Vec<String>,
) -> String {
    let template = generatable.get_template_content();
    template
        .replace(
            "'__TOKEN_ENTRIES__'",
            &generatable.get_token_entries(&user_defined_token_ids),
        )
        .replace(
            "'__STATE_TOKEN_MAPPING__'",
            &generatable.get_state_token_mapping(&user_defined_token_ids),
        )
}
