# smartcat (sc) 🐈‍⬛

Puts a brain behind `cat`! WIP cli interface to bring language models in the Unix ecosystem 🐈‍⬛

- [Installation](#installation)
- [A few examples to get started](#a-few-examples-to-get-started)
  - [Manipulate file and text streams](#manipulate-file-and-text-streams)
  - [Integrating with editors](#integrating-with-editors)
- [Configuration](#configuration)
- [Developping](#developping)

```text
Usage: sc [OPTIONS] [PROMPT]

Arguments:
  [PROMPT]  which prompt in the config to fetch [default: default]

Options:
  -s, --system-message <SYSTEM_MESSAGE>
          system "config"  message to send after the prompt and  before the first user message
  -c, --command <COMMAND>
          custom prompt to append before the input
  -a, --after-input <AFTER_INPUT>
          suffix to add after the input and the custom prompt
  -r, --repeat-input
          whether to repeat the input before the output, useful to extend instead of replacing
      --api <API>
          overrides which api to hit [possible values: openai, another-api-for-tests]
  -m, --model <MODEL>
          overrides which model (of the api) to use
  -f, --file <FILE>
          skip reading from the input and read this file instead
  -i, --input <INPUT>
          skip reading from input and use that value instead
  -h, --help
          Print help
  -V, --version
          Print version
```

Currently only supporting openai and chatgpt but build to work with multiple ones seemlessly if competitors emerge.

You can use it to **accomplish tasks in the CLI** but **also in your editors** (if they are good unix citizens, i.e. work with shell commands and text streams) to complete, refactor, write tests... anything!

The key to make this work seamlessly is a good default prompt that tells the model to behave like a CLI tool an not write any unwanted text like markdown formatting or explanations.

## Installation

With [rust and cargo](https://www.rust-lang.org/tools/install) installed and setup:

```
cargo install smartcat
```

(the binary is named `sc`)

Or download directly the binary compiled for your platform from the [release page](https://github.com/efugier/smartcat/releases).


On the first run, `smartcat` will ask you to generate some default configuration files if it cannot find them.
More about that in the [configuration section](#Configuration).

A `default` prompt is needed for `smartcat` to know which api and model to hit.

## A few examples to get started

Ask anything without leaving you dear terminal

```
sc -i "what's a sed command to remove trailaing whitespaces at the end of all non-markdown files?"
> sed -i '' 's/[ \t]*$//' *.* !(*.md)
```

```
sc -i "shell script to migrate a repository from pipenv to poetry" >> poetry_mirgation.sh
```

use the `-i` so that it doesn't wait for piped input.

### Manipulate file and text streams

```
cat Cargo.toml | sc -c "write a short poem about the content of the file"

A file named package,
Holds the keys of a software's age.
With a name, version, and edition too,
The content speaks of something new.

Dependencies lie within,
With toml, clap, ureq, and serde in,
The stars denote any version will do,
As long as the features are included, too.

A short poem of the file's content,
A glimpse into the software's intent.
With these keys and dependencies,
A program is born, fulfilling needs.
```

```
sc -f Cargo.toml -c "translate the following file in json" | save Cargo.json
```

```
cat my_stuff.py | sc \
  -c "write a parametrized test suite for the following code using pytest" \
  -s "output only the code, as a standalone file with the imports. \n```\n" \
  -a "```" > test.py
```

If you find yourself reusing prompts often, you can create a dedicated config entries and it becomes the following:

```
sc write_tests -f my_file.py > test.py
```

see example in the [configuration section](#Configuration).

Skipping input to talk directly to the model (but mind the default prompt)

```
sc empty -i "Do you like trains?"

So if you wonder, do I like the trains of steel and might,
My answer lies in how they're kin to code that runs so right.
Frameworks and libraries, like stations, stand so proud
And programmers, conductors, who make the engines loud.
```

### Integrating with editors

The key for a good integration in editors is a good default prompt (or set of) combined with the `-c` flag for precising the task at hand.
The `-r` flag can be used to decide whether to replace or extend the selection.

#### Vim

Start by selecting some text, then press `:`. You can then pipe the selection content to `smartcat`.

```
:'<,'>!sc -c "replace the versions with wildcards"
```

```
:'<,'>!sc -c "fix the typos in this text"
```

will **replace** the current selection with the same text transformed by the language model.

```
:'<,'>!sc -c "implement the traits FromStr and ToString for this struct" -r
```

```
:'<,'>!sc write_test -r
```

will **append** at the end of the current selection the result of the language model.

...

With some remapping you may have your most reccurrent action attached to few keystrokes e.g. `<leader>wt`!

#### Helix

Same concept, different shortcut, simply press the pipe key to redirect the selection to `smarcat`.

```
pipe:sc write_test -r
```

These are only some ideas to get started, go nuts!

# Configuration

- by default lives at `$HOME/.config/smartcat`
- the directory can be set using the `SMARTCAT_CONFIG_PATH` environement variable
- use `#[<input>]` as the placeholder for input when writing prompts
- the default model is `gpt-4` but I recommend trying the latest ones and see which one works best for you. I currently use `gpt-4-1106-preview`.

Two files are used:

`.api_configs.toml`

```toml
[openai]  # each api has their own config section with api and url
url = "https://api.openai.com/v1/chat/completions"
api_key = "<your_api_key>"
```

`prompts.toml`

```toml
[default]  # a prompt is a section
api = "openai"  # must refer to an entry in the `.api_configs.toml` file
model = "gpt-4-1106-preview"

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
model = "gpt-4-1106-preview"
messages = []

[write_test]
api = "openai"
model = "gpt-4-1106-preview"

[[write_test.messages]]
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

[[write_test.messages]]
role = "user"
# the following placeholder string #[<input>] will be replaced by the input
# each message seeks it and replaces it
content ='''Write tests using pytest for the following code. Parametrize it if appropriate.

#[<input>]
'''
```

see [the config setup file](./src/config.rs) for more details.

## Developping

Some tests rely on environement variables and don't behave well with multi-threading so make sure to test with

```
cargo test -- --test-threads=1
```

### State of the project

Smartcat has reached an acceptable feature set. The focus is now on upgrading the codebase quality as I hadn't really touched rust since 2019 and it shows.

#### TODO/Ideas:

- make it available on homebrew
- interactive mode to have conversations and make the model iterate on the last answer
