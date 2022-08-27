use aws_sdk_dynamodb::{model::AttributeValue, types::DateTime, Client};
use egnitely_client::{RequestContext, Result};
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_0_17::to_item;
use serde_json::{json, Value};
use std::{time::SystemTime};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct FunctionContextData {
    pub table_name: String,
}

pub async fn handler(mut _ctx: RequestContext, _input: Value) -> Result<Value> {
    let mut item_data = to_item(_input)?;
    let function_id = Uuid::new_v4().to_string();
    if let Some(sdk_config) = _ctx.aws_sdk_config() {
        let client = Client::new(&sdk_config);

        item_data.insert("id".to_string(), AttributeValue::S(function_id.clone()));
        item_data.insert(
            "created_at".to_string(),
            AttributeValue::N(DateTime::from(SystemTime::now()).as_nanos().to_string()),
        );
        item_data.insert(
            "updated_at".to_string(),
            AttributeValue::N(DateTime::from(SystemTime::now()).as_nanos().to_string()),
        );
        item_data.insert("deleted_at".to_string(), AttributeValue::Null(true));

        let context_data = serde_json::from_value::<FunctionContextData>(_ctx.data())?;

        let request = client
            .put_item()
            .table_name(context_data.table_name)
            .set_item(Some(item_data));

        request.send().await?;
    }
    Ok(json!({
        "message": "function created successfully",
        "data": {
            "id" : function_id,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_dynamodb::Credentials;

    #[tokio::test]
    async fn trigger_function() {
        let config = aws_config::from_env()
            .credentials_provider(Credentials::new(
                "PUT_ACCESS_KEY_HERE".to_string(),
                "PUT_ACCESS_SECRET_HERE".to_string(),
                None,
                None,
                "local",
            ))
            .region("ap-south-1")
            .load()
            .await;

        let resp = handler(
            RequestContext::new(
                "test".to_string(),
                "test".to_string(),
                Some(config),
                json!({
                    "table_name": "functions"
                }),
                json!({}),
            ),
            json!({
                "name": "create_function",
                "label": "Create Function",
                "description": "This function can be used to create a function in database",
                "language": "rust",
                "input_schema": {},
                "logo_url": "https://egnitely.com/egnitely.png",
                "repository": "https://github.com/egnitely/egnitely-functions",
                "branch": "main",
                "repo_sub_directory": "create_function",
            }),
        )
        .await
        .unwrap();

        assert_eq!("function created successfully", resp["message"]);
    }
}