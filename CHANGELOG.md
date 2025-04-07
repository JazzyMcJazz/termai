# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.3

### Changed
- Commandline parsing is now done using Clap.

### Removed
- The Ask feature. Use Chat instead.

## 0.2.2

### Added

- **Shell environment detection on Windows, MacOS and Linux.** This information is used to provide the assistant additional context about the user's environment when suggesting commands or providing other help with in the terminal. Only shell name (e.g. `pwsh`, `bash`) and OS name (e.g. `Windows`, `MacOS`) is provided. No other information about the user's system is automatically provided.

## 0.2.1

### Added
- Ability to navigate backwards in menus.

### Changed
- Made improvements to the overall user experience when navigating menus.

### Removed
- The word "thanks" as an exit word in chat mode.

### Fixed
- Setting system message correctly for Anthropic API when revising a suggested command.
- Revising a suggested command now uses the correct system message.
- Fixed a bug where it was impossible to exit from the `ask` input prompt.

## 0.2.0

### Added
- Support for Anthropic.
- Checking for available models with providers before displaying the model selection menu.

### Fixed
- Handling of API errors (e.g. insufficient credits or invalid API key).

### Changed
- Put changelog option at the bottom of the options menu.

### Breaking Changes
- This update comes with major changes to the config file format, meaning all previously stored configuration can no longer be used. This includes, but is not limited to, API keys configured for any provider.

## 0.1.3 

### Changed
- Made improvements to streamed chat responses.

## 0.1.0

### Added
- Ask feature. Similar to `chat`, but the program exits automatically after a single response.
- Option to print latest changelog (found in the options menu or via `termai changelog`).
- Streaming option for the `chat` feature **(experimental)**.

### Changed
- Moved model selection out of provider settings.
