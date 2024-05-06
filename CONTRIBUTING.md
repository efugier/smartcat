## Contributing to Smartcat 🐈‍⬛

### State of the project and main directions

Smartcat has reached an acceptable feature set. As a unix CLI tool it should remain focused and minimal but feature requests and suggestions are encouraged.

Codebase quality improvement are very welcome as I hadn't really used rust since 2019 and it shows.

### Codebase map

```python
src/
│   # args parsing logic
├── main.rs
│   # logic to customize the template prompt with the args
├── prompt_customization.rs
│   # logic to insert the input into the prompt
├── input_processing.rs
│   # smartcat-related config structs
├── config
│   │   # function to check config
│   ├── mod.rs
│   │   # config structs for API definition (url, key...)
│   ├── api.rs
│   │   # config structs for prompt defition (messages, model, temperature...)
│   └── prompt.rs
│   # third-party related code (request, adapters)
└── third_party
    │   # make third party requests
    ├── mod.rs
    │   # logic to adapt smartcat prompts to third party ones
    ├── prompt_adapters.rs
    │   # logic to parse and extract the message from third party answers
    └── response_parsing.rs
```

#### Logic flow:

The prompt object is passed through the entire program, enriched with the input (from stdin) and then the third party response. The third party response is then written stdout and the whole conversation (including the input and the response) is then saved as the last prompt for re-use.

```python
main 
  # parse the args and get the template prompt / continue with last conversation as prompt
  -> prompt_customization::customize_prompt 
     # update the templated prompt with the information from the args
  -> input_processing::process_input_with_request
     # insert the input in the prompt
     # load the api config
     -> third_party::make_api_request
        # translate the smartcat prompt to api-specific prompt
        # make the request
        # get the message from api-specific response
     # add response message to the prompt
     # write the response message to stdout
# save the enriched prompt as last conversation
# exit
```

### Testing

Some tests rely on environement variables and don't behave well with multi-threading. They are marked with `#[serial]` from the [serial_test](https://docs.rs/serial_test/latest/serial_test/index.html) crate.


### TODO

- [ ] make it available on homebrew
- [ ] handle streams
- [ ] automagical context fetches (might be out of scope)
- [ ] add RAG capabilities (might be out of scope)
- [ ] refactor to remove content logic from the `mod.rs` files
