use sqlx::SqlitePool;

use crate::{services::ui::usr_prompts, utils::structs::Prompt};

fn create_test_prompts() -> Vec<Prompt> {
    vec![
        // Single root prompt
        Prompt {
            prompt_id: "1".to_string(),
            prev_prompt_id: "0".to_string(), // No parent
            scroll_id: "scroll_1".to_string(),
            content: "Root prompt content 1".to_string(),
            output: "Root output 1".to_string(),
        },
        // Single child prompt
        Prompt {
            prompt_id: "2".to_string(),
            prev_prompt_id: "1".to_string(), // Child of 1
            scroll_id: "scroll_1".to_string(),
            content: "Child prompt content 1".to_string(),
            output: "Child output 1".to_string(),
        },
        // Chain of depth n (3 in this case)
        Prompt {
            prompt_id: "3".to_string(),
            prev_prompt_id: "2".to_string(), // Child of 2
            scroll_id: "scroll_1".to_string(),
            content: "Grandchild prompt content 1".to_string(),
            output: "Grandchild output 1".to_string(),
        },
        Prompt {
            prompt_id: "4".to_string(),
            prev_prompt_id: "3".to_string(), // Child of 3
            scroll_id: "scroll_1".to_string(),
            content: "Great-grandchild prompt content 1".to_string(),
            output: "Great-grandchild output 1".to_string(),
        },
        // Multiple root prompts in the same scroll
        Prompt {
            prompt_id: "5".to_string(),
            prev_prompt_id: "0".to_string(), // No parent
            scroll_id: "scroll_1".to_string(),
            content: "Second root prompt content".to_string(),
            output: "Second root output".to_string(),
        },
        // Multiple children of the same parent
        Prompt {
            prompt_id: "6".to_string(),
            prev_prompt_id: "5".to_string(), // Child of 5
            scroll_id: "scroll_1".to_string(),
            content: "Child prompt 1 of root 5".to_string(),
            output: "Child output 1 of 5".to_string(),
        },
        Prompt {
            prompt_id: "7".to_string(),
            prev_prompt_id: "5".to_string(), // Another child of 5
            scroll_id: "scroll_1".to_string(),
            content: "Child prompt 2 of root 5".to_string(),
            output: "Child output 2 of 5".to_string(),
        },
        // Prompts in a different scroll
        Prompt {
            prompt_id: "8".to_string(),
            prev_prompt_id: "0".to_string(),
            scroll_id: "scroll_2".to_string(),
            content: "Root of another scroll".to_string(),
            output: "Root output of scroll_2".to_string(),
        },
        Prompt {
            prompt_id: "9".to_string(),
            prev_prompt_id: "8".to_string(),
            scroll_id: "scroll_2".to_string(),
            content: "Child of another scroll".to_string(),
            output: "Child output of scroll_2".to_string(),
        },
        // Edge case: Prompt with no content or output
        Prompt {
            prompt_id: "10".to_string(),
            prev_prompt_id: "1".to_string(),
            scroll_id: "scroll_1".to_string(),
            content: "".to_string(),
            output: "".to_string(),
        },
    ]
}


#[tokio::test]
pub async fn test_usr_prompts() {
    let prompts = create_test_prompts();

    // Mock a dummy pool for the purpose of testing
    let mock_pool = SqlitePool::connect(":memory:").await.unwrap();
    let prompt = Prompt {
        prompt_id: "1".to_string(),
        prev_prompt_id: "0".to_string(),
        scroll_id: "scroll_1".to_string(),
        content: "Root prompt content".to_string(),
        output: "Root output".to_string(),
    };

    usr_prompts(&mock_pool, &prompt).await.unwrap();
}
