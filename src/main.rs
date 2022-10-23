extern crate sillypaste_cli_api;
use sillypaste_cli_api::sillyrest::*;
use std::fs::{File, remove_file};
use std::path::Path;
//use serde::{Deserialize};
use clap::{Subcommand, Parser, Args};
use std::env;
use std::io::{Read, Write};

mod printer;

struct SillyClient {
    token: String,
    sillyconn: SillyPasteClient
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Post {filename: Option<String>},
    GetPost {id: u32},
    PostList{count: Option<u32>, offset: Option<u32>},
    Login(Login),
    Languages
}

#[derive(Args)]
struct Login {
    pub username: String,
    pub password: String
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let uri = match env::var("SILLYPASTE_ENDPOINT") { // api root url
        Ok(u) => u,
        _ => {
            println!("No variable set for sillypaste url.\nSet environment var SILLYPASTE_ENDPOINT");
            return;
        }
    };

    match &cli.command {
        Commands::Login(info) => {
            let conn = SillyPasteClient::new(info.username.clone(), info.password.clone(), uri).await.unwrap();
            let tok = conn.token();
            let mut tokfile = match File::create(".sillypaste_session_token") {
                Ok(f) => f,
                Err(e) => {
                    println!("Unable to open token file. {}", e);
                    return;
                }
            };
            match tokfile.write_all(tok.as_bytes()){
                Ok(_) => return,
                Err(e) => {
                    println!("Unable to write to token file. {}", e);
                    return;
                }
            }
            return;
        },
        _ => {}
    };
    let mut token = String::new();
    match File::open(".sillypaste_session_token") {
        Ok(mut f) => {
            match f.read_to_string(&mut token) {
                Ok(_) => {},
                Err(e) => {
                    panic!("Unable to read token. {}", e);
                }
            };
        }
        Err(_) => panic!("User must login.")
    };

    let conn = match SillyPasteClient::with_token(token, uri).await {
        Ok(c) => c,
        _ => {
            println!("Connection failed. Invalid login token. Relogin.");
            remove_file(".sillypaste_session_token");
            return;
        }
    };

    match &cli.command {
        Commands::Post {filename} => {
            match filename {
                Some(c) => {
                    let mut file = match File::open(c.clone()) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("Could not open file: {}", e);
                            return;
                        }
                    };
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents) {
                        Ok(_) => {},
                        Err(s) => panic!("could not read file: {}", s)
                    };
                    match conn.upload_paste(contents, c.clone(), None).await {
                        Ok(id) => {
                            println!("uploaded with id: {}", id);
                            return;
                        },
                        Err(_) => {
                            println!("Failed to upload post");
                            return;
                        }
                    };
                    return;
                },
                _ => println!("todo: open editor")
            }
        },
        Commands::GetPost {id} => {
            let post = match conn.retrieve_post(*id).await {
                Ok(p) => p,
                Err(_) => panic!("failed to retrieve post")
            };
            println!("{}\n", post.title());
            let code = printer::highlight(String::from("Rust"), post.body()).unwrap();
            println!("{}", code);
            return;
        },
        Commands::PostList {count, offset} => {
            let limit = match count {
                Some(c) => *c,
                _ => 10 as u32
                };
            let off = match offset {
                Some(o) => *o,
                _ => 0 as u32
            };
            let posts = match conn.fetch_posts(limit, off).await{
                Ok(p) => p,
                Err(e) => panic!("Failed to retrieve posts. {:#?}", e)
            };
            for post in posts {
                println!("{}\nID: {:#?}  Author: #{:#?}  Language: #{:#?}\n", post.title(), post.id(), post.author(),
                    post.language());
            }
            return;
        },
        Commands::Languages =>return,
        _ => return
    };
}
