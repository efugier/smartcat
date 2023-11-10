# smartcat 🐈‍⬛

Puts a brain behind cat!

WIP cli interface to language models to bring them in the Unix ecosystem

```
Putting a brain behind `cat`. WIP cli interface to language model to bring them in the Unix echosystem 🐈‍⬛

Usage: smartcat [OPTIONS] [PROMPT]

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
          which api to hit [possible values: openai]
  -m, --model <MODEL>
          which model (of the api) to use
  -f, --file <FILE>
          file to read input from
  -h, --help
          Print help
  -V, --version
          Print version
```

## A few examples to get started

```
cat Cargo.toml | smartcat -c "write a short poem about the content of the file"

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
cat my_file.json | smartcat -c "translate to yaml" > my_file.yaml
```

```
cat my_stuff.py | smartcat \
  -c "write a parametrized test suite for the following code using pytest" \
  -s "output only the code, as a standalone file" \
  -b "```" -a "```" > test.py
```

If you find yourself reusing prompts often, you can create a dedicated config entries and it becomes the following:

```
smartcat write_tests -f my_file.py > test.py
```

see example in the configuration section.

## Skipping input

to talk directly to the model

```
smartcat -i "Do you like trains?"

So if you wonder, do I like the trains of steel and might,
My answer lies in how they're kin to code that runs so right.
Frameworks and libraries, like stations, stand so proud
And programmers, conductors, who make the engines loud.
```

## Vim

You can also integrate this with your editor. For instance in Vim

```
:'<,'> | smartcat write_test -r
```

will append at the end of the current selection tests written by the language model for what was selected.

With some remapping you may have the whole thing attached to few keystrokes e.g. `<leader>wt`.

## Helix

In helix, simply press the pipe key to redirect the selection to `smarcat`.

```
pipe: smartcat write_test -r
```

These are only some ideas to get started, go nuts!

# Configuration

- by default lives at `$HOME/.config/smartcat`
- the directory can be set using the `smartcat_CONFIG_PATH` environement variable

Two files are used:

`.api_configs.toml`

```toml
[openai]  # each api has their own config section with api and url
url = "https://api.openai.com/v1/chat/completions"
api_key = "your api key"
```

`prompts.toml`

```toml
[default]  # a prompt is a section
api = "openai"
model = "gpt-4-1106-preview"

[[default.messages]]  # then you can list messages
role = "system"
content = """\
You are a poetic assistant, skilled in explaining complex programming \
concepts with creative flair.\
"""

[[default.messages]]
role = "user"
# the following placeholder string #[<input>] will be replaced by the input
# each message seeks it and replaces it
content = "#[<input>]" 

[write_test]  # a prompt is a section
api = "openai"
model = "gpt-4-1106-preview"

[[write_test.messages]]  # then you can list messages
role = "system"
content = """\
You are a very skilled programmer with an keen eye for detail. You always make sure to write clean \
code and you value clarity particularly highly. \
When asked for code, output only the code to write directly. Don't provide explanation.\
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

