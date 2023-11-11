# smartcat (sc) 🐈‍⬛

Puts a brain behind cat!

WIP cli interface to language models to bring them in the Unix ecosystem

```
Putting a brain behind `cat`. WIP cli interface to language model to bring them in the Unix ecosystem 🐈‍⬛

Usage: sc [OPTIONS] [PROMPT]

Arguments:
  [PROMPT]  which prompt in the config to fetch.
  The config must have at least one named "default" containing which model and api to hit by default [default: default]

Options:
  -c, --command <COMMAND>
          custom prompt to append before the input
  -a, --after-input <AFTER_INPUT>
          suffix to add after the input and the custom prompt
  -s, --system-message <SYSTEM_MESSAGE>
          a system "config" message to send before the first user message
      --api <API>
          which api to hit [possible values: openai, another-api-for-tests]
  -m, --model <MODEL>
          which model (of the api) to use
  -f, --file <FILE>
          skip reading from the input and read this file instead
  -r, --repeat-input
          wether to repeat the input before the output, useful to extend instead of replacing
  -i, --input <INPUT>
          skips reading from input and use that value instead
  -h, --help
          Print help
  -V, --version
          Print version
```

## Installation

```
cargo install smartcat
```

Optional:
```
mv ~/.cargo/bin/smartcat ~/.cargo/bin/sc
```

where `~/.cargo/` is the cargo home, you can find it with `which smarcat` after installing it.

On the first run, the program will ask you to generate some default configuration if it cannot find them. More about that in the [configuration section](#Configuration).

## A few examples to get started

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
  -s "output only the code, as a standalone file \n```\n" \
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

## Integrating with editors

The key for a good integration in editors is a good default prompt and the usage of the `-r` flag to decide whether to replace or extend the selection.

### Vim

You can also integrate this with your editor. For instance in Vim

```
:'<,'> | sc -c "replace the versions with wildcards"
```

```
:'<,'> | sc -c "fix the typos in this text"
```

will replace the current selection with the same text transformed by the language model.

```
:'<,'> | sc -c "implement the traits FromStr and ToString for this struct" -r
```

will append at the end of the current selection the result of the language model.

```
:'<,'> | sc write_test -r
```

...

With some remapping you may have your most reccurrent action attached to few keystrokes e.g. `<leader>wt`!

### Helix

In helix, simply press the pipe key to redirect the selection to `smarcat`.

```
pipe: sc write_test -r
```

These are only some ideas to get started, go nuts!

# Configuration

- by default lives at `$HOME/.config/smartcat`
- the directory can be set using the `SMARTCAT_CONFIG_PATH` environement variable
- use `#[<input>]` as the placeholder for input when writing prompts

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
api = "openai"
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

[empty]
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
content ='''Write tests using pytest for the following code. Parametrized it if appropriate.

#[<input>]
'''
```

see [the config setup file](./src/config.rs) for more details.

## Developping

Some tests rely on environement variables and don't behave well with multi-threading so make sure to test with

```
cargo test -- --test-threads=1
```
