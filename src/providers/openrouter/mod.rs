//! This module provides the Openrouter provider, wrapping OpenAI Chat Completions for Openrouter requests.
//!
//! NOTE: xAI might not be fully compatible with the OpenAI API. Please refer to the xAI documentation for more information.

pub mod capabilities;

// Generate the settings module
crate::openai_compatible_settings!(
    OpenrouterProviderSettings,
    OpenrouterProviderSettingsBuilder,
    "Openrouter",
    "https://openrouter.ai/api/v1",
    "OPENROUTER_API_KEY"
);

// Generate the provider struct and builder
crate::openai_compatible_provider!(
    Openrouter,
    OpenrouterBuilder,
    OpenrouterProviderSettings,
    "openrouter"
);

// Generate the language model implementation
crate::openai_compatible_language_model!(Openrouter);
