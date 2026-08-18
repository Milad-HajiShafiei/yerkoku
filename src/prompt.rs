use crate::blueprint::{Blueprint, FieldType};
use crate::form::Form;
use chrono::Utc;

pub fn generate_prompt(blueprint: &Blueprint, form: &Form) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("# {} Development Prompt\n\n", blueprint.name));
    prompt.push_str(&format!(
        "Generated: {}\n\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
    prompt.push_str("---\n\n");

    if !blueprint.description.is_empty() {
        prompt.push_str(&format!("{}\n\n", blueprint.description));
    }

    prompt.push_str("You are a senior developer. Create a production-ready application with the following specifications:\n\n");

    let additional_crates = form
        .values
        .get("tech.additional_crates")
        .cloned()
        .unwrap_or_default();
    let crates_list = additional_crates.as_vec();
    if !crates_list.is_empty() {
        prompt.push_str("- **Additional Crates:**\n");
        for crate_info in &crates_list {
            prompt.push_str(&format!("  - {}\n", crate_info));
        }
    }

    for section in &blueprint.sections {
        prompt.push_str(&format!("## {} {}\n\n", section.icon, section.title));

        if !section.description.is_empty() {
            prompt.push_str(&format!("{}\n\n", section.description));
        }

        for field in &section.fields {
            if matches!(field.field_type, FieldType::SectionBreak) {
                continue;
            }

            let value = form.values.get(&field.key).cloned().unwrap_or_default();

            if value.is_empty() {
                continue;
            }

            match &field.field_type {
                FieldType::Checkbox => {
                    if value.as_bool() {
                        prompt.push_str(&format!(
                            "- {} {}\n",
                            field.label,
                            if !field.description.is_empty() {
                                format!("({})", field.description)
                            } else {
                                String::new()
                            }
                        ));
                    }
                }
                FieldType::Select => {
                    prompt.push_str(&format!("- **{}:** {}\n", field.label, value.as_str()));
                }
                FieldType::Multiselect => {
                    let values = value.as_vec();
                    if !values.is_empty() {
                        prompt.push_str(&format!("- **{}:** {}\n", field.label, values.join(", ")));
                    }
                }
                FieldType::Textarea => {
                    let text = value.as_str();
                    if !text.is_empty() {
                        prompt.push_str(&format!("### {}\n\n", field.label));
                        prompt.push_str("```\n");
                        prompt.push_str(text);
                        prompt.push_str("\n```\n\n");
                    }
                }

                FieldType::ListBuilder => {
                    let items = value.as_vec();
                    if !items.is_empty() {
                        prompt.push_str(&format!("### {}\n\n", field.label));
                        for item in &items {
                            prompt.push_str(&format!("- {}\n", item));
                        }
                        prompt.push('\n');
                    }
                }

                _ => {
                    let text = value.as_str();
                    if !text.is_empty() {
                        prompt.push_str(&format!("- **{}:** {}\n", field.label, text));
                    }
                }
            }
        }

        prompt.push('\n');
    }

    prompt.push_str("---\n\n");
    prompt.push_str("## Output Requirements\n\n");
    prompt.push_str("1. Show the complete project structure\n");
    prompt.push_str("2. Provide all source files in fenced code blocks\n");
    prompt.push_str("3. Include configuration files\n");
    prompt.push_str("4. Add a comprehensive README.md\n");
    prompt.push_str("5. Include setup and usage instructions\n");
    prompt.push_str("6. If response is too long, split into parts\n");
    prompt
        .push_str("7. Search the web and use the most up-to-date dependencies & technologies\n\n");
    prompt.push_str("Generate the complete application now.\n");

    prompt
}
