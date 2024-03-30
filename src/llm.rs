use std::env;
use std::fs;
use std::path::PathBuf;

use futures_util::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use std::io::{self, Write};

use serde_json::{from_slice, Value};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::Runtime;
pub enum LLMRole {
    User,
    Assistant,
    System,
}

pub trait LLM {
    fn ask(&mut self, question: &str) -> String;
    fn append_message(&mut self, role: LLMRole, content: &String);
}

pub struct CopilotChat {
    api_request_header: HeaderMap,
    state: Value, // a json value, conains all past conversation
}

impl LLM for CopilotChat {
    fn append_message(&mut self, role: LLMRole, content: &String) {
        let role_str = match role {
            LLMRole::User => "user",
            LLMRole::Assistant => "assistant",
            LLMRole::System => "system",
        };
        match self.state["messages"].as_array_mut() {
            None => {
                self.state["messages"] = serde_json::json!([{
                    "role": role_str,
                    "content": content,
                }]);
            }
            Some(messages) => {
                messages.push(serde_json::json!({
                    "role": role_str,
                    "content": content,
                }));
            }
        }
    }

    fn ask(&mut self, question: &str) -> String {
        self.append_message(LLMRole::User, &question.to_string());
        let rt = Runtime::new().unwrap();
        let res = rt.block_on(self.stream_copilot_request(question));

        match res {
            Ok(ai_output) => {
                if ai_output.is_none() {
                    return "Empty response".to_string();
                }
                let ai_output: String = ai_output.unwrap().to_string();
                self.append_message(LLMRole::Assistant, &ai_output);
                return ai_output;
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return "Error".to_string();
            }
        }
    }
}

impl CopilotChat {
    fn get_oauth_token() -> Option<String> {
        match std::env::var("COPILOT_TOKEN") {
            Ok(token) => return Some(token),
            Err(_) => {}
        }

        let mut path = PathBuf::from(env::var("HOME").unwrap());
        path.push(".config/github-copilot/hosts.json");

        match fs::read_to_string(&path) {
            Ok(data) => {
                let json: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
                let oauth_token = json["github.com"]["oauth_token"]
                    .as_str()
                    .expect("oauth_token not found");

                println!("oauth_token: {}", oauth_token);
                return Some(oauth_token.to_string());
            }
            Err(_) => {
                println!("Error: Could not find the hosts.json file");
                return None;
            }
        }
    }

    fn update_jwt_token(&mut self) -> bool {
        let rt = Runtime::new().unwrap();
        let jwt = rt.block_on(Self::get_jwt_token()).unwrap();

        if jwt == "" {
            println!("Error: Could not get jwt token");
            return false;
        }

        // Update the request header with the new jwt token
        let bearer_token: String = format!("Bearer {jwt_token}", jwt_token = jwt.to_string());

        self.api_request_header.insert(
            "authorization",
            HeaderValue::from_str(&bearer_token).unwrap(),
        );
        return true;
    }

    /**
     * Stream a request to the copilot server, prints out the response in a text stream
     * returns the completed response as a string
     *
     * @param question: the question to ask the copilot server
     */
    async fn stream_copilot_request(&mut self, question: &str) -> Result<Option<String>, Error> {
        let client = Client::new();

        let headers = self.api_request_header.clone();

        let mut response = client
            .post("https://api.githubcopilot.com/chat/completions")
            .headers(headers)
            .json(&self.state)
            .send()
            .await?;

        match response.error_for_status_ref() {
            Ok(_) => {}
            Err(e) => {
                if e.status().is_some() {
                    if e.status().unwrap().as_u16() == 401 {
                        if self.update_jwt_token() {
                            // submit the request again
                            response = client
                                .post("https://api.githubcopilot.com/chat/completions")
                                .headers(self.api_request_header.clone())
                                .json(&self.state)
                                .send()
                                .await?;
                        }
                    }
                }
                println!("Error: {:?}", e);
                return Err(e);
            }
        }

        // read a line that was sent back. copilot responses are sent in lines json-like bytes
        // each line returned by a api is either empty or a json-object. The line is also prefixed
        // with data: so we need to remove that prefix
        fn read_line(line: &str) -> Option<String> {
            // println!("line: {:?}", line);
            if !line.contains("data") {
                return None;
            }
            if line.contains("data: [DONE]") {
                return None;
            }
            let to_parse = &line[6..];
            let json_res: Value = serde_json::from_str(to_parse).unwrap();
            if json_res.get("choices").is_none() {
                return None;
            }
            let choices = json_res.get("choices").unwrap();
            if !choices.is_array() {
                return None;
            }
            let choices = choices.as_array().unwrap();
            if choices.len() == 0 {
                return None;
            }
            if choices[0].get("delta").is_none() {
                return None;
            }
            if choices[0]["delta"].get("content").is_none() {
                return None;
            }
            let word = choices[0]["delta"]["content"].as_str();
            if word.is_none() {
                return None;
            }
            let word = word.unwrap().to_string();

            return Some(word);
        }

        println!("AI:====================");
        let mut ai_response: String = String::new();
        let mut buf: String = String::new();
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            // look for new line character, if found, print the buffer
            let chunk_str = std::str::from_utf8(&chunk).unwrap();
            buf.push_str(chunk_str);
            if chunk_str.contains("\n") {
                // look for all the new lines
                let lines = buf.split("\n").collect::<Vec<&str>>();
                let iter_range: usize;
                let last_line_is_copmlete = lines.last().unwrap().contains("\n"); // unwrap is safe, there is at least one line
                match last_line_is_copmlete {
                    true => {
                        iter_range = lines.len(); // process all lines
                    }
                    false => {
                        iter_range = lines.len() - 1; // don't process the last line yet, byte
                                                      // streams are incoming
                    }
                }
                for i in 0..iter_range {
                    let partial_ai_response = read_line(lines[i]);
                    if partial_ai_response.is_some() {
                        print!("{}", &partial_ai_response.clone().unwrap());
                        io::stdout().flush().unwrap();
                        ai_response.push_str(&partial_ai_response.unwrap());
                    }
                }
                // set buf to last line, if the last line doesn't have a newline
                let last_line = lines.last().unwrap();
                if !last_line_is_copmlete {
                    buf = last_line.to_string(); // buf is an unfinished line
                }
            }
        }
        println!();
        println!("=======================");

        return Ok(Some(ai_response));
    }

    async fn get_jwt_token() -> Option<String> {
        let client = reqwest::Client::new();

        let token_header: String;

        match Self::get_oauth_token() {
            Some(token) => {
                token_header = format!("token {copilot_token}", copilot_token = token);
            }
            None => {
                return None;
            }
        }

        let mut jwt_headers: HeaderMap = [
            ("editor-version", "vscode/1.79.0-insider"),
            ("editor-plugin-version", "copilot/1.86.112"),
            ("user-agent", "GithubCopilot/1.86.112"),
            ("accept", "*/*"),
        ]
        .iter()
        .map(|(k, v)| (k.parse().unwrap(), HeaderValue::from_static(*v)))
        .collect();

        jwt_headers.insert(
            "authorization",
            HeaderValue::from_str(&token_header).unwrap(),
        );

        let res = client
            .get("https://api.github.com/copilot_internal/v2/token")
            .headers(jwt_headers)
            .send()
            .await
            .unwrap();

        let json: Value = res.text().await.unwrap().parse().unwrap();

        if json["token"].is_null() {
            return None;
        }

        let token = json["token"].as_str().unwrap().to_string();

        return Some(token);
    }
    pub fn new() -> CopilotChat {
        let map: HeaderMap = [
            ("x-request-id", "9d4f79c9-7104-4e24-a3ac-73349f95af63"),
            ("openai-organization", "github-copilot"),
            (
                "vscode-sessionid",
                "9188b680-9c71-402e-9e9d-f6d3a99f71f91684844091941",
            ),
            (
                "vscode-machineid",
                "859856161997d243b5f349338d1bd485b6d2664faa24bed9c1f09bdff6dddb08",
            ),
            ("editor-version", "vscode/1.79.0-insider"),
            ("editor-plugin-version", "copilot/0.1.2023052205"),
            ("openai-intent", "conversation-panel"),
            ("content-type", "application/json"),
            ("user-agent", "GithubCopilot/0.1.2023052205"),
            ("accept", "*/*"),
        ]
        .iter()
        .map(|(k, v)| (k.parse().unwrap(), HeaderValue::from_static(*v)))
        .collect();

        let mut ret = CopilotChat {
            api_request_header: map,
            state: serde_json::json!({
                "intent": true,
                "messages": [],
                "model": "gpt-4",
                // # "model": "copilot-chat",
                "n": 1,
                "stream": true,
                "temperature": 0.1,
                "top_p": 1,
            }),
        };

        ret.update_jwt_token();

        return ret;
    }
}
