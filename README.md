# Switchboard! - v0.0.1

Aiming to be a lightweight, modern looking and easy to deploy chat interface to large language models
that can be used as-is and easily self-hosted, themed and modified as needed.

Switchboard is a web 'app' that is shipped as a single web page (HTML, JS, WASM and CSS) and is written
with Rust, targetting WASM, for high performance. Other than communicating to the OpenAI-compatible API
you set in the configuration, the app makes no other connections besides pulling the [lineicons](https://lineicons.com/) font.

Note: Everything is under heavy development right now; Consider any data as disposable as log compatability isn't guaranteed yet.


## Demo Site

[Try The Live Demo Site!](https://animal-machine.com/switchboard/) 

*(Requires an [OpenRouter AI key](https://openrouter.ai/) or your own OpenAI compatible endpoint and key)*


## Features

* **Clean and Simple**: Setup your API settings and you're good to go!

* **Multimodal support for images**: Paste an image in from the clipboard to the message and it will be
  sent to the AI with the text.

* **Themable**: Clone this repo and change [tailwind-import.css](./tailwind-import.css) to have the webpage look
  how you want it to look. Or fork and update the modular components for deeper customization.

* **Supports OpenAI-compatbile APIs** such as [llama.cpp's server](https://github.com/ggml-org/llama.cpp), which when
  combined with [llama-swap](https://github.com/mostlygeek/llama-swap), will allow multiple models to be served.
  It also works with popular cloud API providers such as [openrouter](https://openrouter.ai/).

* Save slots allow you to save your conversation in the web browser's storage, meaning no file management
  is required. 

* Chatlogs can be imported and exported using JSON.


## Requirements

Building from source requires the `git` and `tailwindcss` executables to be in your path, in 
addition to having a [Rust toolchain](https://www.rust-lang.org/) installed.


## Building and Running

Once you have a Rust toolchain, make sure the `tailwindcss` executable is in your
path becausae it's referenced in the build scripts. The build process will also expect `git`
to be in your path, too.

Install [Trunk](https://trunkrs.dev/) to build and serve the project and make sure
the resulting `trunk` executable is in the PATH for your shell:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

The pull down the project's source code, change to the dirctory and serve with `trunk`:

```bash
git clone https://github.com/tbogdala/switchboard.git
cd switchboard
trunk serve --release
```
At this point, browse to `http://localhost:8080` on your browser to see the app.

When deploying to a website that is going to host it someplace other than the root,
you can generate the website with the following trunk command, where the website files are
located in `/switchboard`:

```bash
trunk serve --release --public-url /switchboard/
```


## Notes on Usage

* Firefox, and it's derivatives, may have a small local storage pool by default. You can
  browse to `about:config`, search for the `dom.storage.default_quota` parameter and setting
  it to `102400` for 100MB of local storage.

* Currently, errors are under reported and if something is not acting right, bring up the
  Web Developer Tools (or equivalent) and look at the 'console' tab for error mesasages.

* For image support, the current implementation only sends the image data when it's the
  last message in the chatlog so that it doesn't use up token processing with each subsequent request.
  Feedback can be provided on this, if there's a more preffered way to handle images
  in the chatlog; please open a ticket or start a discussion. Image size is also not
  factored into the token budget estimation (unsure if this has negative effects in practice).


## Roadmap

The first major milestone is to get the small version of this project usable. This will entail
just a single endpoint and a simple system message for prompt generation. Chatlog features will
be basic and just include delete, regenerate and edit capabilities. This will provide a minimalistic
starting point for others to use as just a general chat interface.

Afterwards, that version will get branched and named the 'nano' version of the project.
The main branch will then work on more advanced features such as multiple characters, 
multiple API endpoints and possibly advanced integrations with something like an MCP server. 
Things that are more complicated.


## License

This code is open sourced under the MIT license.
