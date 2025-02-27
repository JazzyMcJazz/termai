# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0

### Added
- Support for Anthropic
- Checking for available models with providers before displaying the model selection menu

### Fixed
- Handling of API errors (e.g. insufficient credits or invalid API key)

### Breaking Changes
- This update comes with major changes to the config file format, meaning all previously stored configuration can no longer be used. This includes, but is not limited to, API keys configured for any provider.

## 0.1.3 

### Changed
- Made improvements to streamed chat responses.

## 0.1.0

### Added
- Ask feature. Similar to `chat`, but the program exits automatically after a single response
- Option to print latest changelog (found in the options menu or via `termai changelog`)
- Streaming option for the `chat` feature **(experimental)**

### Changed
- Moved model selection out of provider settings
