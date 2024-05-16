## Contributing to Smartcat 🐈‍⬛

### State of the project and main directions

Smartcat has reached an acceptable feature set. As a unix CLI tool it should remain focused and minimal but feature requests and suggestions are encouraged.

Codebase quality improvement are very welcome as I hadn't really used rust since 2019 and it shows.

### Codebase map

```python
src/
│   # args parsing logic
├── main.rs
│   # a (manageable) handful of utility functions used in serveral other places
├── utils.rs
│   # logic to customize the template prompt with the args
├── prompt_customization.rs
│   # logic to insert the input into the prompt
├── config
│   │   # function to check config
│   ├── mod.rs
│   │   # config structs for API config definition (url, key...)
│   ├── api.rs
│   │   # config structs for prompt defition (messages, model, temperature...)
│   ├── prompt.rs
│   │   # config structs for voice config (model, url, voice recording command...)
│   └── voice.rs
│   # voice api related code (request, adapters)
├── voice
│   │   # orchestrate the voice recording and request
│   ├── mod.rs
│   │   # start and stop the recording program
│   ├── recording.rs
│   │   # make the request to the api and read the result
│   ├── api_call.rs
│   │   # structs to parse and extract the message from third party answers
│   └── response_schemas.rs
└── text
    │   # make third party requests and read the result
    ├── mod.rs
    │   # make the request to the api and read the result
    ├── api_call.rs
    │   # logic to adapt smartcat prompts to third party ones
    ├── request_schemas.rs
    │   # structs to parse and extract the message from third party answers
    └── response_schemas.rs
```

#### Logic flow

The prompt object is passed through the entire program, enriched with the input (from stdin) and then the third party response. The third party response is then written stdout and the whole conversation (including the input and the response) is then saved as the last prompt for re-use.

**Regular**

```python
main 
# parse the args and get the template prompt / continue with last conversation as prompt
-> prompt_customization::customize_prompt
 ╎# update the templated prompt with the information from the args
<-
-> text::process_input_with_request
 ╎# insert the input in the prompt
 ╎# load the api config
  -> text::api_call::post_prompt_and_get_answer
    ╎# translate the smartcat prompt to api-specific prompt
    ╎# make the request
    ╎# get the message from api-specific response
  <-
 ╎# add response message to the prompt
 ╎# write the response message to stdout
<-
# save the enriched prompt as last conversation
# exit
```

**Voice**

```python
main 
-> prompt_customization::customize_prompt
-> voice::record_voice_and_get_transcript
   -> voice::recording::start_recording
   -> voice::recording::strop_recording
   -> voice::api_call::post_audio_and_get_transcript
<-
-> text::process_input_with_request
  -> text::api_call::post_prompt_and_get_answer
<-
```

### Testing

Some tests rely on environement variables and don't behave well with multi-threading. They are marked with `#[serial]` from the [serial_test](https://docs.rs/serial_test/latest/serial_test/index.html) crate.


### DOING

- Voice intergation

### TODO

- [ ] make it available on homebrew
- [ ] handle streams
- [ ] automagical context fetches (might be out of scope)
- [ ] add RAG capabilities (might be out of scope)
