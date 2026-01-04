//! Language model implementation for the OpenAI Chat Completions provider.

use crate::core::capabilities::ModelName;
use crate::core::client::Client;
use crate::core::language_model::{
    LanguageModel, LanguageModelOptions, LanguageModelResponse, LanguageModelResponseContentType,
    LanguageModelStreamChunk, LanguageModelStreamChunkType, ProviderStream,
};
use crate::core::messages::AssistantMessage;
use crate::core::tools::ToolCallInfo;
use crate::error::Result;
use crate::providers::openai_chat_completions::OpenAIChatCompletions;
use crate::providers::openai_chat_completions::client::{self, types};
use async_trait::async_trait;
use futures::StreamExt;

#[async_trait]
impl<M: ModelName> LanguageModel for OpenAIChatCompletions<M> {
    fn name(&self) -> String {
        self.options.model.clone()
    }

    async fn generate_text(
        &mut self,
        options: LanguageModelOptions,
    ) -> Result<LanguageModelResponse> {
        let mut options: client::ChatCompletionsOptions = options.into();
        options.model = self.options.model.clone();
        self.options = options;

        let response: types::ChatCompletionsResponse = self.send(&self.settings.base_url).await?;

        // Convert choices to LanguageModelResponse
        let mut contents = Vec::new();

        for choice in response.choices {
            // Handle text content
            if let Some(text) = choice.message.content
                && !text.is_empty()
            {
                contents.push(LanguageModelResponseContentType::Text(text));
            }

            // Handle tool calls
            if let Some(tool_calls) = choice.message.tool_calls {
                for tool_call in tool_calls {
                    let mut tool_info = ToolCallInfo::new(tool_call.function.name);
                    tool_info.id(tool_call.id);
                    tool_info.input(
                        serde_json::from_str(&tool_call.function.arguments).unwrap_or_default(),
                    );
                    contents.push(LanguageModelResponseContentType::ToolCall(tool_info));
                }
            }
        }

        Ok(LanguageModelResponse {
            contents,
            usage: response.usage.map(|u| u.into()),
        })
    }

    async fn stream_text(&mut self, options: LanguageModelOptions) -> Result<ProviderStream> {
        let mut options: client::ChatCompletionsOptions = options.into();
        options.model = self.options.model.clone();
        options.stream = Some(true);
        options.stream_options = Some(types::StreamOptions {
            include_usage: Some(true),
            include_obfuscation: Some(false),
        });
        self.options = options;

        let stream = self.send_and_stream(&self.settings.base_url).await?;

        // Map stream events to SDK stream chunks
        let stream = stream.map(|evt_res| match evt_res {
            Ok(types::ChatCompletionsStreamEvent::Chunk(chunk)) => {
                let mut results = Vec::new();

                for choice in chunk.choices {
                    // Text delta
                    if let Some(content) = choice.delta.content
                        && !content.is_empty()
                    {
                        results.push(LanguageModelStreamChunk::Delta(
                            LanguageModelStreamChunkType::Text(content),
                        ));
                    }

                    // Tool call delta
                    if let Some(tool_calls) = choice.delta.tool_calls {
                        for tool_call in tool_calls {
                            if let Some(args) = tool_call
                                .function
                                .as_ref()
                                .and_then(|f| f.arguments.as_ref())
                            {
                                results.push(LanguageModelStreamChunk::Delta(
                                    LanguageModelStreamChunkType::ToolCall(args.clone()),
                                ));
                            }
                        }
                    }

                    // Handle finish
                    if let Some(finish_reason) = choice.finish_reason {
                        let usage = chunk.usage.clone().map(|u| u.into());

                        match finish_reason.as_str() {
                            "stop" | "length" => {
                                results.push(LanguageModelStreamChunk::Done(AssistantMessage {
                                    content: LanguageModelResponseContentType::Text(String::new()),
                                    usage,
                                }));
                            }
                            "tool_calls" => {
                                // Tool calls are completed - signal done
                                results.push(LanguageModelStreamChunk::Done(AssistantMessage {
                                    content: LanguageModelResponseContentType::Text(String::new()),
                                    usage,
                                }));
                            }
                            "content_filter" => {
                                results.push(LanguageModelStreamChunk::Delta(
                                    LanguageModelStreamChunkType::Failed(
                                        "Content filtered".to_string(),
                                    ),
                                ));
                            }
                            _ => {}
                        }
                    }
                }

                Ok(results)
            }
            Ok(types::ChatCompletionsStreamEvent::Done) => Ok(vec![]),
            Ok(types::ChatCompletionsStreamEvent::Error(e)) => {
                Ok(vec![LanguageModelStreamChunk::Delta(
                    LanguageModelStreamChunkType::Failed(e),
                )])
            }
            Err(e) => Err(e),
        });

        Ok(Box::pin(stream))
    }
}
