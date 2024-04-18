<p align="center">
  <a href="https://github.com/efugier/smartcat/discussions">
    <img src="https://img.shields.io/badge/commmunity-discussion-blue?style=flat-square" alt="community discussion">
  </a>
  <a href="https://github.com/efugier/smartcat/actions/workflows/ci.yml">
      <img src="https://github.com/efugier/smartcat/actions/workflows/ci.yml/badge.svg?branch=main" alt="Github Actions CI Build Status">
  </a>
  <a href="https://crates.io/crates/smartcat">
      <img src="https://img.shields.io/crates/v/smartcat.svg?style=flat-square" alt="crates.io">
  </a>
  <br>
</p>

<p align="center">
  <img src="assets/sc_logo.png" width="200">
</p>

# smartcat (sc)

Puts a brain behind `cat`! CLI interface to bring language models in the Unix ecosystem and allow power users to make the most out of llms.

What makes it special:
- made for power users;
- minimalist, built following to the unix philosophy with terminal and editor intergation in mind;
- good io handling to insert user input in prompts and use the result in cli-based workflows;
- built-in partial prompt to make the model play nice as a cli tool;
- full configurability on which API, LLM version and temperature you use;
- write and save your own prompt templates for faster reccuring tasks (simplify, optimize, tests, etc);
- conversation support;
- glob expressions to include context files.

Currently supports **OpenAi**, **Mistral AI** and **Anthropic** APIs.


![](assets/workflow.gif)

- [Installation](#installation-)
- [Usage](#usage)
- [A few examples to get started 🐈‍⬛](#a-few-examples-to-get-started-)
  - [Integrating with editors](#integrating-with-editors)
    - [Example workflows](#example-workflows)
- [Configuration](#configuration) ← please read this carefully
- [Developping](#developping)

## Installation


### With Cargo

With an **up to date** [rust and cargo](https://www.rust-lang.org/tools/install) setup (you might consider running `rustup update`):

```
cargo install smartcat
```

run this command again to update `smartcat`.

### By downloading the binary

Chose the one compiled for your platform on the [release page](https://github.com/efugier/smartcat/releases).

(the binary is named `sc`)

---

On the first run, `smartcat` will ask you to generate some default configuration files if it cannot find them.
More about that in the [configuration section](#Configuration).

A `default` prompt is needed for `smartcat` to know which api and model to hit.

## Usage

```text
Usage: sc [OPTIONS] [INPUT_OR_CONFIG_REF] [INPUT_IF_CONFIG_REF]

Arguments:
  [INPUT_OR_CONFIG_REF]  ref to a prompt from config or straight input (will use `default` prompt template)
  [INPUT_IF_CONFIG_REF]  if the first arg matches a config ref, the second will be used as input

Options:
  -e, --extend-conversation        whether to extend the previous conversation or start a new one
  -r, --repeat-input               whether to repeat the input before the output, useful to extend instead of replacing
      --api <API>                  overrides which api to hit [possible values: openai, mistral, anthropic]
  -m, --model <MODEL>              overrides which model (of the api) to use
  -t, --temperature <TEMPERATURE>  temperature higher means answer further from the average
  -l, --char-limit <CHAR_LIMIT>    max number of chars to include, ask for user approval if more, 0 = no limit
  -c, --context <CONTEXT>...       glob patterns or list of files to use the content as context
                                   make sure it's the last arg.
  -h, --help                       Print help
  -V, --version                    Print version
```

You can use it to **accomplish tasks in the CLI** but **also in your editors** (if they are good unix citizens, i.e. work with shell commands and text streams) to complete, refactor, write tests... anything!

The key to make this work seamlessly is a good default prompt that tells the model to behave like a CLI tool an not write any unwanted text like markdown formatting or explanations.

## A few examples to get started 🐈‍⬛

```
sc "say hi"  # just ask

sc test                         # use templated prompts
sc test "and parametrize them"  # extend them on the fly

sc "explain how to use this program" -c **/*.md main.py  # use files as context

git diff | sc "summarize the changes"  # pipe data in

cat en.md | sc "translate in french" >> fr.md   # write data out
sc -e "use a more informal tone" -t 2 >> fr.md  # extend the conversation and raise the temprature
```

### Integrating with editors

The key for a good integration in editors is a good default prompt (or set of) combined with the `-p` flag for precising the task at hand.
The `-r` flag can be used to decide whether to replace or extend the selection.

#### Vim

Start by selecting some text, then press `:`. You can then pipe the selection content to `smartcat`.

```
:'<,'>!sc "replace the versions with wildcards"
```

```
:'<,'>!sc "fix this function"
```

will **replace** the current selection with the same text transformed by the language model.

```
:'<,'>!sc -r write_test
```

will **append** at the end of the current selection the result of the language model.

Add the following remap to your vimrc for easy access:

```vimrc
nnoremap <leader>sc :'<,'>!sc
```

#### Helix and Kakoune

Same concept, different shortcut, simply press the pipe key to redirect the selection to `smarcat`.

```
pipe:sc write_test -r
```
With some remapping you may have your most reccurrent action attached to few keystrokes e.g. `<leader>wt`!

#### Example Workflows

To enhance coding:

select a struct

```
:'<,'>!sc "implement the traits FromStr and ToString for this struct"
```

select the generated impl block

```
:'<,'>!sc -e "can you make it more concise?"
```

put the cursor at the bottom of the file and give example usage as input

```
:'<,'>!sc -e "now write tests for it knowing it's used like this" -c src/main.rs
```

...

To have a full conversation with a llm from a markdown file:

```
vim problem_solving.md

> write your question as comment in the markdown file then select your question
> and send it to smartcat using the aforementioned trick, use `-r` to repeat the input.

If you wan to continue the conversation, write your new question as a comment and repeat
the previous step with `-e -r`.

> This allows you to keep track of your questions and forms a nice reusable document.
```


For quick questions:

```
sc "my quick question"
```

which will likely be your fasted path anser because you'll have a shortcut to opens your terminal and there will be no tab finding, no logins, no redirects etc.

# Configuration

- by default lives at `$HOME/.config/smartcat`
- the directory can be set using the `SMARTCAT_CONFIG_PATH` environement variable
- use `#[<input>]` as the placeholder for input when writing prompts
- the default model is `gpt-4` but I recommend trying the latest ones and see which one works best for you;
- you can play with the temperature and set a default for each prompt depending on its use case.

Three files are used:

`conversation.toml`

which stores the latest chat if you need to continue it.

`.api_configs.toml`

```toml
[openai]  # each supported api has their own config section with api and url
api_key = "<your_api_key>"
default_model = "gpt-4-turbo-preview"
url = "https://api.openai.com/v1/chat/completions"

[mistral]
api_key_command = "pass mistral/api_key"  # you can use a command to grab the key
default_model = "mistral-medium"
url = "https://api.mistral.ai/v1/chat/completions"

[anthropic]
api_key = "<yet_another_api_key>"
url = "https://api.anthropic.com/v1/messages"
default_model = "claude-3-opus-20240229"
version = "2023-06-01"
```

`prompts.toml`

```toml
[default]  # a prompt is a section
api = "openai"  # must refer to an entry in the `.api_configs.toml` file
model = "gpt-4-1106-preview"  # each prompt may define its own model

[[default.messages]]  # then you can list messages
role = "system"
content = """\
You are an extremely skilled programmer with a keen eye for detail and an emphasis on readable code. \
You have been tasked with acting as a smart version of the cat unix program. You take text and a prompt in and write text out. \
For that reason, it is of crucial importance to just write the desired output. Do not under any circumstance write any comment or thought \
as you output will be piped into other programs. Do not write the markdown delimiters for code as well. \
Sometimes you will be asked to implement or extend some input code. Same thing goes here, write only what was asked because what you write will \
be directly added to the user's editor. \
Never ever write ``` around the code. \
Now let's make something great together!
"""

[empty]  # always nice to have an empty prompt available
api = "openai"
messages = []

[write_tests]
api = "openai"
temperature = 0.0

[[write_tests.messages]]
role = "system"
content = """\
You are an extremely skilled programmer with a keen eye for detail and an emphasis on readable code. \
You have been tasked with acting as a smart version of the cat unix program. You take text and a prompt in and write text out. \
For that reason, it is of crucial importance to just write the desired output. Do not under any circumstance write any comment or thought \
as you output will be piped into other programs. Do not write the markdown delimiters for code as well. \
Sometimes you will be asked to implement or extend some input code. Same thing goes here, write only what was asked because what you write will \
be directly added to the user's editor. \
Never ever write ``` around the code. \
Now let's make something great together!
"""

[[write_tests.messages]]
role = "user"
# the following placeholder string #[<input>] will be replaced by the input
# each message seeks it and replaces it
content ='''Write tests using pytest for the following code. Parametrize it if appropriate.

#[<input>]
'''
```

see [the config setup file](./src/config/mod.rs) for more details.

## Developping

Some tests rely on environement variables and don't behave well with multi-threading. They are marked with `#[serial]` from the [serial_test](https://docs.rs/serial_test/latest/serial_test/index.html) crate.

### State of the project

Smartcat has reached an acceptable feature set. The focus is now on upgrading the codebase quality as I hadn't really touched rust since 2019 and it shows.

#### TODO

- [ ] make it available on homebrew
- [ ] automagical context fetches
