use ask_ai::{
    // Point to your actual crate/module where ask_question and structs live
    ask_ai::ask_question,
    config::{AiConfig, Framework, Question},
};
use legatio::services::config::{UserConfig, store_config};
use legatio::utils::structs::Project;
use legatio::services::legatio::Legatio;

use httpmock::prelude::*;
use serial_test::serial;
use sqlx::{sqlite::SqlitePoolOptions};
use std::env;
use tempfile::NamedTempFile;
use tokio;

#[tokio::test]
#[serial]
async fn test_legatio_model_request_flow_with_mock_openai() {
    // 1. MOCK the OpenAI API
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/chat/completions")
            .header("Authorization", "Bearer legatio_test_key")
            .header("Content-Type", "application/json")
            .body_contains("Testing Legatio prompt.");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(
                r#"{
                    "choices": [ { "message": { "content": "Mocked reply from OpenAI" } } ]
                }"#,
            );
    });

    if env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }
    env::set_var("OPENAI_API_URL", format!("{}/v1/chat/completions", server.base_url()));

    // 2. Setup config
    let user_conf = UserConfig {
        ai_conf: AiConfig {
            llm: Framework::OpenAI,
            model: "gpt-4.1".into(),
            max_token: Some(256),
        },
        theme: "Tokyo Storm".to_string(),
        ask_conf: false,
    };
    //store_config(&user_conf).unwrap();

    // 3. Setup your test Project (if you need to interact with DB, etc)
    // This part would usually use your sqlx::SqlitePool and relevant project logic
    // let pool = ... ; // setup a test pool/database
    // let project = Project::new("/tmp/test_proj");
    // store_project(&pool, &project).await.unwrap();

    // 4. INSTANTIATE a Question (structure your call as in production)
    let q = Question {
        system_prompt: Some("You are precise.".to_string()),
        messages: None,
        new_prompt: "Testing Legatio prompt.".to_string(),
    };

    // 5.1 TEST correct llm & model
    println!(
        "Llm: {:?}, Model: {}, URL: {:?}",
        user_conf.ai_conf.llm,
        user_conf.ai_conf.model,
        std::env::var("OPENAI_API_URL")
    );

    // 5.2 RUN the model request logic (as in your Legatio flow)
    let result = ask_question(&AiConfig {
            llm: Framework::OpenAI,
            model: "gpt-4.1".into(),
            max_token: Some(256),
        }, q).await.expect("Should get OpenAI mock reply");

    assert_eq!(result, "Mocked reply from OpenAI");
    mock.assert();

    env::remove_var("OPENAI_API_KEY");
    env::remove_var("OPENAI_API_URL");
}

#[tokio::test]
#[serial]
async fn test_legatio_model_request_flow_with_mock_anthropic() {
    // 1. MOCK the OpenAI API
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/chat/completions")
            .header("Authorization", "Bearer legatio_test_key")
            .header("Content-Type", "application/json")
            .body_contains("Testing Legatio prompt.");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(
                r#"{
                    "choices": [ { "message": { "content": "Mocked reply from OpenAI" } } ]
                }"#,
            );
    });

    if env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
        return;
    }
    env::set_var("ANTHROPIC_API_URL", format!("{}/v1/messages", server.base_url()));

    // 2. Setup config
    let user_conf = UserConfig {
        ai_conf: AiConfig {
            llm: Framework::Anthropic,
            model: "claude-opus-4-0".into(),
            max_token: Some(256),
        },
        theme: "Tokyo Storm".to_string(),
        ask_conf: false,
    };
    //store_config(&user_conf).unwrap();

    // 3. Setup your test Project (if you need to interact with DB, etc)
    // This part would usually use your sqlx::SqlitePool and relevant project logic
    // let pool = ... ; // setup a test pool/database
    // let project = Project::new("/tmp/test_proj");
    // store_project(&pool, &project).await.unwrap();

    // 4. INSTANTIATE a Question (structure your call as in production)
    let q = Question {
        system_prompt: Some("You are precise.".to_string()),
        messages: None,
        new_prompt: "Testing Legatio prompt.".to_string(),
    };

    // 5.1 TEST correct llm & model
    println!(
        "Llm: {:?}, Model: {}, URL: {:?}",
        user_conf.ai_conf.llm,
        user_conf.ai_conf.model,
        std::env::var("OPENAI_API_URL")
    );

    // 5.2 RUN the model request logic (as in your Legatio flow)
    let result = ask_question(&AiConfig {
            llm: Framework::Anthropic,
            model: "claude-3-5-haiku-20241022".into(),
            max_token: Some(256),
        }, q).await.expect("Should get Anthropic mock reply");

    assert_eq!(result, "Mocked reply from OpenAI");
    mock.assert();

    env::remove_var("OPENAI_API_KEY");
    env::remove_var("OPENAI_API_URL");
}

//#[tokio::test]
//#[serial]
//async fn test_legatio_produce_question_full_flow() {
//    // 1. Mock OpenAI API
//    let server = MockServer::start();
//    let api_url = format!("{}/v1/chat/completions", server.base_url());
//
//    let mock = server.mock(|when, then| {
//        when.method(POST)
//            .header("Authorization", "Bearer legatio_test_key")
//            .header("Content-Type", "application/json")
//            .body_contains("Testing Legatio prompt.");
//        then.status(200)
//            .header("content-type", "application/json")
//            .body(
//                r#"{"choices":[{"message":{"content":"Mocked chat response from OpenAI"}}]}"#,
//            );
//    });
//
//    env::set_var("OPENAI_API_KEY", "legatio_test_key");
//    env::set_var("OPENAI_API_URL", &api_url);
//
//    // 2. Setup temp database
//    let db_temp = NamedTempFile::new().unwrap();
//    let db_path = db_temp.path().to_str().unwrap();
//    let pool = SqlitePoolOptions::new()
//        .max_connections(2)
//        .connect(&format!("sqlite://{}", db_path))
//        .await
//        .unwrap();
//
//    // 3. Setup UserConfig
//    let ai_config = AiConfig {
//        llm: Framework::OpenAI,
//        model: "gpt-4-turbo".to_string(),
//        max_token: Some(128),
//    };
//    let user_config = UserConfig {
//        ai_conf: AiConfig {
//            llm: Framework::OpenAI,
//            model: "gpt-4-turbo".to_string(),
//            max_token: Some(128),
//        },
//        theme: "Tokyo Storm".into(),
//        ask_conf: false,
//    };
//    store_config(&user_config).expect("store config works");
//
//    // 4. Setup a new Legatio app with 1 test project and a prompt
//    let mut app = Legatio::default();
//
//    let project = Project::new("/tmp/legatio-test-project");
//    legatio::core::project::store_project(&pool, &project)
//        .await
//        .expect("project insert works");
//    app.current_project = Some(project.clone());
//
//    // If your `produce_question` expects `current_prompt`, set it up here too,
//    // otherwise it will fall back to an empty prompt.
//
//    // 5. Run the flow -- will hit the mocked ask_ai endpoint
//    app
//        .produce_question(&pool)
//        .await
//        .expect("should produce question without error");
//
//    // 6. Optional: Check the DB for the new prompt, or verify in memory state
//    // (skipped here, but you can query the pool for the prompt table)
//
//    // 7. Assert that the model mock was hit
//    mock.assert();
//
//    env::remove_var("OPENAI_API_KEY");
//    env::remove_var("OPENAI_API_URL");
//}
