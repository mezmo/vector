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
