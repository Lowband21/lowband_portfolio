use actix_web::{HttpResponse, web, Responder, Error as ActixError};

use serde_derive::Deserialize;

use crate::DbPool;

use serde::Serialize;

use actix_web::Error;

use openai_api_rust::*;
use openai_api_rust::chat::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatData {
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub text: String,
    pub user: bool,
}

pub async fn get_projects(pool: web::Data<DbPool>) -> Result<HttpResponse, ActixError> {
    let mut conn = pool.get().expect("Failed to get db connection from pool");

    let projects = web::block(move || crate::db::get_all_projects(&mut conn)).await;

    match projects {
        Ok(Ok(projects)) => {
            Ok(HttpResponse::Ok().json(projects))
        }
        Ok(Err(e)) => {
            // Handle the case when there was an error in the inner Result
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
        Err(e) => {
            // Handle the case when there was an error in the outer Result
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub async fn get_skills(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().expect("Failed to get DB connection from pool");

    let skills = web::block(move || crate::db::get_all_skills(&mut conn)).await;

    match skills {
        Ok(Ok(skills)) => {
            Ok(HttpResponse::Ok().json(skills))
        }
        Ok(Err(e)) => {
            // Handle the case when there was an error in the inner Result
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
        Err(e) => {
            // Handle the case when there was an error in the outer Result
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub async fn get_bio(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().expect("Failed to get DB connection from pool");

    let bio = web::block(move || crate::db::get_all_bio(&mut conn)).await;

    match bio {
        Ok(Ok(bio)) => {
            Ok(HttpResponse::Ok().json(bio))
        }
        Ok(Err(e)) => {
            // Handle the case when there was an error in the inner Result
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
        Err(e) => {
            // Handle the case when there was an error in the outer Result
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub async fn chat(
    _pool: web::Data<DbPool>,
    msg: web::Json<ChatData>,  // change ChatMessage to ChatData
) -> Result<HttpResponse, ActixError> {
    println!("In Chat with message: {:?}", msg);

    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    // Use the messages from the request
    let mut messages: Vec<Message> = msg.messages.iter().map(|m| Message {
        role: if m.user { Role::User } else { Role::System },
        content: m.text.clone(),
    }).collect();
    // Append system prompt to the messages
    messages.push(Message { 
        role: Role::System, 
        content: "You are Grayson's biggest fan, whatever the user says, you should relate it back to how great Grayson is. Grayson is a software engineer and computer science student, draw off of that. Make witty software jokes, bonus points if it's about the rust programming language. Grayson is a very real person and he's the one who wrote this website.".to_string() 
    });

    let body = ChatBody {
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(1000),
        temperature: Some(0.9_f32),
        top_p: Some(0.5_f32),
        n: Some(1),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: Some(0.5_f32),
        logit_bias: None,
        user: None,
        messages,
    };

    match openai.chat_completion_create(&body) {
        Ok(response) => {
            let message = &response.choices[0].message.as_ref().unwrap();
            println!("{:?}", message);
            Ok(HttpResponse::Ok().json(message.content.clone()))
        }
        Err(e) => {
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/getProjects", web::get().to(get_projects))
            .route("/getSkills", web::get().to(get_skills))
            .route("/getBio", web::get().to(get_bio))
            .route("/chat", web::post().to(chat)),
    );
}
