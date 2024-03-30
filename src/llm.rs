use std::env;
use std::fs;
use std::path::PathBuf;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use futures_util::stream::StreamExt;

use serde_json::{Value, from_slice};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::runtime::Runtime;


pub trait LLM {
    fn ask(&mut self, question: &str) -> String;
}

pub struct CopilotChat {
    jwt_token: Option<String>, //TODO: probably don't need this anymore
    request_header: HeaderMap,
    state: Value,
}

impl LLM for CopilotChat {
    fn ask(&mut self, question: &str) -> String {
        println!("Asking: {}", question);
        
        if let Some(messages) = self.state["messages"].as_array_mut() {
            messages.push(serde_json::json!({
                "role": "user",
                "content": question,
            }));
        }
        let rt = Runtime::new().unwrap();
        let jwt = rt.block_on(self.stream_copilot_request(question)).unwrap();
        return "fuck you!".to_string();
    }
}

impl CopilotChat {
    fn get_oauth_token() -> String {
        match std::env::var("COPILOT_TOKEN") {
            Ok(token) => return token,
            Err(_) => {}
        }

        let mut path = PathBuf::from(env::var("HOME").unwrap());
        path.push(".config/github-copilot/hosts.json");

        let data = fs::read_to_string(path).expect("Unable to read file");
        let json: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
        let oauth_token = json["github.com"]["oauth_token"]
            .as_str()
            .expect("oauth_token not found");

        println!("oauth_token: {}", oauth_token);
        return oauth_token.to_string();
    }

    fn update_jwt_token(&mut self) {
        let rt = Runtime::new().unwrap();
        let jwt = rt.block_on(Self::get_jwt_token()).unwrap();
        
        // Update the request header with the new jwt token
        let bearer_token: String = format!(
            "Bearer {jwt_token}",
            jwt_token = jwt.to_string()
        );

        println!("bearer_token: {}", bearer_token);
        self.request_header.insert(
            "authorization",
            HeaderValue::from_str(&bearer_token).unwrap(),
        );
    }

    async fn stream_copilot_request(&self, question: &str) -> Result<(), Error> {
        let client = Client::new();

        println!("state: {:?}", self.state);
        let headers = self.request_header.clone();
        println!("headers: {:?}", headers);
        let mut response = client
            .post("https://api.githubcopilot.com/chat/completions")
            .headers(headers)
            .json(&self.state) // Assuming `state` is already defined and serializable
            .send()
            .await?;

        println!("response: {:?}", response);

        println!("reponse:");

        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item?;
            println!("{:?}", chunk);
        }

        return Ok(());
    }

    async fn get_jwt_token() -> Option<String> {
        let client = reqwest::Client::new();

        let token_header = format!(
            "token {copilot_token}",
            copilot_token = Self::get_oauth_token()
        );

        let token_header_encoded = HeaderValue::from_str(&token_header).unwrap();

        let header_pairs = vec![
            ("authorization", token_header_encoded),
            (
                "editor-version",
                HeaderValue::from_static("vscode/1.79.0-insider"),
            ),
            (
                "editor-plugin-version",
                HeaderValue::from_static("copilot/1.86.112"),
            ),
            (
                "user-agent",
                HeaderValue::from_static("GithubCopilot/1.86.112"),
            ),
            ("accept", HeaderValue::from_static("*/*")),
        ];

        let mut headers = HeaderMap::new();

        for (key, value) in header_pairs {
            headers.insert(key, value);
        }

        let res = client
            .get("https://api.github.com/copilot_internal/v2/token")
            .headers(headers)
            .send()
            .await
            .unwrap();

        let json: Value = res.text().await.unwrap().parse().unwrap();

        if (json["token"].is_null()) {
            return None;
        }

        let token = json["token"].as_str().unwrap().to_string();

        return Some(token);
    }
    pub fn new() -> CopilotChat {
        let mut map = HeaderMap::new();
        map.insert(
            "x-request-id",
            HeaderValue::from_static("9d4f79c9-7104-4e24-a3ac-73349f95af63"),
        );
        map.insert(
            "openai-organization",
            HeaderValue::from_static("github-copilot"),
        );
        map.insert(
            "vscode-sessionid",
            HeaderValue::from_static("9188b680-9c71-402e-9e9d-f6d3a99f71f91684844091941"),
        );
        map.insert(
            "vscode-machineid",
            HeaderValue::from_static(
                "859856161997d243b5f349338d1bd485b6d2664faa24bed9c1f09bdff6dddb08",
            ),
        );
        map.insert(
            "editor-version",
            HeaderValue::from_static("vscode/1.79.0-insider"),
        );
        map.insert(
            "editor-plugin-version",
            HeaderValue::from_static("copilot/0.1.2023052205"),
        );
        map.insert(
            "openai-intent",
            HeaderValue::from_static("conversation-panel"),
        );
        map.insert("content-type", HeaderValue::from_static("application/json"));
        map.insert(
            "user-agent",
            HeaderValue::from_static("GithubCopilot/0.1.2023052205"),
        );
        map.insert("accept", HeaderValue::from_static("*/*"));

        let mut ret = CopilotChat {
            jwt_token: None,
            request_header: map,
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
