use serde_json::{json, Value};
use crate::http_client::HttpClient;
use crate::http_client::HttpMethod::{Delete, Get, Post};

mod http_client;

fn main() {
    let url: &str = "https://jsonplaceholder.typicode.com/posts/2";

    let post_url: &str = "https://jsonplaceholder.typicode.com/comments";
    let json_data: Value = json!({
        "postId": 1,
        "id": 101,
        "name": "John Doe",
        "email": "john.doe@example.com",
        "body": "This is a test comment"
    });

    match HttpClient::request(Post, post_url, Some(&json_data)) {
        Ok(Some(http_response)) => {
            if http_response.status_code == 201 {
                println!("Post successful (Status: 201 Created)");
                println!("Response JSON body:\n{}", http_response.json_body);
            } else {
                println!("Unexpected response: {} {}", http_response.status_code, http_response.status_text);
            }
        }
        Ok(None) => {
            println!("Request was successful, but no response received");
        }
        Err(err) => {
            eprintln!("Request failed: {:?}", err);
            std::process::exit(1);
        }
    }


    match HttpClient::request(Delete, url, None) {
        Ok(Some(http_response)) => {
            if http_response.status_code == 200 {
                println!("Delete successful (Status: 200 OK)");
            } else if http_response.status_code == 204 {
                println!("Delete successful (Status: 204 No Content)");
            } else {
                println!("Unexpected response: {} {}", http_response.status_code, http_response.status_text);
            }
        }
        Ok(None) => {
            eprintln!("Connection timeout or invalid URL");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Request failed: {:?}", err);
            std::process::exit(1);
        }
    }

    match HttpClient::request(Get, url, None) {
        Ok(Some(http_response)) => {
            println!("Response status code: {}", http_response.status_code);
            println!("Response status text: {}", http_response.status_text);
            println!("Response JSON body:\n{}", http_response.json_body);
            println!("Duration: {:?}", http_response.duration);
            println!("Headers:");
            for (name, value) in http_response.headers {
                println!("{}: {}", name, value);
            }
        }
        Ok(None) => {
            eprintln!("Invalid URL");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Request failed: {:?}", err);
            std::process::exit(1);
        }
    }
}