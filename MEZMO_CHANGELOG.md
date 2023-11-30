## [1.25.4](https://github.com/answerbook/vector/compare/v1.25.3...v1.25.4) (2023-11-30)


### Bug Fixes

* **otlp**: get user meta from event and parent log [ce99a12](https://github.com/answerbook/vector/commit/ce99a1295579cfaaff1e7561e6bed52ce7c5d4eb) - Matt March [LOG-18717](https://logdna.atlassian.net/browse/LOG-18717)

## [1.25.3](https://github.com/answerbook/vector/compare/v1.25.2...v1.25.3) (2023-11-30)


### Bug Fixes

* Incorrect usage of enterprise build feature in ConfigBuilder [5164e6c](https://github.com/answerbook/vector/commit/5164e6c4442ac5228d25240c35b5734675971e1e) - Jorge Bay [LOG-18735](https://logdna.atlassian.net/browse/LOG-18735)

## [1.25.2](https://github.com/answerbook/vector/compare/v1.25.1...v1.25.2) (2023-11-28)


### Bug Fixes

* **automated-parsing**: only consider the first valid line_field [b157004](https://github.com/answerbook/vector/commit/b1570043b1b3ff9b58180e616ce59c58b87c0c6d) - Mike Del Tito [LOG-18677](https://logdna.atlassian.net/browse/LOG-18677)
* Http config provider sends reload signal when config changes [da7c8eb](https://github.com/answerbook/vector/commit/da7c8ebb347c02371d03cb321f0e0c06d8117486) - Jorge Bay [LOG-18264](https://logdna.atlassian.net/browse/LOG-18264)


### Code Refactoring

* **automated-parsing**: use grok crate [d1bbd90](https://github.com/answerbook/vector/commit/d1bbd90acc91ec155f60a9efb9a6b35fde90ba67) - Mike Del Tito [LOG-18677](https://logdna.atlassian.net/browse/LOG-18677)


### Miscellaneous

* Merge pull request #349 from answerbook/mdeltito/poc-grok [43110bd](https://github.com/answerbook/vector/commit/43110bdd1b0be2bed4fa4de4342cfddee553e896) - GitHub

## [1.25.1](https://github.com/answerbook/vector/compare/v1.25.0...v1.25.1) (2023-11-28)


### Bug Fixes

* Make sure log clustering data is aggregated in memory [93b0016](https://github.com/answerbook/vector/commit/93b0016062c21f0eed99eeb65522e0670842479f) - Jorge Bay [LOG-18708](https://logdna.atlassian.net/browse/LOG-18708)

# [1.25.0](https://github.com/answerbook/vector/compare/v1.24.3...v1.25.0) (2023-11-23)


### Features

* **otlp**: bring in resource, scope attributes [660f36d](https://github.com/answerbook/vector/commit/660f36db610fe4c20d767b805d0c5bfa73bd3c84) - Matt March [LOG-18506](https://logdna.atlassian.net/browse/LOG-18506)

## [1.24.3](https://github.com/answerbook/vector/compare/v1.24.2...v1.24.3) (2023-11-22)


### Chores

* Bump vrl library to include new functions [bf4c1e9](https://github.com/answerbook/vector/commit/bf4c1e902b775cb5253052931a49b036005cb0e9) - Jorge Bay [LOG-17868](https://logdna.atlassian.net/browse/LOG-17868)

## [1.24.2](https://github.com/answerbook/vector/compare/v1.24.1...v1.24.2) (2023-11-21)


### Bug Fixes

* **tap**: avoid compiling globs frequently during tap [5c7eea1](https://github.com/answerbook/vector/commit/5c7eea16b02fac821381e3dbb2494cc6017b1ee5) - Adam Holmberg [LOG-18645](https://logdna.atlassian.net/browse/LOG-18645)
* **tap**: reduce glob compilation for performance [5927407](https://github.com/answerbook/vector/commit/592740789c380a48dfbb0ebdd28472f3f33befe0) - Adam Holmberg [LOG-18645](https://logdna.atlassian.net/browse/LOG-18645)


### Miscellaneous

* Merge pull request #350 from answerbook/feature/LOG-18645 [0b5f102](https://github.com/answerbook/vector/commit/0b5f10220344de9c5123265f5ccc41a0f3f6148f) - GitHub [LOG-18645](https://logdna.atlassian.net/browse/LOG-18645)

## [1.24.1](https://github.com/answerbook/vector/compare/v1.24.0...v1.24.1) (2023-11-15)


### Bug Fixes

* **automated-parsing**: avoid clobbering annotations.classification [a8ab95a](https://github.com/answerbook/vector/commit/a8ab95af6d6a77f40b0e11c1805c121866c116d2) - Mike Del Tito [LOG-18650](https://logdna.atlassian.net/browse/LOG-18650)


### Miscellaneous

* Merge pull request #348 from answerbook/mdeltito/LOG-18650 [24f2c18](https://github.com/answerbook/vector/commit/24f2c18138c8951d31bce82ca4b08fb4ad3189bc) - GitHub [LOG-18650](https://logdna.atlassian.net/browse/LOG-18650)

# [1.24.0](https://github.com/answerbook/vector/compare/v1.23.0...v1.24.0) (2023-11-15)


### Features

* Adapt profiling to work in the analysis phase. [306eb4c](https://github.com/answerbook/vector/commit/306eb4c05c09bef406fecb0de80f211a6328772b) - Jorge Bay [LOG-18571](https://logdna.atlassian.net/browse/LOG-18571)

# [1.23.0](https://github.com/answerbook/vector/compare/v1.22.0...v1.23.0) (2023-11-10)


### Features

* **automated-parsing**: fgrok classification transform [0d325aa](https://github.com/answerbook/vector/commit/0d325aaf2791bf761b6229ac4b92353a36023420) - Mike Del Tito [LOG-18364](https://logdna.atlassian.net/browse/LOG-18364)


### Miscellaneous

* Merge pull request #345 from answerbook/mdeltito/LOG-18364 [10ada72](https://github.com/answerbook/vector/commit/10ada7200d701e4423b164968464564c489bbf5e) - GitHub [LOG-18364](https://logdna.atlassian.net/browse/LOG-18364)

# [1.22.0](https://github.com/answerbook/vector/compare/v1.21.0...v1.22.0) (2023-11-10)


### Features

* Support storing log clustering info out of band [d43a518](https://github.com/answerbook/vector/commit/d43a5186062dcf4fd6a4b3d67a588dffb49bf7ed) - Jorge Bay [LOG-18410](https://logdna.atlassian.net/browse/LOG-18410)

# [1.21.0](https://github.com/answerbook/vector/compare/v1.20.0...v1.21.0) (2023-10-26)


### Features

* Use stable ids for log clustering [f517538](https://github.com/answerbook/vector/commit/f51753886866bdea3a13a5711e2ebfe6f9a7a0f7) - Jorge Bay [LOG-18409](https://logdna.atlassian.net/browse/LOG-18409)

# [1.20.0](https://github.com/answerbook/vector/compare/v1.19.0...v1.20.0) (2023-10-24)


### Features

* Intercept and track annotated events using the UsageTracker [5577922](https://github.com/answerbook/vector/commit/55779221a709574d6d7720de280017f0c1f0b919) - Jorge Bay [LOG-18411](https://logdna.atlassian.net/browse/LOG-18411)

# [1.19.0](https://github.com/answerbook/vector/compare/v1.18.2...v1.19.0) (2023-10-18)


### Features

* **deploy**: enable configuring partitions [ae165c3](https://github.com/answerbook/vector/commit/ae165c36d8e04c6657f921b7713cf40e6479c770) - Matt March [LOG-18365](https://logdna.atlassian.net/browse/LOG-18365)

## [1.18.2](https://github.com/answerbook/vector/compare/v1.18.1...v1.18.2) (2023-10-12)


### Chores

* **deps**: bump rdkafka from 0.30.0 to 0.31.0 (#17428) [96409c6](https://github.com/answerbook/vector/commit/96409c652649d8133075b3e7ed30065aca3a58e8) - Chris Nixon
* **deps**: Bump rdkafka from 0.31.0 to 0.32.2 (#17664) [2cad998](https://github.com/answerbook/vector/commit/2cad9984b7125950b685bb3ff326faff6ab9a03a) - Chris Nixon
* **deps**: Bump rdkafka from 0.32.2 to 0.33.2 (#17891) [5713394](https://github.com/answerbook/vector/commit/57133949589d94f27082302d6d14eee5c17688c3) - Chris Nixon
* **deps**: Bump rdkafka from 0.33.2 to 0.34.0 (#18393) [bb93658](https://github.com/answerbook/vector/commit/bb93658abd77cb87bdd573f6f233b78cc3732695) - Chris Nixon

## [1.18.1](https://github.com/answerbook/vector/compare/v1.18.0...v1.18.1) (2023-10-06)


### Bug Fixes

* **state_variables**: Polling is killed on topology reload [b8412ca](https://github.com/answerbook/vector/commit/b8412cab16a0f8d3c99916b5ca519f180a9c7f0f) - Darin Spivey [LOG-18392](https://logdna.atlassian.net/browse/LOG-18392)


### Miscellaneous

* Merge pull request #339 from answerbook/darinspivey/LOG-18392 [afbbe26](https://github.com/answerbook/vector/commit/afbbe26811f154a494f19bb53a2b7dfcca3a498a) - GitHub [LOG-18392](https://logdna.atlassian.net/browse/LOG-18392)

# [1.18.0](https://github.com/answerbook/vector/compare/v1.17.4...v1.18.0) (2023-10-05)


### Features

* **enrichment**: Dynamic Pipelines can look up state variables [8fb1abd](https://github.com/answerbook/vector/commit/8fb1abd6fc02041fe372013c7d86b173706c9fdc) - Darin Spivey [LOG-18286](https://logdna.atlassian.net/browse/LOG-18286) [LOG-18333](https://logdna.atlassian.net/browse/LOG-18333)


### Miscellaneous

* Merge pull request #338 from answerbook/darinspivey/LOG-18286 [d964789](https://github.com/answerbook/vector/commit/d964789e02ddf80f7effa54617b7ace72ce056f3) - GitHub [LOG-18286](https://logdna.atlassian.net/browse/LOG-18286)

## [1.17.4](https://github.com/answerbook/vector/compare/v1.17.3...v1.17.4) (2023-10-02)


### Bug Fixes

* Change HPA apiVersion to v2 [a10d84c](https://github.com/answerbook/vector/commit/a10d84ca0565ac403f3a3f329af66884e34b0632) - Michael Penick [LOG-18049](https://logdna.atlassian.net/browse/LOG-18049)

## [1.17.3](https://github.com/answerbook/vector/compare/v1.17.2...v1.17.3) (2023-09-27)


### Chores

* **sinks**: Mezmo sink add header for custom response [173badc](https://github.com/answerbook/vector/commit/173badc8e81f00eac36148ce21238602fcd83285) - Darin Spivey [LOG-18256](https://logdna.atlassian.net/browse/LOG-18256)


### Miscellaneous

* Merge pull request #332 from answerbook/darinspivey/LOG-18256 [9231c3f](https://github.com/answerbook/vector/commit/9231c3ff949fa9e384aa11ae44fb8affd5d416cd) - GitHub [LOG-18256](https://logdna.atlassian.net/browse/LOG-18256)

## [1.17.2](https://github.com/answerbook/vector/compare/v1.17.1...v1.17.2) (2023-09-27)


### Bug Fixes

* **pulse**: Revert align timeouts for fetching tap data [c39dc9d](https://github.com/answerbook/vector/commit/c39dc9d189746f062575b0810d581a41a10325a7) - Jorge Bay [LOG-18263](https://logdna.atlassian.net/browse/LOG-18263)

## [1.17.1](https://github.com/answerbook/vector/compare/v1.17.0...v1.17.1) (2023-09-26)


### Bug Fixes

* **pulse**: Align timeouts for fetching tap data [28c16b9](https://github.com/answerbook/vector/commit/28c16b9716a959b99d3a8369c7172dc9d8cf6199) - Jorge Bay [LOG-18263](https://logdna.atlassian.net/browse/LOG-18263)


### Chores

* **ci**: Disable Splunk integration tests [de434f8](https://github.com/answerbook/vector/commit/de434f82ff8c8d36b9766b7fc46d334da333144c) - Jorge Bay [LOG-17997](https://logdna.atlassian.net/browse/LOG-17997)

# [1.17.0](https://github.com/answerbook/vector/compare/v1.16.0...v1.17.0) (2023-09-22)


### Features

* **decoding**: otlp traces decoding implementation [57ce4ce](https://github.com/answerbook/vector/commit/57ce4ce540f87f74584f5415a86f65e325f3e1ee) - Sergey Opria [LOG-18137](https://logdna.atlassian.net/browse/LOG-18137)


### Miscellaneous

* Merge pull request #331 from answerbook/sopria/LOG-18137 [b9ad513](https://github.com/answerbook/vector/commit/b9ad513a09585faa86eafb626dae4bb9687ec539) - GitHub [LOG-18137](https://logdna.atlassian.net/browse/LOG-18137)

# [1.16.0](https://github.com/answerbook/vector/compare/v1.15.1...v1.16.0) (2023-09-21)


### Features

* **otlp-logs**: add otlp log parsing [a8e76e9](https://github.com/answerbook/vector/commit/a8e76e98d4ea041367dbc91211681c5877172ee9) - Chris Nixon [LOG-17746](https://logdna.atlassian.net/browse/LOG-17746)


### Miscellaneous

* otlp log support [ea16542](https://github.com/answerbook/vector/commit/ea16542252d7e9072de83c2e44a59f25ebb972c0) - Chris Nixon

## [1.15.1](https://github.com/answerbook/vector/compare/v1.15.0...v1.15.1) (2023-09-20)


### Chores

* Adapt the topology builder to match upstream [0af206b](https://github.com/answerbook/vector/commit/0af206b3b0bfd6f47eec653f3e416b007e587885) - Jorge Bay [LOG-18150](https://logdna.atlassian.net/browse/LOG-18150)

# [1.15.0](https://github.com/answerbook/vector/compare/v1.14.0...v1.15.0) (2023-09-14)


### Bug Fixes

* **mezmo_reduce**: Revert including event size [ef907fd](https://github.com/answerbook/vector/commit/ef907fdd3221f1800f606d69f2a2dfbe20e4f1d4) - Michael Penick [LOG-18143](https://logdna.atlassian.net/browse/LOG-18143)


### Features

* handle numeric metrics without value.type [908086f](https://github.com/answerbook/vector/commit/908086f92d5a39c5727ba3e4e95df3e9ce471cec) - Adam Holmberg [LOG-17689](https://logdna.atlassian.net/browse/LOG-17689)
* support converting a metric with 'count' type [fa87791](https://github.com/answerbook/vector/commit/fa87791544157c294c98c438338a11ac69b04e89) - Adam Holmberg [LOG-17689](https://logdna.atlassian.net/browse/LOG-17689)


### Miscellaneous

* Merge pull request #321 from answerbook/holmberg/LOG-17689 [968d422](https://github.com/answerbook/vector/commit/968d422509add351af099760a8611c301736d596) - GitHub [LOG-17689](https://logdna.atlassian.net/browse/LOG-17689)

# [1.14.0](https://github.com/answerbook/vector/compare/v1.13.1...v1.14.0) (2023-09-14)


### Bug Fixes

* **mezmo_reduce**: Account for event size [981200e](https://github.com/answerbook/vector/commit/981200e60b697407b25035add2f004b77cbc325d) - Michael Penick [LOG-18143](https://logdna.atlassian.net/browse/LOG-18143)


### Features

* **prototype**: Use global clustering ID [2dcc6d0](https://github.com/answerbook/vector/commit/2dcc6d0bb081dcb99ece80655fdd0e1f0ecf022c) - Michael Penick [LOG-17981](https://logdna.atlassian.net/browse/LOG-17981)

## [1.13.1](https://github.com/answerbook/vector/compare/v1.13.0...v1.13.1) (2023-09-14)


### Bug Fixes

* Revert "feat: Add liveness check for config provider loading" [b62774c](https://github.com/answerbook/vector/commit/b62774cfb78ae550431f05a78bc964205a99c938) - Michael Penick [LOG-18139](https://logdna.atlassian.net/browse/LOG-18139)


### Chores

* Add (fixed) startup probe [4df53ab](https://github.com/answerbook/vector/commit/4df53abe9c639c34c5e99a74a535b4b2775dfc75) - Michael Penick [LOG-18139](https://logdna.atlassian.net/browse/LOG-18139)
* Improve CI reliability [a4cbf68](https://github.com/answerbook/vector/commit/a4cbf68323450a2b02617a955b19f669840f0090) - Jorge Bay [LOG-17997](https://logdna.atlassian.net/browse/LOG-17997)

# [1.13.0](https://github.com/answerbook/vector/compare/v1.12.1...v1.13.0) (2023-09-13)


### Bug Fixes

* **appsignal sink**: Add TLS config option (#17122) [198068c](https://github.com/answerbook/vector/commit/198068cf55732a3bfe034697d9dc5c9abadb1b63) - GitHub
* **buffers**: correctly handle partial writes in reader seek during initialization (#17099) [a791595](https://github.com/answerbook/vector/commit/a791595b0cfcae36d0c46708a91d5e2813ed38eb) - GitHub
* **config**: recurse through schema refs when determining eligibility for unevaluated properties (#17150) [71d1bf6](https://github.com/answerbook/vector/commit/71d1bf6bae80b4d4518e9ea3f87d0d6ecd000984) - GitHub
* **docker_logs source**: Support tcp schema [e1c0c02](https://github.com/answerbook/vector/commit/e1c0c0275a32ac70a6371ece23f8283abe19880c) - GitHub
* **elasticsearch sink**:  Elasticsearch sink with api_version set to "auto" does not recognize the API version of ES6 as V6 (#17226) (#17227) [9b6ef24](https://github.com/answerbook/vector/commit/9b6ef243cac4abc758e288133fb732b7b504f032) - GitHub
* **gcp_stackdriver_metrics sink**: Call function to regenerate auth token (#17297) [bf7904b](https://github.com/answerbook/vector/commit/bf7904b4ff9dbe354c401b816f43123ba6d48335) - GitHub
* **influxdb_logs**: encode influx line when no tags present (#17029) [c3aa14f](https://github.com/answerbook/vector/commit/c3aa14fd4d2b72a3863b8a8f6590c8ba65cc6c56) - GitHub
* **reduce transform**: Revert flushing on interval change to `expire_metrics_ms` (#17084) [e86b155](https://github.com/answerbook/vector/commit/e86b1556d4c9ac106a7baa61950584198edc68bc) - GitHub
* **releasing**: Fix globbing of release artifacts for GitHub (#17114) [7fe089c](https://github.com/answerbook/vector/commit/7fe089c923c198f84cee567e890f4516d13d281f) - GitHub
* **schemas**: Dont panic with non object field kinds (#17140) [1e43208](https://github.com/answerbook/vector/commit/1e432089f4a3375b2a6dfefb1de3197af5f2313d) - GitHub


### Chores

* (syslog source): add source_ip to some syslog tests (#17235) [29c34c0](https://github.com/answerbook/vector/commit/29c34c073c0dde0e5d9f69c94193ae547538da5d) - GitHub
* add note to DEVELOPING.md re panics (#17277) [03e905e](https://github.com/answerbook/vector/commit/03e905e304d2253dfcd0019105337df23e72d80c) - GitHub
* Add UX note about encoding of log_schema keys (#17266) [dc6e54c](https://github.com/answerbook/vector/commit/dc6e54c18cc3eb7754d3865602b54ae46ec1f67a) - GitHub
* **administration**: add `appsignal` to codeowners (#17127) [7b15d19](https://github.com/answerbook/vector/commit/7b15d191b9b019dfdfea8dd743ff5fa07a19b82f) - GitHub
* **buffer**: tidy up some of the module level docs for `disk_v2` (#17093) [edaa612](https://github.com/answerbook/vector/commit/edaa6124bd7a47cbb551127168b764d496bf73c2) - GitHub
* **ci**: bump docker/metadata-action from 4.3.0 to 4.4.0 (#17170) [854d71e](https://github.com/answerbook/vector/commit/854d71e48883b703b1eb67b538e7ac3b21037fae) - GitHub
* **ci**: Disable `appsignal` integration test until CA issues are resolved (#17109) [f3b5d42](https://github.com/answerbook/vector/commit/f3b5d42cd5d01acf86235e6edc17f5b0d3b837c4) - GitHub
* **ci**: Disable scheduled runs of Baseline Timings workflow (#17281) [4335b0a](https://github.com/answerbook/vector/commit/4335b0a34a44af82bb63739e8e9b3ffc72ecf3f7) - GitHub
* **ci**: Fix event assertions for `aws_ec2_metadata` transform (#17413) [da36fb6](https://github.com/answerbook/vector/commit/da36fb6f9df3724267b30d845e092d2f7628d359) - GitHub
* **ci**: Increase timeout for integration tests (#17326) [e1f125a](https://github.com/answerbook/vector/commit/e1f125a34c91b2344174298a1f508124a0a0b4dd) - GitHub
* **ci**: Increase timeout for integration tests to 30m (#17350) [5d3f619](https://github.com/answerbook/vector/commit/5d3f619ef3295180657529ad5bd44d837cb123b5) - GitHub
* **ci**: re-enable `appsignal` integration test (#17111) [48fc574](https://github.com/answerbook/vector/commit/48fc574e7177bfcc5acf2f9aac85474cb38faef8) - GitHub
* **ci**: Remove ci-sweep tasks (#17415) [5c33f99](https://github.com/answerbook/vector/commit/5c33f999f1e0814c4cc1857cef67415f7bba5cb7) - GitHub
* **ci**: remove unnecessary dep install (#17128) [f56d1ef](https://github.com/answerbook/vector/commit/f56d1ef50d57a5057807b1d122032980bbc70d8d) - GitHub
* **ci**: Try to fix apt retries (#17393) [6b3db04](https://github.com/answerbook/vector/commit/6b3db04f7f7ca700e7696d3430b989efc2a4b3b4) - GitHub
* **ci**: update unsupported ubuntu version runners (#17113) [e7c4815](https://github.com/answerbook/vector/commit/e7c481558373625e04d763ea34451f219f7656d9) - GitHub
* **ci**: use python v3.8 in ubuntu 20.04 runner (#17116) [7a40c81](https://github.com/answerbook/vector/commit/7a40c817151819ba72ed2e31d5860956f693fa8d) - GitHub
* **config**: begin laying out primitives for programmatically querying schema (#17130) [aad8115](https://github.com/answerbook/vector/commit/aad811540ff2a544c8d1fd7410d2c029099845f0) - GitHub
* **config**: emit human-friendly version of enum variant/property names in schema (#17171) [3b38ba8](https://github.com/answerbook/vector/commit/3b38ba82c3727eac93c0d0a992f248b72435dac6) - GitHub
* **config**: improve config schema output with more precise `unevaluatedProperties` + schema ref flattening (#17026) [2d72f82](https://github.com/answerbook/vector/commit/2d72f82b22054a3a7c392061010f94eec15c66be) - GitHub
* **deps**: Add 3rd party license file and CI checks (#17344) [7350e1a](https://github.com/answerbook/vector/commit/7350e1a11805db510814d4fc357e84d0e8d2cf25) - GitHub
* **deps**: bump anyhow from 1.0.70 to 1.0.71 (#17300) [6a5af3b](https://github.com/answerbook/vector/commit/6a5af3b862b0ffdcb509bd8a49641e41294b77b8) - GitHub
* **deps**: bump assert_cmd from 2.0.10 to 2.0.11 (#17290) [c4784fd](https://github.com/answerbook/vector/commit/c4784fd6e62d6cec76ced412512d909df304d005) - GitHub
* **deps**: bump async-compression from 0.3.15 to 0.4.0 (#17365) [b9aac47](https://github.com/answerbook/vector/commit/b9aac475025905943c80dd710f833e2e445c9093) - GitHub
* **deps**: bump async-graphql from 5.0.7 to 5.0.8 (#17357) [05a4f17](https://github.com/answerbook/vector/commit/05a4f17c555c1d2bd25acd7f3173940d98224b53) - GitHub
* **deps**: bump async-graphql-warp from 5.0.7 to 5.0.8 (#17367) [693584e](https://github.com/answerbook/vector/commit/693584eb5002fc0c00586afa1c058bb8cfd0d58e) - GitHub
* **deps**: bump async-stream from 0.3.4 to 0.3.5 (#17076) [c29c817](https://github.com/answerbook/vector/commit/c29c8171bdcea02f991ef9bdc3cbd3ea0b8adedb) - GitHub
* **deps**: bump aws-sigv4 from 0.55.0 to 0.55.1 (#17138) [dbb3f25](https://github.com/answerbook/vector/commit/dbb3f251ce952bcbe47e996d72a00972b12e1095) - GitHub
* **deps**: bump axum from 0.6.12 to 0.6.18 (#17257) [41ac76e](https://github.com/answerbook/vector/commit/41ac76ed03bfc7c08e2f8262eee66c7bae01d5af) - GitHub
* **deps**: bump cached from 0.42.0 to 0.43.0 (#17118) [f90b3b3](https://github.com/answerbook/vector/commit/f90b3b305f23bcb9e4c03d7199a6ad3d4a27045b) - GitHub
* **deps**: bump chrono-tz from 0.8.1 to 0.8.2 (#17088) [623b838](https://github.com/answerbook/vector/commit/623b838b2193e019173ad5d223fb217bbf5d94bd) - GitHub
* **deps**: bump clap_complete from 4.2.0 to 4.2.1 (#17229) [d286d16](https://github.com/answerbook/vector/commit/d286d16dcccca67ea2c1bd994f5440cfca303732) - GitHub
* **deps**: bump clap_complete from 4.2.1 to 4.2.2 (#17359) [565668e](https://github.com/answerbook/vector/commit/565668ea6598992ba47a039e872a18b2ffd19844) - GitHub
* **deps**: bump clap_complete from 4.2.2 to 4.2.3 (#17383) [111cd07](https://github.com/answerbook/vector/commit/111cd07702befce55242c3940c59f05e374d52cf) - GitHub
* **deps**: bump console-subscriber from 0.1.8 to 0.1.9 (#17358) [97b862c](https://github.com/answerbook/vector/commit/97b862c4db77a0192da3b505accf43dcba1c8d59) - GitHub
* **deps**: bump directories from 5.0.0 to 5.0.1 (#17271) [be69f5f](https://github.com/answerbook/vector/commit/be69f5f361ce4621c01f522c7270c5f97b2b7069) - GitHub
* **deps**: bump dunce from 1.0.3 to 1.0.4 (#17244) [cfc387d](https://github.com/answerbook/vector/commit/cfc387d8c4595bfd031cd28d88ac2483200cb53e) - GitHub
* **deps**: bump enumflags2 from 0.7.5 to 0.7.6 (#17079) [cbc17be](https://github.com/answerbook/vector/commit/cbc17be42af382dc200d8f1516be29f231485026) - GitHub
* **deps**: bump enumflags2 from 0.7.6 to 0.7.7 (#17206) [c80c5eb](https://github.com/answerbook/vector/commit/c80c5eb22c1f238903d5c291d944a2b8db7b73b9) - GitHub
* **deps**: bump flate2 from 1.0.25 to 1.0.26 (#17320) [ef13370](https://github.com/answerbook/vector/commit/ef1337024677d4c6ff25cf9cb571cbada39fbe55) - GitHub
* **deps**: bump getrandom from 0.2.8 to 0.2.9 (#17101) [d53240b](https://github.com/answerbook/vector/commit/d53240b53a789edec8bd6700953dccbe660c7a65) - GitHub
* **deps**: bump h2 from 0.3.18 to 0.3.19 (#17388) [6088abd](https://github.com/answerbook/vector/commit/6088abdf6b956940fee4ee827eefb9dce3e84a43) - GitHub
* **deps**: bump hashlink from 0.8.1 to 0.8.2 (#17419) [01b3cd7](https://github.com/answerbook/vector/commit/01b3cd7698dd9a7bf5e2fce909d6e7ef1ffa1313) - GitHub
* **deps**: bump hyper from 0.14.25 to 0.14.26 (#17347) [c43dcfd](https://github.com/answerbook/vector/commit/c43dcfdba4781b81f6418e96b286f37323c7fb26) - GitHub
* **deps**: bump inventory from 0.3.5 to 0.3.6 (#17401) [5b5ad16](https://github.com/answerbook/vector/commit/5b5ad1682dc827e17610eb086d68f4f56e17138d) - GitHub
* **deps**: bump libc from 0.2.140 to 0.2.141 (#17104) [dd9608a](https://github.com/answerbook/vector/commit/dd9608a40da7758ab06f1a298093130abfc72418) - GitHub
* **deps**: bump libc from 0.2.141 to 0.2.142 (#17273) [bc618a2](https://github.com/answerbook/vector/commit/bc618a25e4c501857a0ac3747c4c7735a6192791) - GitHub
* **deps**: bump libc from 0.2.142 to 0.2.143 (#17338) [6afe206](https://github.com/answerbook/vector/commit/6afe206bd595d7933c518342a1602fa15668c0c9) - GitHub
* **deps**: bump libc from 0.2.143 to 0.2.144 (#17346) [99b8dc1](https://github.com/answerbook/vector/commit/99b8dc13bcff379062ac276119e650055e08d0fc) - GitHub
* **deps**: bump memmap2 from 0.5.10 to 0.6.0 (#17355) [dae0c6a](https://github.com/answerbook/vector/commit/dae0c6ad6882bf0bdfa75bde439e3e0f9f4a9dea) - GitHub
* **deps**: bump memmap2 from 0.6.0 to 0.6.1 (#17364) [58ba741](https://github.com/answerbook/vector/commit/58ba7411967af541199042f76590e306e4c8c41f) - GitHub
* **deps**: bump metrics, metrics-tracing-context, metrics-util (#17336) [9a723e3](https://github.com/answerbook/vector/commit/9a723e33cc161b680140c4ef230fedf071e68031) - GitHub
* **deps**: bump mlua from 0.8.8 to 0.8.9 (#17423) [57f8bd4](https://github.com/answerbook/vector/commit/57f8bd4ea2cfdf305dab9875f49e3d5c348c2529) - GitHub
* **deps**: bump mock_instant from 0.2.1 to 0.3.0 (#17210) [40c9afc](https://github.com/answerbook/vector/commit/40c9afc584be350117ada03216cbdf43cbe8775d) - GitHub
* **deps**: bump mongodb from 2.4.0 to 2.5.0 (#17337) [64f4f69](https://github.com/answerbook/vector/commit/64f4f697ecaf8c67096d6ceb5a33e42042e57cdc) - GitHub
* **deps**: bump nkeys from 0.2.0 to 0.3.0 (#17421) [3320eda](https://github.com/answerbook/vector/commit/3320eda52e5144eb8c0214481705a97edc197e81) - GitHub
* **deps**: bump notify from 5.1.0 to 6.0.0 (#17422) [58603b9](https://github.com/answerbook/vector/commit/58603b90ad595df96b6239c42c2dd9e4dce46475) - GitHub
* **deps**: bump num_enum from 0.5.11 to 0.6.0 (#17106) [42f298b](https://github.com/answerbook/vector/commit/42f298b3721098aca7754b1759cf6abd84a1e6fc) - GitHub
* **deps**: bump num_enum from 0.6.0 to 0.6.1 (#17272) [f696e7b](https://github.com/answerbook/vector/commit/f696e7bde782eac78d4692ad5d0de81a7e99c400) - GitHub
* **deps**: bump opendal from 0.30.5 to 0.31.0 (#17119) [8762563](https://github.com/answerbook/vector/commit/8762563a3b19d5b65df3172a5f7bdcd670102eee) - GitHub
* **deps**: bump opendal from 0.31.0 to 0.33.2 (#17286) [3d41931](https://github.com/answerbook/vector/commit/3d419315987671c1d3867e357d921f266c549c71) - GitHub
* **deps**: bump opendal from 0.33.2 to 0.34.0 (#17354) [ae602da](https://github.com/answerbook/vector/commit/ae602da29daad0c1c0081cac0bc27440d28440ad) - GitHub
* **deps**: bump openssl from 0.10.48 to 0.10.50 (#17087) [9a56ed8](https://github.com/answerbook/vector/commit/9a56ed8226a764fa09dcfe9f4e8d968646555bf9) - GitHub
* **deps**: bump openssl from 0.10.50 to 0.10.52 (#17299) [0ecceb3](https://github.com/answerbook/vector/commit/0ecceb3ba95312ed2a22b7f4350547d875184be9) - GitHub
* **deps**: bump pin-project from 1.0.12 to 1.1.0 (#17385) [e8d3002](https://github.com/answerbook/vector/commit/e8d3002d4bcb226ab79ed8b3212d1a123833c535) - GitHub
* **deps**: bump prettydiff from 0.6.2 to 0.6.4 (#17089) [e090610](https://github.com/answerbook/vector/commit/e0906105bc0c6ed297ed96ab8c545535c4fa83b2) - GitHub
* **deps**: bump prettydiff from 0.6.2 to 0.6.4 (#17315) [a1ec68d](https://github.com/answerbook/vector/commit/a1ec68d302757a7fae1082cc90c27ce8aad2c6ea) - GitHub
* **deps**: bump proc-macro2 from 1.0.55 to 1.0.56 (#17103) [6f74523](https://github.com/answerbook/vector/commit/6f745234ed3c7d22cd446769fcac86549c105416) - GitHub
* **deps**: bump proc-macro2 from 1.0.56 to 1.0.57 (#17400) [a6e1ae7](https://github.com/answerbook/vector/commit/a6e1ae737e6ad17f9d3deecc6c887e41a1d86099) - GitHub
* **deps**: bump prost-build from 0.11.8 to 0.11.9 (#17260) [a88aba4](https://github.com/answerbook/vector/commit/a88aba49a357e547a43a7d985a9ebd8d5c58f9a2) - GitHub
* **deps**: bump quote from 1.0.26 to 1.0.27 (#17348) [f81ff18](https://github.com/answerbook/vector/commit/f81ff1837adcf1cc4419bc936fe539e7dd882dbb) - GitHub
* **deps**: bump rdkafka from 0.29.0 to 0.30.0 (#17387) [9703188](https://github.com/answerbook/vector/commit/970318839d5722a3ab40e8276a0ee6982fa798b3) - GitHub
* **deps**: bump regex from 1.7.3 to 1.8.1 (#17222) [410aa3c](https://github.com/answerbook/vector/commit/410aa3cab29b91b59abadadceccffe14e022f06e) - GitHub
* **deps**: bump reqwest from 0.11.16 to 0.11.17 (#17316) [09176ec](https://github.com/answerbook/vector/commit/09176ec3e98febbca0ee54985248c5ecd0fdb39d) - GitHub
* **deps**: bump security-framework from 2.8.2 to 2.9.0 (#17386) [1287168](https://github.com/answerbook/vector/commit/12871685d3f6261ee0d50171584426aba96264ee) - GitHub
* **deps**: bump serde from 1.0.159 to 1.0.160 (#17270) [036ad4a](https://github.com/answerbook/vector/commit/036ad4ab17ddadfa1e24164ffbfa28b458e4c74e) - GitHub
* **deps**: bump serde from 1.0.160 to 1.0.162 (#17317) [79e97a2](https://github.com/answerbook/vector/commit/79e97a2bc96f424335c62fe3519c8e1501f65bcf) - GitHub
* **deps**: bump serde from 1.0.162 to 1.0.163 (#17366) [9852c17](https://github.com/answerbook/vector/commit/9852c1770bd2dceecc9b30ffa72b1f95c0dfd719) - GitHub
* **deps**: bump serde_json from 1.0.95 to 1.0.96 (#17258) [7570bb3](https://github.com/answerbook/vector/commit/7570bb31e2f471e3ff8bc818c24e9bde3090818c) - GitHub
* **deps**: bump serde_with from 2.3.1 to 2.3.2 (#17090) [adbf4d5](https://github.com/answerbook/vector/commit/adbf4d54e5b11a562b1323d3dcbc2587c855b093) - GitHub
* **deps**: bump serde_yaml from 0.9.19 to 0.9.21 (#17120) [d6f2625](https://github.com/answerbook/vector/commit/d6f26254ac1fecbf8c88c275ebd93e107386e740) - GitHub
* **deps**: bump socket2 from 0.4.7 to 0.5.2 (#17121) [db39d83](https://github.com/answerbook/vector/commit/db39d837e5083fe2788ea729dd20abf20234cc72) - GitHub
* **deps**: bump socket2 from 0.5.2 to 0.5.3 (#17384) [ac51b8a](https://github.com/answerbook/vector/commit/ac51b8a35d83e5c24ac0686eb57f4f4bb347773b) - GitHub
* **deps**: bump syslog from 6.0.1 to 6.1.0 (#17301) [61e6154](https://github.com/answerbook/vector/commit/61e6154fd5f4712dae0b60661ff34ae586ce8ac4) - GitHub
* **deps**: bump tokio from 1.27.0 to 1.28.0 (#17231) [8067f84](https://github.com/answerbook/vector/commit/8067f84ae38ad613af0063179e19e7bbf5448ca4) - GitHub
* **deps**: bump tokio from 1.28.0 to 1.28.1 (#17368) [ae6a51b](https://github.com/answerbook/vector/commit/ae6a51b52d2a0f93b3cf16638fd10a52e33294c9) - GitHub
* **deps**: bump tokio-stream from 0.1.12 to 0.1.14 (#17339) [80c8247](https://github.com/answerbook/vector/commit/80c82470b309901d83de03529312fc3e733d8e3e) - GitHub
* **deps**: bump tokio-tungstenite from 0.18.0 to 0.19.0 (#17404) [ae1dd6e](https://github.com/answerbook/vector/commit/ae1dd6e4a67d046037154dab425e4fe6bfd11087) - GitHub
* **deps**: bump tonic from 0.8.3 to 0.9.1 (#17077) [eafba69](https://github.com/answerbook/vector/commit/eafba69a355c8b7ae099134392c6ebd7cab6dcce) - GitHub
* **deps**: bump tonic from 0.9.1 to 0.9.2 (#17221) [aa9cbd0](https://github.com/answerbook/vector/commit/aa9cbd078ff7f8ac35dc5555533b7394764b86ca) - GitHub
* **deps**: bump tonic-build from 0.8.4 to 0.9.2 (#17274) [e0a07c6](https://github.com/answerbook/vector/commit/e0a07c6dfe3ecadb8f88fcd343d302d5c142761d) - GitHub
* **deps**: bump tracing-subscriber from 0.3.16 to 0.3.17 (#17268) [1406c08](https://github.com/answerbook/vector/commit/1406c087db2f377eff65065c5f2fbcb295d4d138) - GitHub
* **deps**: bump typetag from 0.2.7 to 0.2.8 (#17302) [c8e0e5f](https://github.com/answerbook/vector/commit/c8e0e5febbffece0a9a2fd7776767fd93a04e0db) - GitHub
* **deps**: bump uuid from 1.3.0 to 1.3.1 (#17091) [9cc2f1d](https://github.com/answerbook/vector/commit/9cc2f1de1cce6c43e335ec1815363f510e111fbd) - GitHub
* **deps**: bump uuid from 1.3.0 to 1.3.2 (#17256) [bc6f7fd](https://github.com/answerbook/vector/commit/bc6f7fd5109242cc53d7f388ff264662b6a6c223) - GitHub
* **deps**: bump uuid from 1.3.2 to 1.3.3 (#17403) [3a3fe63](https://github.com/answerbook/vector/commit/3a3fe6337d940af3d2667c7775b2fa2e657648fc) - GitHub
* **deps**: bump warp from 0.3.4 to 0.3.5 (#17288) [d8c1f12](https://github.com/answerbook/vector/commit/d8c1f12f4a65129cad225632c9a43b13ac354a7a) - GitHub
* **deps**: bump wasm-bindgen from 0.2.84 to 0.2.85 (#17356) [ea24b4d](https://github.com/answerbook/vector/commit/ea24b4d1695e2484ad54f7e03edb6fcd1b8d0971) - GitHub
* **deps**: bump wasm-bindgen from 0.2.85 to 0.2.86 (#17402) [0518176](https://github.com/answerbook/vector/commit/05181765a5d2c7610adfcf6cd1e44610eb7ed79e) - GitHub
* **deps**: bump wiremock from 0.5.17 to 0.5.18 (#17092) [51312aa](https://github.com/answerbook/vector/commit/51312aaa919cbe4e0d25dcfc202a6e9f618389a3) - GitHub
* **deps**: Fix up missing license (#17379) [a2b8903](https://github.com/answerbook/vector/commit/a2b890352bc42e9a9a30163e26a2f181f08c4a3b) - GitHub
* **deps**: Reset dependencies bumped by a61dea1 (#17100) [887d6d7](https://github.com/answerbook/vector/commit/887d6d7971c86e17448054484e7956b8fd393be7) - GitHub
* **deps**: true up cargo.lock (#17149) [10fce65](https://github.com/answerbook/vector/commit/10fce656f624344facf662c7a54282dc46d63303) - GitHub
* **deps**: Update h2 (#17189) [a2882f3](https://github.com/answerbook/vector/commit/a2882f384e24c13efc2dcf55885f609470e7e9e4) - GitHub
* **deps**: Upgrade cue to 0.5.0 (#17204) [d396320](https://github.com/answerbook/vector/commit/d396320162e068d82f8f7d4e47bc8984503750e7) - GitHub
* **deps**: Upgrade Debian to bullseye for distroless image (#17160) [c304a8c](https://github.com/answerbook/vector/commit/c304a8c9b554a18dc39eadcd4d06f81d0d3baed1) - GitHub
* **deps**: Upgrade rust to 1.69.0 (#17194) [ef15696](https://github.com/answerbook/vector/commit/ef15696292c80b80932e20093e833792d9b2aa71) - GitHub
* **dev**: Add note about generating licenses to CONTRIBUTING.md (#17410) [539f379](https://github.com/answerbook/vector/commit/539f379911f735656eaff3aadd4f6aeeb4b681d5) - GitHub
* **dev**: ignore `.helix` dir (#17203) [32a935b](https://github.com/answerbook/vector/commit/32a935b4d74bd38ba96c717291430f03fa80f4c4) - GitHub
* **dev**: Install the correct `mold` based on CPU architecture (#17248) [4b80c71](https://github.com/answerbook/vector/commit/4b80c714b68bb9acc2869c48b71848d11954c6aa) - GitHub
* **dev**: remove editors from gitignore (#17267) [61c0d76](https://github.com/answerbook/vector/commit/61c0d764af78826c8d01c5295924bf0a31c810e2) - GitHub
* **docs**: Add Enterprise link and update Support link (#17408) [5184d50](https://github.com/answerbook/vector/commit/5184d50f115426306a236402b9c76b0e6aa12fe6) - GitHub
* **docs**: Add missing 0.28.2 version [38607cd](https://github.com/answerbook/vector/commit/38607cdc92ac4a9aa40d3da20cbccd8d85eb89b1) - Jesse Szwedko
* **docs**: Clarify `key_field` for `sample` and `throttle` transforms (#17372) [d1e5588](https://github.com/answerbook/vector/commit/d1e558800a570556372949fd332097c3e138a2e8) - GitHub
* **docs**: Document event type conditions (#17311) [a9c8dc8](https://github.com/answerbook/vector/commit/a9c8dc88ce7c35b75ab3d1bf903aca0a6feaee53) - GitHub
* **docs**: make doc style edits (#17155) [65a8856](https://github.com/answerbook/vector/commit/65a8856ab08296bf6da22f7dbf3b9a6da64aff6a) - GitHub
* **docs**: Remove trailing, unmatched quote (#17163) [3c92556](https://github.com/answerbook/vector/commit/3c9255658c994a002b024db89c9cc32dd718de9c) - GitHub
* **docs**: Remove unneeded `yaml` dependency from website (#17215) [752d424](https://github.com/answerbook/vector/commit/752d4245c7f4cfbb4513183aeada24ce8a0e4891) - GitHub
* **docs**: Update component statuses 2023Q2 (#17362) [22cda94](https://github.com/answerbook/vector/commit/22cda94d3b8fa555533b51f3ee6de39932b04775) - GitHub
* **docs**: update the `v0.28.0` upgrade guide with note about `datadog_logs` sink `hostname` key (#17156) [c169131](https://github.com/answerbook/vector/commit/c1691313e34fc77af5c37abdefa1322ee20e3398) - GitHub
* **external docs**: correctly mark some sinks as stateful (#17085) [64d560d](https://github.com/answerbook/vector/commit/64d560d7737e553190d473dbbb07ae001cf169b3) - GitHub
* **loki sink**: warn on label expansions and collisions (#17052) [f06692b](https://github.com/answerbook/vector/commit/f06692b27ac480eb258faab14adce1f7b500f030) - GitHub
* **pulsar**: pulsar-rs bump to v5.1.1 (#17159) [68b54a9](https://github.com/answerbook/vector/commit/68b54a9bc0ae07d916ec48e997a03f7681e54ccc) - GitHub
* Re-add transform definitions (#17152) [9031d0f](https://github.com/answerbook/vector/commit/9031d0faa2811187874364e1b5a3305c9ed0c0da) - GitHub
* Regen docs for sample and throttle (#17390) [6c57ca0](https://github.com/answerbook/vector/commit/6c57ca07aee4402582b7b7c9c37324f49c14bf65) - GitHub
* **releasing**: Add known issues fixed by 0.29.1 (#17218) [40d543a](https://github.com/answerbook/vector/commit/40d543a6a4cfc70a870080df6e543257b4004198) - GitHub
* **releasing**: Bump Vector version to 0.30.0 (#17134) [3834612](https://github.com/answerbook/vector/commit/3834612cb052edcae99f22aecbf07fdad32f816c) - GitHub
* **releasing**: Fix homebrew release script (#17131) [cfbf233](https://github.com/answerbook/vector/commit/cfbf23367a09486075313a0e91b2d1f3c909a313) - Jesse Szwedko
* **releasing**: Fix release channels (#17133) [58b44e8](https://github.com/answerbook/vector/commit/58b44e8e98ebdb799e1080ce3d8d0caa8bc21c1c) - Jesse Szwedko
* **releasing**: Prepare v0.28.2 release [a61dea1](https://github.com/answerbook/vector/commit/a61dea12b34b1d72744e7662ea8706f9ec328251) - Jesse Szwedko
* **releasing**: Prepare v0.29.0 release [4bf6805](https://github.com/answerbook/vector/commit/4bf68057dc6fbd8ac5560be9e391c8b2bba2d92f) - Jesse Szwedko
* **releasing**: Prepare v0.30.0 release [38c3f0b](https://github.com/answerbook/vector/commit/38c3f0be7b7d72ffa7d64976d8ce1d0ddb52f692) - Jesse Szwedko
* **releasing**: Regenerate Kubernetes manifests for 0.21.2 (#17108) [fd13d64](https://github.com/answerbook/vector/commit/fd13d64c7b911f7fa4cb901640dbe6b1042018cc) - GitHub
* **releasing**: Regenerate manifests for 0.21.1 chart (#17187) [1f0de6b](https://github.com/answerbook/vector/commit/1f0de6b5b90734b99b2c44ea500767f2c013e25c) - GitHub
* **releasing**: Regenerate manifests for 0.22.0 chart (#17135) [e7ea0a8](https://github.com/answerbook/vector/commit/e7ea0a82132d7572aad66c6d0b1297777d1196c6) - GitHub
* **releasing**: update patch release template with extra step details [27c3526](https://github.com/answerbook/vector/commit/27c3526e98679c208f3d304e64def51efabdcd76) - GitHub
* Remove skaffold from project (#17145) [d245927](https://github.com/answerbook/vector/commit/d245927f570bca42082f9495844ca4eddef715f2) - GitHub
* remove transform type coercion (#17411) [b6c7e0a](https://github.com/answerbook/vector/commit/b6c7e0ae43222cd173e3d3bae7a62c3dcc985639) - GitHub
* Revert transform definitions (#17146) [05a3f44](https://github.com/answerbook/vector/commit/05a3f447d9f492fe36cf4948931adecab01b0136) - GitHub
* **socket source**: Remove deprecated `max_length` setting from `tcp` and `unix` modes. (#17162) [9ecfc8c](https://github.com/answerbook/vector/commit/9ecfc8c8159d4093a28de270885e880628a90127) - GitHub
* **syslog source**: remove the remove of source_ip (#17184) [5dff0ed](https://github.com/answerbook/vector/commit/5dff0ed37a89e8cfc9db3ca499454dfe8198eadf) - GitHub
* **topology**: Transform outputs hash table of OutputId -> Definition (#17059) [1bdb24d](https://github.com/answerbook/vector/commit/1bdb24d04329aabb7212942b08f503e910ed60ff) - GitHub
* Upgrade `VRL` to `0.3.0` (#17325) [4911d36](https://github.com/answerbook/vector/commit/4911d3600a3fcce81f70fd8cb427b8389aca0bfb) - GitHub


### Features

* adds 'metric_name' field to internal logs for the tag_cardinality_limit component (#17295) [4317340](https://github.com/answerbook/vector/commit/43173403e7f01d169a9b10a53b0e462e77c9c0f0) - GitHub
* **codecs**: Add full codec support to AWS S3 source/sink (#17098) [d648c86](https://github.com/answerbook/vector/commit/d648c86721a689f2e4add0da46c6c9b011e438d6) - GitHub
* **kubernetes_logs**: use kube-apiserver cache for list requests (#17095) [e46fae7](https://github.com/answerbook/vector/commit/e46fae798120f7d3ce762382dcf9cfd3b79e4a55) - GitHub
* Merge with upstream v0.30.0 [6b93177](https://github.com/answerbook/vector/commit/6b93177cbbd62dc7422f9fb64738e4768e728e29) - GitHub [LOG-17997](https://logdna.atlassian.net/browse/LOG-17997)
* **new sink**: Initial `datadog_events` sink (#7678) [fef3810](https://github.com/answerbook/vector/commit/fef3810d3f4513466e482eef7c0b2178187098a0) - Jesse Szwedko
* Update VRL library [6ace1e6](https://github.com/answerbook/vector/commit/6ace1e6fd042eb9551ba39addf87134682c7e008) - Jorge Bay


### Miscellaneous

* Merge branch 'master' [d4b35bb](https://github.com/answerbook/vector/commit/d4b35bb0212f4d26b8cec40de36c9bec56dfaf07) - Jorge Bay
* Merge tag 'v0.30.0' into update-upstream [ee2f300](https://github.com/answerbook/vector/commit/ee2f30081f29fd10057eeff181f388ac2a473555) - Jorge Bay
* Merge commit '9031d0faa2811187874364e1b5a3305c9ed0c0da' into update-upstream [c12faae](https://github.com/answerbook/vector/commit/c12faaee526239416f6b200fa3df5e096bd4e110) - Jorge Bay
* Improve tokio::select behavior for kafka source and finalizers (#17380) [d4df21c](https://github.com/answerbook/vector/commit/d4df21ccef91d675b82e411414679f56cacc5c4e) - GitHub
* Prepare v0.29.1 release [21beed7](https://github.com/answerbook/vector/commit/21beed73290a6d857fdfb0c447972e2ab614417d) - Kyle Criddle
* Add a quickfix to handle special capitalization cases (#17141) [ba63e21](https://github.com/answerbook/vector/commit/ba63e2148afeb3824fc413d63ed165c3c5068eee) - GitHub
* Adjust doc comment locations (#17154) [730c938](https://github.com/answerbook/vector/commit/730c9386f66b6348c64a268ef37e752343d8fb9a) - GitHub
* **amqp sink**: Support AMQ Properties (content-type) in AMQP sink (#17174) [c10d30b](https://github.com/answerbook/vector/commit/c10d30bd35494ea336d90d0abf9977349c38d154) - GitHub
* **aws provider**: Let `region` be configured for default authentication (#17414) [c81ad30](https://github.com/answerbook/vector/commit/c81ad30c3f6627a70586703e4e5e8db7625aeef7) - GitHub
* **core**: add unit test (ignored) for support for encoding special chars in `ProxyConfig` (#17148) [5247972](https://github.com/answerbook/vector/commit/5247972ed8ae85dc384571c2bcc473aa9cb8e661) - GitHub
* **loki sink**: Fix formatting in labels example (#17396) [f3734e8](https://github.com/answerbook/vector/commit/f3734e81cb6409e496e771c0f75f18101b5e9605) - GitHub
* **observability**: Log underlying error for unhandled HTTP errors (#17327) [bf8376c](https://github.com/answerbook/vector/commit/bf8376c3030e6d6df61ca245f2d8be87443bf075) - GitHub
* **observability**: Update internal log rate limiting messages (#17394) [1951535](https://github.com/answerbook/vector/commit/1951535eefe7e0812952d3037b40216106350e95) - GitHub
* **pulsar sink**: Refactor to use StreamSink (#14345) [1e97a2f](https://github.com/answerbook/vector/commit/1e97a2fc5c5cbdee8b27aa34ca14dde67eac2166) - GitHub
* **topology**: Add source id to metadata (#17369) [c683999](https://github.com/answerbook/vector/commit/c6839995e28fd17aefbe440f092046e660d2fd70) - GitHub
* **vdev**: Load compose files and inject network block (#17025) [5d88655](https://github.com/answerbook/vector/commit/5d886550784e1fe49ba5d670f81161c5b8614abc) - GitHub

## [1.12.1](https://github.com/answerbook/vector/compare/v1.12.0...v1.12.1) (2023-09-11)


### Chores

* Remove startup probe [ec40707](https://github.com/answerbook/vector/commit/ec40707de3a11d61cd8c9884fb7f71194c657b04) - Jorge Bay [LOG-17724](https://logdna.atlassian.net/browse/LOG-17724)

# [1.12.0](https://github.com/answerbook/vector/compare/v1.11.1...v1.12.0) (2023-09-08)


### Features

* **mezmo_config**: VRL validation for remap tranforms [6a5dbfb](https://github.com/answerbook/vector/commit/6a5dbfbf7f06df3299bce11c7ddb5614d59db3a4) - Michael Penick [LOG-17690](https://logdna.atlassian.net/browse/LOG-17690)

## [1.11.1](https://github.com/answerbook/vector/compare/v1.11.0...v1.11.1) (2023-08-30)


### Code Refactoring

* log out underlying error [22086b3](https://github.com/answerbook/vector/commit/22086b3d906070ea4995188e2add8c9d7c756b19) - Dan Hable [LOG-18027](https://logdna.atlassian.net/browse/LOG-18027)

# [1.11.0](https://github.com/answerbook/vector/compare/v1.10.1...v1.11.0) (2023-08-30)


### Features

* **prototype**: Add a log clustering transform [04f8214](https://github.com/answerbook/vector/commit/04f821400bcefcf3af409f21f6d07dde2515517a) - Michael Penick [LOG-17981](https://logdna.atlassian.net/browse/LOG-17981)

## [1.10.1](https://github.com/answerbook/vector/compare/v1.10.0...v1.10.1) (2023-08-30)


### Bug Fixes

* **provider**: update v1 route aliases [394e72b](https://github.com/answerbook/vector/commit/394e72b7f5ada64b23e28c9d236820eda0edb4c5) - Mike Del Tito [LOG-18011](https://logdna.atlassian.net/browse/LOG-18011)


### Miscellaneous

* Merge pull request #318 from answerbook/mdeltito/LOG-18011 [089ad48](https://github.com/answerbook/vector/commit/089ad4860c2e57dd789c1846991325e8acbe1f25) - GitHub [LOG-18011](https://logdna.atlassian.net/browse/LOG-18011)

# [1.10.0](https://github.com/answerbook/vector/compare/v1.9.1...v1.10.0) (2023-08-28)


### Chores

* **deployment**: Add startup probe [0e51db4](https://github.com/answerbook/vector/commit/0e51db462ee2ff9970dc8d707af67b5cde083396) - Michael Penick [LOG-17724](https://logdna.atlassian.net/browse/LOG-17724)


### Features

* Add liveness check for config provider loading [21bf021](https://github.com/answerbook/vector/commit/21bf0211e9aec74c216741af4797742ac0285950) - Michael Penick [LOG-17724](https://logdna.atlassian.net/browse/LOG-17724)

## [1.9.1](https://github.com/answerbook/vector/compare/v1.9.0...v1.9.1) (2023-08-24)


### Bug Fixes

* **pulse**: adjust deserialization for expected payload shape [b41fea2](https://github.com/answerbook/vector/commit/b41fea2cdc3d4f4be0493839de94291eaf9896a4) - Mike Del Tito [LOG-17961](https://logdna.atlassian.net/browse/LOG-17961)
* **pulse**: update expected shape for errors [32cd24e](https://github.com/answerbook/vector/commit/32cd24e97998efda0d20b4eb3b0cc1ace9fca6c8) - Mike Del Tito [LOG-17961](https://logdna.atlassian.net/browse/LOG-17961)
* **pulse**: update shape of results data [dba9f75](https://github.com/answerbook/vector/commit/dba9f75b373c50d18dbbc9eebe4257e39b5762f6) - Mike Del Tito [LOG-17961](https://logdna.atlassian.net/browse/LOG-17961)


### Miscellaneous

* Merge pull request #315 from answerbook/mdeltito/LOG-17961 [1324cf9](https://github.com/answerbook/vector/commit/1324cf9cb2efe3e059b179296ff3f69d9509f270) - GitHub [LOG-17961](https://logdna.atlassian.net/browse/LOG-17961)

# [1.9.0](https://github.com/answerbook/vector/compare/v1.8.2...v1.9.0) (2023-08-23)


### Bug Fixes

* **shutdown**: correct ShutdownSignal polling semantics [0368cef](https://github.com/answerbook/vector/commit/0368cefa506de81c22806dcf54d0154b1d02c4dc) - Dan Hable [LOG-17649](https://logdna.atlassian.net/browse/LOG-17649)


### Features

* **pulse**: Support remote task execution [032d342](https://github.com/answerbook/vector/commit/032d3425b011656bd11ad293ab69b6ba12293de6) - Jorge Bay [LOG-17469](https://logdna.atlassian.net/browse/LOG-17469)

## [1.8.2](https://github.com/answerbook/vector/compare/v1.8.1...v1.8.2) (2023-08-22)


### Bug Fixes

* **config**: use abort() instead of panic!() to kill vector [343865e](https://github.com/answerbook/vector/commit/343865eb264091317a59aa5c9e31b496e64238f6) - Dan Hable [LOG-17772](https://logdna.atlassian.net/browse/LOG-17772)

## [1.8.1](https://github.com/answerbook/vector/compare/v1.8.0...v1.8.1) (2023-08-18)


### Bug Fixes

* **user_log**: log a message when `user_log` is called without context [4da68d4](https://github.com/answerbook/vector/commit/4da68d483526877c7c81c28180d0317e9bf1bdb4) - Mike Del Tito [LOG-17903](https://logdna.atlassian.net/browse/LOG-17903)


### Miscellaneous

* Merge pull request #311 from answerbook/mdeltito/warn-on-user-log-without-context [437bf53](https://github.com/answerbook/vector/commit/437bf530b430dfaad030fba54ce9942d0a02c8e3) - GitHub

# [1.8.0](https://github.com/answerbook/vector/compare/v1.7.0...v1.8.0) (2023-08-17)


### Features

* **sinks**: add metrics to sumo logic destination [698cfd9](https://github.com/answerbook/vector/commit/698cfd9310d349813bee2e672d052ae82a133895) - stsantilena [LOG-17363](https://logdna.atlassian.net/browse/LOG-17363)


### Miscellaneous

* Merge pull request #303 from answerbook/stsantilena/LOG-17363 [df7fdfe](https://github.com/answerbook/vector/commit/df7fdfec38744feb4c83faed280dcfb542bacf98) - GitHub [LOG-17363](https://logdna.atlassian.net/browse/LOG-17363)

# [1.7.0](https://github.com/answerbook/vector/compare/v1.6.3...v1.7.0) (2023-08-14)


### Features

* New usage metrics flusher for Pulse [d8fc4d8](https://github.com/answerbook/vector/commit/d8fc4d8727ca29e45ebe3014b6aadc76da1f0b0f) - Jorge Bay [LOG-17778](https://logdna.atlassian.net/browse/LOG-17778)

## [1.6.3](https://github.com/answerbook/vector/compare/v1.6.2...v1.6.3) (2023-08-10)


### Bug Fixes

* **mezmo logs**: Make apache demo error logs parseable by pipeline parser (#309) [615572a](https://github.com/answerbook/vector/commit/615572a4293bbd41c4f7ba2542f808db86e98989) - GitHub [LOG-17701](https://logdna.atlassian.net/browse/LOG-17701)

## [1.6.2](https://github.com/answerbook/vector/compare/v1.6.1...v1.6.2) (2023-08-10)


### Chores

* add config reload limit to k8s template [0f3ff9a](https://github.com/answerbook/vector/commit/0f3ff9ab75d2ef2161b52a5d85fd9c2e4494c3e2) - Dan Hable [LOG-17772](https://logdna.atlassian.net/browse/LOG-17772)

## [1.6.1](https://github.com/answerbook/vector/compare/v1.6.0...v1.6.1) (2023-08-10)


### Bug Fixes

* **config**: limit time vector waits for new topology to start [5343115](https://github.com/answerbook/vector/commit/534311559f4246f70e76caab9f18b32dc44c30cc) - Dan Hable [LOG-17772](https://logdna.atlassian.net/browse/LOG-17772)

# [1.6.0](https://github.com/answerbook/vector/compare/v1.5.1...v1.6.0) (2023-08-07)


### Features

* **syslog**: enable syslog and splunk-hec source features [f170d92](https://github.com/answerbook/vector/commit/f170d921d07ca77aca48cec8d846df6cabd533b0) - Mike Del Tito [LOG-17752](https://logdna.atlassian.net/browse/LOG-17752)


### Miscellaneous

* Merge pull request #305 from answerbook/mdeltito/LOG-17752 [922c945](https://github.com/answerbook/vector/commit/922c945933eb02fb85da869bf066c35fd1ea7770) - GitHub [LOG-17752](https://logdna.atlassian.net/browse/LOG-17752)

## [1.5.1](https://github.com/answerbook/vector/compare/v1.5.0...v1.5.1) (2023-08-03)


### Bug Fixes

* avoid running release stage if running sanity build [fd956ed](https://github.com/answerbook/vector/commit/fd956ed45ddc56ceace4bdb66813e633f65498bd) - Adam Holmberg [LOG-17726](https://logdna.atlassian.net/browse/LOG-17726)


### Miscellaneous

* Merge pull request #304 from answerbook/holmberg/LOG-17726 [aa6a170](https://github.com/answerbook/vector/commit/aa6a170e718c672b9613d104bc4d8a696114703b) - GitHub [LOG-17726](https://logdna.atlassian.net/browse/LOG-17726)

# [1.5.0](https://github.com/answerbook/vector/compare/v1.4.1...v1.5.0) (2023-07-28)


### Features

* Bump VRL version to v0.4.2 [acf9299](https://github.com/answerbook/vector/commit/acf9299df70b7c030dbd63aacc37c812323a680d) - Michael Penick [LOG-17443](https://logdna.atlassian.net/browse/LOG-17443)

## [1.4.1](https://github.com/answerbook/vector/compare/v1.4.0...v1.4.1) (2023-07-20)


### Bug Fixes

* **protobuf**: protobuf to metric transform does not capture metadata [b3031c2](https://github.com/answerbook/vector/commit/b3031c29985239d527750206c9ec34e5ab0d1530) - Sergey Opria [LOG-17507](https://logdna.atlassian.net/browse/LOG-17507)


### Miscellaneous

* Merge pull request #301 from answerbook/sopria/LOG-17507 [62a5974](https://github.com/answerbook/vector/commit/62a597443e6f4d8ab112244caca4c927a5541879) - GitHub [LOG-17507](https://logdna.atlassian.net/browse/LOG-17507)

# [1.4.0](https://github.com/answerbook/vector/compare/v1.3.6...v1.4.0) (2023-07-14)


### Features

* Adds hashing group instance ID to the Kafka source [a41d94c](https://github.com/answerbook/vector/commit/a41d94c89433cc441ee462e803503104031f5834) - Michael Penick [LOG-17023](https://logdna.atlassian.net/browse/LOG-17023)

## [1.3.6](https://github.com/answerbook/vector/compare/v1.3.5...v1.3.6) (2023-07-13)


### Chores

* **sink**: Refactor Sumo Logic sink [083fcf6](https://github.com/answerbook/vector/commit/083fcf6c716a43fbb08d70beed7b4fd1e9e6c11b) - stsantilena [LOG-17358](https://logdna.atlassian.net/browse/LOG-17358)


### Miscellaneous

* Merge pull request #291 from answerbook/stsantilena/LOG-17358 [9b5c9da](https://github.com/answerbook/vector/commit/9b5c9da7ea74a783c08e14754cbf49bd00a51de3) - GitHub [LOG-17358](https://logdna.atlassian.net/browse/LOG-17358)

## [1.3.5](https://github.com/answerbook/vector/compare/v1.3.4...v1.3.5) (2023-07-11)


### Bug Fixes

* **otlp**: fix for otlp metric structure [2a0b43f](https://github.com/answerbook/vector/commit/2a0b43f62cb50a2f0068dcb7be2b3c77d0115813) - Sergey Opria [LOG-15745](https://logdna.atlassian.net/browse/LOG-15745)


### Miscellaneous

* Merge pull request #297 from answerbook/sopria/LOG-15745 [f5aadbf](https://github.com/answerbook/vector/commit/f5aadbfba3fdf93490152044886616675177c851) - GitHub [LOG-15745](https://logdna.atlassian.net/browse/LOG-15745)

## [1.3.4](https://github.com/answerbook/vector/compare/v1.3.3...v1.3.4) (2023-07-11)


### Bug Fixes

* **mezmo_reduce**: Account for `message` schema prefix [526efca](https://github.com/answerbook/vector/commit/526efcaf2942c86daf6e6c1d6faf656b4f30c14c) - Darin Spivey [LOG-17505](https://logdna.atlassian.net/browse/LOG-17505)


### Miscellaneous

* Merge pull request #298 from answerbook/darinspivey/LOG-17505 [11f9499](https://github.com/answerbook/vector/commit/11f94992c5910bd9c5de6576836736c81b457d18) - GitHub [LOG-17505](https://logdna.atlassian.net/browse/LOG-17505)

## [1.3.3](https://github.com/answerbook/vector/compare/v1.3.2...v1.3.3) (2023-07-10)


### Bug Fixes

* **mezmo_reduce**: Allow field paths for merge strategies and dates [f427f32](https://github.com/answerbook/vector/commit/f427f3229b0be34648f3b03eadb306dce62af719) - Darin Spivey [LOG-17480](https://logdna.atlassian.net/browse/LOG-17480)


### Miscellaneous

* Merge pull request #296 from answerbook/darinspivey/LOG-17480 [276128a](https://github.com/answerbook/vector/commit/276128aec5e3d3f8fae9544a3ab537f463926ada) - GitHub [LOG-17480](https://logdna.atlassian.net/browse/LOG-17480)

## [1.3.2](https://github.com/answerbook/vector/compare/v1.3.1...v1.3.2) (2023-07-06)


### Chores

* Bump vrl lib to 0.3.2 [2910836](https://github.com/answerbook/vector/commit/29108367683f76ce7836797c4d4bb1b75324a0f5) - Jorge Bay [LOG-17425](https://logdna.atlassian.net/browse/LOG-17425)

## [1.3.1](https://github.com/answerbook/vector/compare/v1.3.0...v1.3.1) (2023-06-29)


### Bug Fixes

* **mezmo_reduce**: Cloned finalizers causing duplicate data [0325b02](https://github.com/answerbook/vector/commit/0325b02d3e09c69fb934e5b1e1f97c1fdd9eb91e) - Darin Spivey [LOG-16873](https://logdna.atlassian.net/browse/LOG-16873)
* **mezmo_reduce**: Efficiency improvement for `transform_one()` [d1cf969](https://github.com/answerbook/vector/commit/d1cf969b4678c25df5a178713627961b0da112b4) - Darin Spivey [LOG-16873](https://logdna.atlassian.net/browse/LOG-16873)


### Miscellaneous

* Merge pull request #294 from answerbook/darinspivey/LOG-16873 [ccd4832](https://github.com/answerbook/vector/commit/ccd483229ad33fb9237e094ee6feb73a78df4e0a) - GitHub [LOG-16873](https://logdna.atlassian.net/browse/LOG-16873)

# [1.3.0](https://github.com/answerbook/vector/compare/v1.2.0...v1.3.0) (2023-06-29)


### Features

* Bump to VRL version v0.2.0.6 [34f62d8](https://github.com/answerbook/vector/commit/34f62d8b94984a212da89166cf4e3366c2469e98) - Michael Penick [LOG-17381](https://logdna.atlassian.net/browse/LOG-17381)

# [1.2.0](https://github.com/answerbook/vector/compare/v1.1.3...v1.2.0) (2023-06-29)


### Features

* Bump to VRL version v0.2.0.5 [e5ad3e2](https://github.com/answerbook/vector/commit/e5ad3e28725e67ee4173a63dea9f11283104c2ee) - Michael Penick [LOG-17381](https://logdna.atlassian.net/browse/LOG-17381)

## [1.1.3](https://github.com/answerbook/vector/compare/v1.1.2...v1.1.3) (2023-06-23)


### Bug Fixes

* **transform**: move protobuf transform into mezmo build [f3277a2](https://github.com/answerbook/vector/commit/f3277a265504aa1bfd6b20b12e9393edd99ab171) - Sergey Opria [LOG-15745](https://logdna.atlassian.net/browse/LOG-15745)


### Miscellaneous

* Merge pull request #290 from answerbook/sopria/LOG-15745 [f276180](https://github.com/answerbook/vector/commit/f27618039dedfb30c13f0d6359d726742909e82b) - GitHub [LOG-15745](https://logdna.atlassian.net/browse/LOG-15745)

## [1.1.2](https://github.com/answerbook/vector/compare/v1.1.1...v1.1.2) (2023-06-16)


### Chores

* Bump VRL to include new AST-friendly functions [5124351](https://github.com/answerbook/vector/commit/5124351b39f8c3458ae5cccf3349fb57ab84ce7a) - Jorge Bay [LOG-17157](https://logdna.atlassian.net/browse/LOG-17157)

## [1.1.1](https://github.com/answerbook/vector/compare/v1.1.0...v1.1.1) (2023-06-16)


### Bug Fixes

* **lib/codecs**: cache metadata for prom_rw [3d5b410](https://github.com/answerbook/vector/commit/3d5b4107236f1f928f3871dbc1ba35a3c8666cf6) - Chris Nixon [LOG-17317](https://logdna.atlassian.net/browse/LOG-17317)


### Chores

* **lib/codecs**: minor tidying [46fe265](https://github.com/answerbook/vector/commit/46fe265b723d9ef24236d210f7838c52f7e7a196) - Chris Nixon [LOG-17317](https://logdna.atlassian.net/browse/LOG-17317)

# [1.1.0](https://github.com/answerbook/vector/compare/v1.0.4...v1.1.0) (2023-06-14)


### Features

* **otlp**: transform protobuf to metric [bbf2208](https://github.com/answerbook/vector/commit/bbf22085bf9c24f53639e3d3529063c911d16e25) - Sergey Opria [LOG-15745](https://logdna.atlassian.net/browse/LOG-15745)


### Miscellaneous

* Merge pull request #277 from answerbook/sopria/LOG-15745 [79736c9](https://github.com/answerbook/vector/commit/79736c9667100200ba4038f83c2fac9fd1885e10) - GitHub [LOG-15745](https://logdna.atlassian.net/browse/LOG-15745)

## [1.0.4](https://github.com/answerbook/vector/compare/v1.0.3...v1.0.4) (2023-06-14)


### Bug Fixes

* bump prometheus-remote-write [78389fa](https://github.com/answerbook/vector/commit/78389fa5acfd8637a9ce434f18c1a0bf24d6fc70) - Chris Nixon [LOG-17287](https://logdna.atlassian.net/browse/LOG-17287)

## [1.0.3](https://github.com/answerbook/vector/compare/v1.0.2...v1.0.3) (2023-06-13)


### Build System

* add npm plugin to semantic relase process [379c7f1](https://github.com/answerbook/vector/commit/379c7f13a3ca7994976982ab7d928df8bd3061b5) - Dan Hable [LOG-16798](https://logdna.atlassian.net/browse/LOG-16798)

## [1.0.2](https://github.com/answerbook/vector/compare/v1.0.1...v1.0.2) (2023-06-13)


### Bug Fixes

* tag release image with release-version [cd68984](https://github.com/answerbook/vector/commit/cd68984531b66218c86df5504938f7178ffcf121) - Chris Nixon [LOG-16798](https://logdna.atlassian.net/browse/LOG-16798)

## [1.0.1](https://github.com/answerbook/vector/compare/v1.0.0...v1.0.1) (2023-06-13)


### Chores

* lint fix in usage_metrics [34be2fd](https://github.com/answerbook/vector/commit/34be2fdb522236cb45a38c57f68b9003822e858e) - Chris Nixon [LOG-16798](https://logdna.atlassian.net/browse/LOG-16798)


### Miscellaneous

* 2023-06-13, Version 1.0.0 [skip ci] [83344cc](https://github.com/answerbook/vector/commit/83344cc7e23ede4aadc7377327292962e38b8acd) - Dan Hable
