<!-- TOC -->

- [Mezmo Developments](#mezmo-developments)
  - ["Reshape" LogEvent for selected sinks](#reshape-logevent-for-selected-sinks)
    - [Notable Changes](#notable-changes)
    - [Environment Variables](#environment-variables)
  - [Create a Mezmo-specific Reduce transform](#create-a-mezmo-specific-reduce-transform)
    - [Notable Changes](#notable-changes-1)

<!-- /TOC -->
# Mezmo Developments

This document will highlight some of the mezmo-specific changes that have been made to
this repository. The goal is to highlight any usage changes, especially the addition
of any environment variables that will trigger new functionality.

---
## "Reshape" LogEvent for selected sinks
Since Mezmo treats the `message` property as the entire message, the service used to
reshape LogEvents prior to entering a sink. That reshaping moved all the contents of the
`message` property to the root of the event, obscuring any vector-related shaping. This
functionality has now been moved to selected sinks in this project.

* **Date:** 2-6-2023
* **PR**: https://github.com/answerbook/vector/pull/176

### Notable Changes
* Changes were made to `src/codecs/encoding/transformer.rs` to optionally call the reshaping function exposed in `mezmo/mod.rs`. When `.transform()` is called by all sinks, it will reshape if `should_mezmo_reshape` is set.

  * Sinks were selected individually for reshaping, choosing to replace `transformer` with
    `Transformer::new_with_mezmo_reshape(transformer)` which will set `should_mezmo_reshape` to `true`.
  * `mezmo_integration_tests.rs` was created where possible to keep tests separate.

### Environment Variables

* `MEZMO_RESHAPE_MESSAGE = "1"`

If set to `"1"`, the reshaping will happen as long as the LogEvent is an object that contains a `message` property, and the `Transformer` is properly replaced with `Transformer::new_with_mezmo_reshape(transformer)`

---

## Create a Mezmo-specific Reduce transform

* **Date:** 12-21-2022
* **PR**: https://github.com/answerbook/vector/pull/141

### Notable Changes
* Added code `src/transforms/reduce/mezmo_reduce.rs`
* Creates a transform type called `mezmo_reduce`
* Added features:
  * Support for `date_formats` and general date parsing
  * Proper persistence of date data types (string or integer epocs)
  * Operates on the `message` contents instead of root contents

---
