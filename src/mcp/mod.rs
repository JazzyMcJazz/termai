use rmcp::{model::Tool, serve_client, service::RunningService, transport::TokioChildProcess, RoleClient};
use tokio::process::Command;


pub struct McpClient {
    client: RunningService<RoleClient, ()>,
}

impl McpClient {
    pub async fn new(command: &str, args: Vec<&str>) -> Self {
        let client= serve_client(
            (),
            TokioChildProcess::new(Command::new(command).args(args)).unwrap(),
        )
        .await.unwrap(); 

        McpClient {
            client,
        }
    }

    pub async fn tools(&self) -> Option<Vec<Tool>> {
        if self.client.peer_info().capabilities.tools.is_none() {
            return None;
        }

        match self.client.list_all_tools().await {
            Ok(tools) => Some(tools),
            Err(e) => {
                eprintln!("Error listing tools: {}", e);
                None
            }
        }
    }

    pub async fn cancel(self) {
        let _ = self.client.cancel().await;
    }
}
