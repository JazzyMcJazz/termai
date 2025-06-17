use anyhow::Result;
use mcp_core::{
    client::{Client, ClientBuilder},
    transport::{ClientSseTransport, ClientSseTransportBuilder, ClientStdioTransport, Transport},
    types::{ClientCapabilities, InitializeResponse, RootCapabilities, Tool},
};
use rig::{agent::AgentBuilder, completion::CompletionModel};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStdioInfo {
    pub name: String,
    pub version: String,
    pub program: String,
    pub args: Vec<String>,
    pub enabled: bool,
    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSseInfo {
    pub name: String,
    pub version: String,
    pub url: String,
    pub enabled: bool,
    pub tools: Vec<Tool>,
}

#[derive(Clone)]
pub enum McpClient {
    StdIo(Client<ClientStdioTransport>, ClientStdioInfo),
    Sse(Client<ClientSseTransport>, ClientSseInfo),
}

impl McpClient {
    pub fn set_enabled(&mut self, enabled: bool) {
        match self {
            McpClient::StdIo(_, info) => info.enabled = enabled,
            McpClient::Sse(_, info) => info.enabled = enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            McpClient::StdIo(_, info) => info.enabled,
            McpClient::Sse(_, info) => info.enabled,
        }
    }

    pub fn name(&self) -> String {
        match self {
            McpClient::StdIo(_, info) => info.name.clone(),
            McpClient::Sse(_, info) => info.name.clone(),
        }
    }

    pub fn version(&self) -> String {
        match self {
            McpClient::StdIo(_, info) => info.version.clone(),
            McpClient::Sse(_, info) => info.version.clone(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        match self {
            McpClient::StdIo(client, info) => {
                if client.assert_initialized().await.is_err() {
                    let (res, tools) = initialize_client(client).await?;
                    info.name = res.server_info.name;
                    info.version = res.server_info.version;
                    info.tools = tools;
                }
            }
            McpClient::Sse(client, info) => {
                if client.assert_initialized().await.is_err() {
                    let (res, tools) = initialize_client(client).await?;
                    info.name = res.server_info.name;
                    info.version = res.server_info.version;
                    info.tools = tools;
                }
            }
        }

        Ok(())
    }

    pub async fn add_tools<M: CompletionModel>(
        &self,
        agent_builder: AgentBuilder<M>,
    ) -> AgentBuilder<M> {
        let tools = self.tools().await;

        match self {
            McpClient::StdIo(client, _) => tools.iter().fold(agent_builder, |builder, tool| {
                builder.mcp_tool(tool.clone(), client.clone())
            }),
            McpClient::Sse(client, _) => tools.iter().fold(agent_builder, |builder, tool| {
                builder.mcp_tool(tool.clone(), client.clone())
            }),
        }
    }

    async fn tools(&self) -> &Vec<Tool> {
        match self {
            McpClient::StdIo(_, info) => &info.tools,
            McpClient::Sse(_, info) => &info.tools,
        }
    }
}

impl From<McpClientConfig> for McpClient {
    fn from(config: McpClientConfig) -> Self {
        match config {
            McpClientConfig::StdIo(name, version, program, args, enabled) => {
                let program_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let transport = ClientStdioTransport::new(&program, &program_args)
                    .expect("Failed to create transport");
                let client = ClientBuilder::new(transport)
                    .set_protocol_version(mcp_core::types::ProtocolVersion::V2025_03_26)
                    .set_client_info(name.to_owned(), version.to_owned())
                    .set_capabilities(ClientCapabilities {
                        experimental: Some(json!({})),
                        roots: Some(RootCapabilities {
                            list_changed: Some(false),
                        }),
                        sampling: Some(json!({})),
                    })
                    .build();
                McpClient::StdIo(
                    client,
                    ClientStdioInfo {
                        name,
                        version,
                        program,
                        args,
                        enabled,
                        tools: vec![],
                    },
                )
            }
            McpClientConfig::Sse(name, version, url, enabled) => {
                let transport = ClientSseTransportBuilder::new(url.clone()).build();
                let client = ClientBuilder::new(transport)
                    .set_protocol_version(mcp_core::types::ProtocolVersion::V2025_03_26)
                    .set_client_info(name.to_owned(), version.to_owned())
                    .set_capabilities(ClientCapabilities {
                        experimental: Some(json!({})),
                        roots: Some(RootCapabilities {
                            list_changed: Some(false),
                        }),
                        sampling: Some(json!({})),
                    })
                    .build();
                McpClient::Sse(
                    client,
                    ClientSseInfo {
                        name,
                        version,
                        url,
                        enabled,
                        tools: vec![],
                    },
                )
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpClientConfig {
    StdIo(String, String, String, Vec<String>, bool), // Name, Version, Program, Arguments, Enabled
    Sse(String, String, String, bool),                // Name, Version, URL, Enabled
}

impl From<McpClient> for McpClientConfig {
    fn from(client: McpClient) -> Self {
        match client {
            McpClient::StdIo(_, info) => McpClientConfig::StdIo(
                info.name,
                info.version,
                info.program,
                info.args,
                info.enabled,
            ),
            McpClient::Sse(_, info) => {
                McpClientConfig::Sse(info.name, info.version, info.url, info.enabled)
            }
        }
    }
}

impl Serialize for McpClient {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let config: McpClientConfig = self.clone().into();
        config.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for McpClient {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let config = McpClientConfig::deserialize(deserializer)?;
        let client = McpClient::from(config);
        Ok(client)
    }
}

async fn initialize_client<T: Transport>(
    client: &Client<T>,
) -> Result<(InitializeResponse, Vec<Tool>)> {
    client.open().await?;

    let res = client.initialize().await?;

    let tools = match client.list_tools(None, None).await {
        Ok(res) => res.tools,
        Err(_) => vec![],
    };

    Ok((res, tools))
}
