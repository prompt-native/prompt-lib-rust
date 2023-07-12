use serde::Deserialize;
use serde_yaml::Value;

#[derive(Debug, Deserialize)]
struct PromptType {
    #[serde(rename = "type")]
    prompt_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CompletionExampleColumn {
    pub name: String,
    pub values: Vec<String>,
    pub test: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatExample {
    pub input: String,
    pub output: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub input: String,
    pub output: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Deserialize)]
pub struct Completion {
    #[serde(rename = "type")]
    pub prompt_type: String,
    pub vendor: String,
    pub model: String,
    pub prompt: String,
    pub parameters: Option<Vec<Parameter>>,
    pub examples: Option<Vec<CompletionExampleColumn>>,
}

pub fn find_parameter(
    parameters: &Option<Vec<crate::prompt::Parameter>>,
    name: &str,
) -> Option<Value> {
    if let Some(real_parameters) = parameters {
        real_parameters
            .iter()
            .find(|p| p.name == name)
            .map(|x| x.value.clone())
    } else {
        None
    }
}

impl Completion {
    pub fn example_count(&self) -> usize {
        let mut max_length = 0;
        if let Some(columns) = &self.examples {
            for column in columns {
                if column.values.len() > max_length {
                    max_length = column.values.len()
                }
            }
        }
        max_length
    }

    pub fn final_prompt(&self) -> String {
        let mut prompt = self.prompt.clone();
        prompt.push_str("\n\n");
        if let Some(columns) = &self.examples {
            for i in 0..self.example_count() {
                for column in columns {
                    let line: String = format!(
                        "{}: {}\n",
                        column.name,
                        column.values.get(i).unwrap_or(&"".to_string())
                    );
                    prompt.push_str(&line);
                }
                prompt.push('\n');
            }
            for column in columns {
                let line: String = format!(
                    "{}: {}\n",
                    column.name,
                    column.test.as_ref().unwrap_or(&"".to_string())
                );
                prompt.push_str(&line);
            }
        }
        prompt.to_string()
    }

    pub fn find_parameter_as_i32(&self, name: &str) -> Option<i32> {
        find_parameter(&self.parameters, name).map(|p| p.as_i64().unwrap() as i32)
    }

    pub fn find_parameter_as_f32(&self, name: &str) -> Option<f32> {
        find_parameter(&self.parameters, name).map(|p| p.as_f64().unwrap() as f32)
    }

    pub fn find_parameter_as_str(&self, name: &str) -> Option<String> {
        find_parameter(&self.parameters, name).map(|p| p.as_str().unwrap().to_string())
    }

    pub fn find_parameter_as_bool(&self, name: &str) -> Option<bool> {
        find_parameter(&self.parameters, name).map(|p| p.as_bool().unwrap())
    }
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    #[serde(rename = "type")]
    pub prompt_type: String,
    pub vendor: String,
    pub model: String,
    pub parameters: Option<Vec<Parameter>>,
    pub examples: Option<Vec<ChatExample>>,
    pub context: Option<String>,
    pub messages: Option<Vec<Message>>,
}

impl Chat {
    pub fn find_parameter_as_i32(&self, name: &str) -> Option<i32> {
        find_parameter(&self.parameters, name).map(|p| p.as_i64().unwrap() as i32)
    }

    pub fn find_parameter_as_f32(&self, name: &str) -> Option<f32> {
        find_parameter(&self.parameters, name).map(|p| p.as_f64().unwrap() as f32)
    }

    pub fn find_parameter_as_str(&self, name: &str) -> Option<String> {
        find_parameter(&self.parameters, name).map(|p| p.as_str().unwrap().to_string())
    }

    pub fn find_parameter_as_bool(&self, name: &str) -> Option<bool> {
        find_parameter(&self.parameters, name).map(|p| p.as_bool().unwrap())
    }
}

#[derive(Debug, Deserialize)]
pub enum Prompt {
    Completion(Completion),
    Chat(Chat),
    Unknown,
}

pub fn deserialize_prompt(yaml: &str) -> Prompt {
    let prompt_type: PromptType = serde_yaml::from_str(&yaml).unwrap();
    match prompt_type.prompt_type.as_str() {
        "completion" => {
            let completion: Completion = serde_yaml::from_str(&yaml).unwrap();
            Prompt::Completion(completion)
        }
        "chat" => {
            let chat: Chat = serde_yaml::from_str(&yaml).unwrap();
            Prompt::Chat(chat)
        }
        _ => Prompt::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_prompt_completion() {
        let yaml = r#"  
            type: completion  
            vendor: google
            model: text-bison
            prompt: Write a hello world in java
            parameters:  
                - name: maxOutputTokens
                  value: 256
                - name: temperature
                  value: 0.4
            examples:  
                - name: input
                  values:
                    - a
                    - b
                  test: c
                - name: output
                  values:
                    - x
                    - y
        "#;

        let prompt = deserialize_prompt(yaml);

        if let Prompt::Completion(completion) = prompt {
            assert_eq!(completion.vendor, "google");
            assert_eq!(completion.model, "text-bison");
            assert_eq!(completion.prompt, "Write a hello world in java");

            if let Some(parameters) = completion.parameters {
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].name, "maxOutputTokens");
                assert_eq!(parameters[0].value, 256);
                assert_eq!(parameters[1].name, "temperature");
                assert_eq!(parameters[1].value, 0.4);
            }

            if let Some(examples) = completion.examples {
                assert_eq!(examples.len(), 2);
                assert_eq!(examples[0].name, "input");
                assert_eq!(examples[1].name, "output");
                assert_eq!(examples[0].values, vec!["a", "b"]);
                assert_eq!(examples[0].test, Some("c".to_string()));
                assert_eq!(examples[1].values, vec!["x", "y"]);
                assert_eq!(examples[1].test, None);
            }
        } else {
            panic!("Expected Prompt::Completion, got {:?}", prompt);
        }
    }

    #[test]
    fn test_deserialize_prompt_chat() {
        let yaml = r#"  
            type: chat  
            vendor: google
            model: chat-bison 
            parameters:  
                - name: maxOutputTokens
                  value: 256
                - name: temperature
                  value: 0.4
            examples:  
                - input: who are u?
                  output: I'm google
            messages:  
                - input: what's your name?
        "#;

        let prompt = deserialize_prompt(yaml);

        if let Prompt::Chat(chat) = prompt {
            assert_eq!(chat.vendor, "google");
            assert_eq!(chat.model, "chat-bison");

            if let Some(parameters) = chat.parameters {
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].name, "maxOutputTokens");
                assert_eq!(parameters[0].value, 256);
                assert_eq!(parameters[1].name, "temperature");
                assert_eq!(parameters[1].value, 0.4);
            }

            if let Some(examples) = chat.examples {
                assert_eq!(examples.len(), 1);
                assert_eq!(examples[0].input, "who are u?");
                assert_eq!(examples[0].output, Some("I'm google".to_string()));
            }

            assert_eq!(chat.context, None);

            if let Some(messages) = chat.messages {
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0].input, "what's your name?");
                assert_eq!(messages[0].output, None);
            }
        } else {
            panic!("Expected Prompt::Chat, got {:?}", prompt);
        }
    }

    #[test]
    fn test_deserialize_prompt_unknown() {
        let yaml = r#"  
            type: unknown  
        "#;

        let prompt = deserialize_prompt(yaml);

        if let Prompt::Unknown = prompt {
            // Test passed
        } else {
            panic!("Expected Prompt::Unkwon, got {:?}", prompt);
        }
    }

    #[test]
    fn test_get_examples_count() {
        let yaml = r#"  
            type: completion  
            vendor: google
            model: text-bison
            prompt: Write a hello world in java
            parameters:  
                - name: maxOutputTokens
                  value: 256
                - name: temperature
                  value: 0.4
            examples:  
                - name: input
                  values:
                    - a
                    - b
                  test: c
                - name: output
                  values:
                    - x
                    - y
        "#;

        let prompt = deserialize_prompt(yaml);

        let final_prompt = r#"Write a hello world in java

input: a
output: x

input: b
output: y

input: c
output: 
"#;
        if let Prompt::Completion(completion) = prompt {
            assert_eq!(completion.example_count(), 2);
            assert_eq!(completion.final_prompt(), final_prompt);
        } else {
            panic!("Expected Prompt::Unkwon, got {:?}", prompt);
        }
    }
}
