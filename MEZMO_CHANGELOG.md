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
