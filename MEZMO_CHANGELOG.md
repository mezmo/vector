## [8.7.5](https://github.com/answerbook/vector/compare/v8.7.4...v8.7.5) (2025-02-07)


### Bug Fixes

* **mezmo::config**: avoid revalidating all pipelines when one is removed [9aaae9f](https://github.com/answerbook/vector/commit/9aaae9fee34c70a90f2efa277d19664803b169b3) - Mike Del Tito [LOG-21350](https://logdna.atlassian.net/browse/LOG-21350)
* **mezmo::config**: avoid walking the graph for input definitions [a63908a](https://github.com/answerbook/vector/commit/a63908a38e2e21a82676e2704f7e67cc4df60d68) - Mike Del Tito [LOG-21352](https://logdna.atlassian.net/browse/LOG-21352)


### Miscellaneous

* Merge pull request #646 from answerbook/mdeltito/LOG-21352 [f262fcb](https://github.com/answerbook/vector/commit/f262fcb930297956312856ce1b473d076daace81) - GitHub [LOG-21352](https://logdna.atlassian.net/browse/LOG-21352)

## [8.7.4](https://github.com/answerbook/vector/compare/v8.7.3...v8.7.4) (2025-01-14)


### Bug Fixes

* **config::compiler**: add trace logging [0736ee6](https://github.com/answerbook/vector/commit/0736ee60a1b5b759a716345392060fc353a7dd50) - Mike Del Tito [LOG-21226](https://logdna.atlassian.net/browse/LOG-21226)


### Miscellaneous

* Merge pull request #644 from answerbook/mdeltito/LOG-21226-more [78a8dc5](https://github.com/answerbook/vector/commit/78a8dc56b719aada0cd018d611e767c51b8a1305) - GitHub [LOG-21226](https://logdna.atlassian.net/browse/LOG-21226)

## [8.7.3](https://github.com/answerbook/vector/compare/v8.7.2...v8.7.3) (2025-01-13)


### Bug Fixes

* **metrics**: update the tcp-prometheus port [81f8066](https://github.com/answerbook/vector/commit/81f8066f7a3075f49cab7ece92c9457513de6c72) - Tony Rogers [INFRA-7321](https://logdna.atlassian.net/browse/INFRA-7321)


### Miscellaneous

* Merge pull request #643 from answerbook/INFRA-7321 [4f4239e](https://github.com/answerbook/vector/commit/4f4239eaa4991c91853ddedd627e091fd80695bb) - GitHub [INFRA-7321](https://logdna.atlassian.net/browse/INFRA-7321)

## [8.7.2](https://github.com/answerbook/vector/compare/v8.7.1...v8.7.2) (2025-01-09)


### Chores

* **mezmo::config**: improve and add debug/trace logging [c6409df](https://github.com/answerbook/vector/commit/c6409df92bc2359e19a7888e2d490ef9d2cae45b) - Mike Del Tito [LOG-21226](https://logdna.atlassian.net/browse/LOG-21226)


### Miscellaneous

* Merge pull request #642 from answerbook/mdeltito/LOG-21226 [a6cdc18](https://github.com/answerbook/vector/commit/a6cdc18206678e3964892fa2bd161f6939697a5c) - GitHub [LOG-21226](https://logdna.atlassian.net/browse/LOG-21226)

## [8.7.1](https://github.com/answerbook/vector/compare/v8.7.0...v8.7.1) (2025-01-07)


### Bug Fixes

* **honeycomb sink**: The batch body should be encoded as array (#21878) [0199ab0](https://github.com/answerbook/vector/commit/0199ab0ddce00ea689a6ca67d5f742a96930fa07) - Darin Spivey [LOG-20430](https://logdna.atlassian.net/browse/LOG-20430)
* **sinks**: Honeycomb sink needs Mezmo reshaping [bbb4844](https://github.com/answerbook/vector/commit/bbb484400375c23f538445e637994e8a2894c169) - Darin Spivey [LOG-20430](https://logdna.atlassian.net/browse/LOG-20430)

# [8.7.0](https://github.com/answerbook/vector/compare/v8.6.0...v8.7.0) (2024-12-11)


### Features

* **logging**: raise cpu thread pool message to info [b0f1a47](https://github.com/answerbook/vector/commit/b0f1a471cdfdf35dd9847f173d19abf4a60e12f3) - Dan Hable [LOG-21168](https://logdna.atlassian.net/browse/LOG-21168)

# [8.6.0](https://github.com/answerbook/vector/compare/v8.5.4...v8.6.0) (2024-12-10)


### Features

* **metrics**: Expose port for vector internal metrics [95f51c1](https://github.com/answerbook/vector/commit/95f51c1163d946d908393057720a0da857ae679a) - Katerina Koutsonikoli [INFRA-7321](https://logdna.atlassian.net/browse/INFRA-7321)


### Miscellaneous

* Merge pull request #639 from answerbook/internal-vector-metrics [f68c557](https://github.com/answerbook/vector/commit/f68c5574e6a9b160cb9b98e9f068e0f83280f4c2) - GitHub

## [8.5.4](https://github.com/answerbook/vector/compare/v8.5.3...v8.5.4) (2024-12-04)


### Miscellaneous

* Merge pull request #638 from answerbook/INFRA-7294/martin [c721737](https://github.com/answerbook/vector/commit/c72173763c8718e7c4deedbebe17b5f8815c9809) - GitHub [INFRA-7294](https://logdna.atlassian.net/browse/INFRA-7294)


### Tests

* **usage-metrics**: prepared statements [50fe3be](https://github.com/answerbook/vector/commit/50fe3bec761571b04d6942d50cb03f0c3c020915) - Martin Hansen [INFRA-7327](https://logdna.atlassian.net/browse/INFRA-7327)

## [8.5.3](https://github.com/answerbook/vector/compare/v8.5.2...v8.5.3) (2024-11-25)


### Chores

* Update secret-key name [28c9ac2](https://github.com/answerbook/vector/commit/28c9ac28e3d4f9016e056eb678c3aa4e40ed38ac) - Eric Lake [INFRA-7303](https://logdna.atlassian.net/browse/INFRA-7303)


### Miscellaneous

* Merge pull request #637 from answerbook/secret-key-name [7ad3c84](https://github.com/answerbook/vector/commit/7ad3c8458de2c6dd923f43928f7321d021ec3d9a) - GitHub

## [8.5.2](https://github.com/answerbook/vector/compare/v8.5.1...v8.5.2) (2024-11-20)


### Bug Fixes

* **tracing**: add persistence config [6543351](https://github.com/answerbook/vector/commit/65433512f2a02be561b0e57dade6412289a9c1e4) - dominic-mcallister-logdna [LOG-20949](https://logdna.atlassian.net/browse/LOG-20949)


### Miscellaneous

* Merge pull request #636 from answerbook/dominic/LOG-20949-directory [362c41c](https://github.com/answerbook/vector/commit/362c41c9ab7f8fd6c03ce03cf73635917e8439ab) - GitHub [LOG-20949](https://logdna.atlassian.net/browse/LOG-20949)

## [8.5.1](https://github.com/answerbook/vector/compare/v8.5.0...v8.5.1) (2024-11-19)


### Bug Fixes

* **infra logs**: Remove $ characters from component names (#634) [ad2b971](https://github.com/answerbook/vector/commit/ad2b971c2c7ef32d450e12dfb6ed43382cb12ecd) - GitHub [LOG-20991](https://logdna.atlassian.net/browse/LOG-20991)

# [8.5.0](https://github.com/answerbook/vector/compare/v8.4.0...v8.5.0) (2024-11-19)


### Features

* **tracing**: implement head based sampling transform [022f4b1](https://github.com/answerbook/vector/commit/022f4b1dc76e24290d1771ed56889c907546d4f7) - dominic-mcallister-logdna [LOG-20949](https://logdna.atlassian.net/browse/LOG-20949)


### Miscellaneous

* Merge pull request #635 from answerbook/dominic/LOG-20949 [19269f3](https://github.com/answerbook/vector/commit/19269f39acf293b4cc1f035077d1852e8a1fe0e6) - GitHub [LOG-20949](https://logdna.atlassian.net/browse/LOG-20949)

# [8.4.0](https://github.com/answerbook/vector/compare/v8.3.1...v8.4.0) (2024-11-14)


### Bug Fixes

* **deployment**: template database uri [c53e3d3](https://github.com/answerbook/vector/commit/c53e3d3f2d51b46e8dcc9a124a1d4657502cecc2) - Martin Hansen [INFRA-7294](https://logdna.atlassian.net/browse/INFRA-7294)


### Continuous Integration

* Simplifying Jenkinsfile and adding conditional for Dockerfile test [56a698f](https://github.com/answerbook/vector/commit/56a698f39e0b5d62d2d5bcc6040c0d821952b844) - Dan Hable [REL-1757](https://logdna.atlassian.net/browse/REL-1757)


### Features

* **deployment**: template database uri [4a86d60](https://github.com/answerbook/vector/commit/4a86d60a39f5bbd6eb514e7d40dd31d75b7af3f9) - Martin Hansen [INFRA-7294](https://logdna.atlassian.net/browse/INFRA-7294) [LOG-20937](https://logdna.atlassian.net/browse/LOG-20937)


### Miscellaneous

* Merge pull request #633 from answerbook/spr/master/9a5d99be [66c28e3](https://github.com/answerbook/vector/commit/66c28e391937e023fa4a8654b33020af07344fe3) - GitHub
* Merge pull request #631 from answerbook/spr/master/1a94512e [2a8ff6e](https://github.com/answerbook/vector/commit/2a8ff6eae3196125349bc512989191d615563e0b) - GitHub

## [8.3.1](https://github.com/answerbook/vector/compare/v8.3.0...v8.3.1) (2024-11-07)


### Bug Fixes

* **persistence**: ensure read metrics are captured [84db62b](https://github.com/answerbook/vector/commit/84db62be4ff7e9d4a7946a01f5886fdf128248a7) - Mike Del Tito [LOG-20916](https://logdna.atlassian.net/browse/LOG-20916)
* **persistence**: reuse rocksdb options instance [7da32f3](https://github.com/answerbook/vector/commit/7da32f3b6f9651440e52aef5390d576886e16a8b) - Mike Del Tito [LOG-20916](https://logdna.atlassian.net/browse/LOG-20916)


### Miscellaneous

* Merge pull request #630 from answerbook/mdeltito/LOG-20916-fix [66dd1c4](https://github.com/answerbook/vector/commit/66dd1c4bb3c9b2ae864207816b3093cd84487ec8) - GitHub [LOG-20916](https://logdna.atlassian.net/browse/LOG-20916)

# [8.3.0](https://github.com/answerbook/vector/compare/v8.2.0...v8.3.0) (2024-11-05)


### Features

* **persistence**: expose per-db metrics for rocksdb [b818e74](https://github.com/answerbook/vector/commit/b818e74c9f4310b73613a9abbc089832e1800ff2) - Mike Del Tito [LOG-20916](https://logdna.atlassian.net/browse/LOG-20916)


### Miscellaneous

* Merge pull request #629 from answerbook/mdeltito/LOG-20916 [c1c43a7](https://github.com/answerbook/vector/commit/c1c43a7fe5868cbb89f5940d8583a3ea493b5f8a) - GitHub [LOG-20916](https://logdna.atlassian.net/browse/LOG-20916)

# [8.2.0](https://github.com/answerbook/vector/compare/v8.1.5...v8.2.0) (2024-11-05)


### Features

* **demo-logs**: Add infrastructure and kubernetes logs (#628) [f495e3d](https://github.com/answerbook/vector/commit/f495e3d68ec14c11bb3aef393540f90cff64a75c) - GitHub [LOG-20855](https://logdna.atlassian.net/browse/LOG-20855)

## [8.1.5](https://github.com/answerbook/vector/compare/v8.1.4...v8.1.5) (2024-11-04)


### Bug Fixes

* **aggregate-v2**: move rocksdb persistence to blocking thread [81768d7](https://github.com/answerbook/vector/commit/81768d72dd96b54556dfb65436993dfdf3a6d6d8) - Mike Del Tito [LOG-20906](https://logdna.atlassian.net/browse/LOG-20906)
* **throttle**: move rocksdb persistence to blocking thread [828c465](https://github.com/answerbook/vector/commit/828c4650a06c4e514f6a44221dba13d32a910039) - Mike Del Tito [LOG-20906](https://logdna.atlassian.net/browse/LOG-20906)


### Miscellaneous

* Merge pull request #627 from answerbook/mdeltito/LOG-20906 [eb48c99](https://github.com/answerbook/vector/commit/eb48c99f86477c67f55d8f2c3f03a9902303f485) - GitHub [LOG-20906](https://logdna.atlassian.net/browse/LOG-20906)

## [8.1.4](https://github.com/answerbook/vector/compare/v8.1.3...v8.1.4) (2024-10-31)


### Bug Fixes

* **vrl**: pick up to_float / mezmo_to_float fix [f3f0402](https://github.com/answerbook/vector/commit/f3f04023ada72b97bc8e83485794e960416ecf4c) - Dan Hable [LOG-20860](https://logdna.atlassian.net/browse/LOG-20860)

## [8.1.3](https://github.com/answerbook/vector/compare/v8.1.2...v8.1.3) (2024-10-31)


### Bug Fixes

* **cluster**: size as event size instead of line size [7e69972](https://github.com/answerbook/vector/commit/7e699721d4313a3ccfea0e2ef7854c9bb64955b4) - dominic-mcallister-logdna [LOG-20872](https://logdna.atlassian.net/browse/LOG-20872)


### Miscellaneous

* Merge pull request #626 from answerbook/dominic/LOG-20872 [ea25e2b](https://github.com/answerbook/vector/commit/ea25e2be0b384c34c9a0b7b0f168501eaa75777c) - GitHub [LOG-20872](https://logdna.atlassian.net/browse/LOG-20872)

## [8.1.2](https://github.com/answerbook/vector/compare/v8.1.1...v8.1.2) (2024-10-28)


### Bug Fixes

* **alloc-tracing**: reverts reuse allocation group ids [b83d5d1](https://github.com/answerbook/vector/commit/b83d5d10b92fc96a2e0de7899fe7646689bb6894) - Dan Hable [LOG-20743](https://logdna.atlassian.net/browse/LOG-20743)

## [8.1.1](https://github.com/answerbook/vector/compare/v8.1.0...v8.1.1) (2024-10-22)


### Bug Fixes

* **alloc-tracing**: reuse allocation group ids [83c3874](https://github.com/answerbook/vector/commit/83c38749dd18e87e2bc153fb050be14701090157) - Dan Hable [LOG-20743](https://logdna.atlassian.net/browse/LOG-20743)

# [8.1.0](https://github.com/answerbook/vector/compare/v8.0.0...v8.1.0) (2024-10-21)


### Features

* **s3**: support timeout conf in consolidation [72a9898](https://github.com/answerbook/vector/commit/72a98981247f1b7c8d58eb18a75b637ec1c969fd) - Dan Hable [LOG-20807](https://logdna.atlassian.net/browse/LOG-20807)

# [8.0.0](https://github.com/answerbook/vector/compare/v7.0.1...v8.0.0) (2024-10-18)


### Bug Fixes

* **amqp source**: fix crash when handling metrics (#21141) [cade0d2](https://github.com/answerbook/vector/commit/cade0d2e87d9b7c6da13dcc3934b84299af8232d) - GitHub
* **aws_kinesis sink**: fix batching of requests #20575 #1407 (#20653) [8834741](https://github.com/answerbook/vector/commit/8834741a4dc182618a5c7801782814d5923d1507) - GitHub
* **ci**: usage of `a deprecated Node.js version` (#21210) [b546f12](https://github.com/answerbook/vector/commit/b546f12aa5c20559f48ceb5fa13d1c1002902618) - GitHub
* **codecs**: use nanosecond-precision timestamps in `influxdb` decoder (#21042) [b9b98ff](https://github.com/answerbook/vector/commit/b9b98ff20ba61d5f20f5a90ed3c2e99830298655) - GitHub
* **config**: allow usage of metrics-only decoders in log sources (#21040) [ee0168a](https://github.com/answerbook/vector/commit/ee0168a65725da544508e890ff2c6ab014f2957a) - GitHub
* **config**: allows fetching secrets from AWS secrets manager with sso profiles (#21038) [7f206cd](https://github.com/answerbook/vector/commit/7f206cdc507d775154f7d6c1ae96a9a6fd0a5650) - GitHub
* **config**: Avoid parsing configuration files without interpolating secrets (#20985) [93e423f](https://github.com/answerbook/vector/commit/93e423feeea60cfbabe9af692d4afab221eac788) - GitHub
* **config**: For templates using stftime specifiers use semantic timestamp (#20851) [b679245](https://github.com/answerbook/vector/commit/b6792451711511585d33db991a446765f1d4d723) - GitHub
* **config**: Make config watcher recursive. (#20996) [559e069](https://github.com/answerbook/vector/commit/559e069caffaf7826d8e2feea4647b9c944128be) - GitHub
* disallow zero values for sink retry parameters (#20891) [5256de1](https://github.com/answerbook/vector/commit/5256de14796c29a1cbe116495215cd61fbec17b7) - GitHub
* **docs, codecs**: Correctly render character delimiter as a char in the docs (#21124) [76a525b](https://github.com/answerbook/vector/commit/76a525b17368a3b656ff7d3c76ef8feb28fd4908) - GitHub
* **docs, vrl stdlib**: Document that `to_int` coerces `null`s (#21154) [968b5df](https://github.com/answerbook/vector/commit/968b5df014a027dd1a4cb366e3db383c1c501c6c) - GitHub
* **file sources**: exclude pattern with multi slashes can not match some files (#21082) [e95f098](https://github.com/answerbook/vector/commit/e95f098f3e10af6b40038d100c1601252ef156c3) - GitHub
* **GcpAuthenticator**: Improve token regeneration (#20574) [1589eb3](https://github.com/answerbook/vector/commit/1589eb334421d17d0afd00fee5a37666eda1f0ed) - GitHub
* **gcs**: replace // with / when merging base url and object key (#20810) [8325300](https://github.com/answerbook/vector/commit/8325300a767bf342f455ad796fe01c9722bb8fd9) - GitHub
* **kafka sink**: Use rdkafka::client::Client instead of Consumer (#21129) [66b55fe](https://github.com/answerbook/vector/commit/66b55fe74b1ca231368dfe5aeea29a6cc41bc9dd) - GitHub
* **kafka source**: consumer subscribe in main kafka source task (#20698) [476016b](https://github.com/answerbook/vector/commit/476016b28890df879789c5408dfab5c4eb80c33e) - GitHub
* **loki sinks**: Fix loki event timestamp out of range panic (#20780) [6d179e5](https://github.com/answerbook/vector/commit/6d179e523164d1e2332ac644746104bbdfdfed22) - GitHub
* **metrics**: use correct series payload origin labels (#21068) [61b1b18](https://github.com/answerbook/vector/commit/61b1b187169739870d94b6e709a901398f865aac) - GitHub
* **playground**: removes conflicting overflow settings creating multiple scrollbars (#21168) [012a18d](https://github.com/answerbook/vector/commit/012a18df708a299369002f56f1b7510c9c5c4c9f) - GitHub
* **proto**: Remove error log when source_event_id is not present (#21257) [702b221](https://github.com/answerbook/vector/commit/702b22128f25477948a386920048b7333ff369c4) - Jesse Szwedko
* **proxy**: support url-encoded auth values (#20868) [e9b4fe1](https://github.com/answerbook/vector/commit/e9b4fe15b5a84ee9cd2459a70ca92e6c634d519a) - GitHub
* **reduce transform**: surround invalid path segments with quotes (#21201) [f2155f1](https://github.com/answerbook/vector/commit/f2155f173ef33996cec0d6a05fab7776a2ab543d) - GitHub
* **reduce transform**: use the correct merge strategy for top level objects (#21067) [f7e4470](https://github.com/answerbook/vector/commit/f7e4470f6e1a415c8b93fbc290ad9c6d05d77f14) - GitHub
* **reduce**: reduce values for nested fields (#20800) [2d9b1c4](https://github.com/answerbook/vector/commit/2d9b1c434478dc969a93028e7b21b0401d0c6d74) - GitHub
* **socket sink**: Allow socket sink to accept metrics. (#20930) [9121a7f](https://github.com/answerbook/vector/commit/9121a7fe711ba673aa5aa791a6e11530f40dceb6) - GitHub
* **socket source**: fix socket source ignoring global log_namespace when computing outputs (#20966) [ecce2ed](https://github.com/answerbook/vector/commit/ecce2ed10ed593f1e0d1d1bf17726013aeaf2e4e) - GitHub
* **transforms**: Hande numeric equality in `datadog_search` condition  (#21179) [c2dc1aa](https://github.com/answerbook/vector/commit/c2dc1aa81f55588c44b6b6f0bcb0bbcb34d0fe06) - GitHub
* **vrl**: Mark set_secret and remove_secret as impure (#20820) [fce0fbf](https://github.com/answerbook/vector/commit/fce0fbfddc08c1334cabd9ca2a48947d9d38eeb3) - GitHub


### Chores

* Adds a trailing newline escape for smp job submit (#20829) [60552ab](https://github.com/answerbook/vector/commit/60552ab7c7e6ec0569115c01e3caadbc27d46647) - GitHub
* **ci**: add codec test data dir to regression workflow ignore filter (#21185) [78c470e](https://github.com/answerbook/vector/commit/78c470e490f1c0d544e02093c832858bc724d307) - GitHub
* **ci**: Add global CODEOWNERS fallback (#20947) [d728d5a](https://github.com/answerbook/vector/commit/d728d5aa9f82712f0660e5fb313bda8f6bd19912) - GitHub
* **ci**: Bump actions/add-to-project from 1.0.1 to 1.0.2 (#20738) [9bbf21f](https://github.com/answerbook/vector/commit/9bbf21fc44c0ecdcb62c8d5e0346a6b26705d7a0) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.33.0 to 1.34.0 (#20723) [ef218fa](https://github.com/answerbook/vector/commit/ef218fa0c922b4d2048f02f887fd34386fc52b8d) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.34.0 to 1.35.0 (#20915) [dea7e2b](https://github.com/answerbook/vector/commit/dea7e2bf3e69cd297d56c313363566d540c05416) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.35.0 to 1.35.1 (#20929) [96de0ac](https://github.com/answerbook/vector/commit/96de0ac0be0da6228a38906cd8e3400e7b6910cc) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.35.1 to 1.36.0 (#21019) [2a0c51f](https://github.com/answerbook/vector/commit/2a0c51fbcfb951d7ba4b4b82d01bf2c41b70b898) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.36.0 to 1.37.0 (#21100) [970b222](https://github.com/answerbook/vector/commit/970b2225c45b72d4a5dacb918eca533f0fffbe70) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.37.0 to 1.38.0 (#21135) [b5c137b](https://github.com/answerbook/vector/commit/b5c137bc5e0acb773ab453789be0494db51e6344) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.38.0 to 1.39.0 (#21172) [832c02e](https://github.com/answerbook/vector/commit/832c02eb124c8e003d08de609177e9fde806f0fd) - GitHub
* **ci**: Bump docker/build-push-action from 5.4.0 to 6.2.0 (#20748) [b256ba2](https://github.com/answerbook/vector/commit/b256ba283270e5996a11231de5c4f4f3775ed882) - GitHub
* **ci**: Bump docker/build-push-action from 6.2.0 to 6.3.0 (#20788) [5a54444](https://github.com/answerbook/vector/commit/5a54444dcc7bc809b79cd7fa7e047f41f1cdcd62) - GitHub
* **ci**: Bump docker/build-push-action from 6.3.0 to 6.4.1 (#20877) [7c56a11](https://github.com/answerbook/vector/commit/7c56a11cffdc8a26cda0389e3052e5ef898bafe7) - GitHub
* **ci**: Bump docker/build-push-action from 6.4.1 to 6.5.0 (#20907) [5870099](https://github.com/answerbook/vector/commit/587009971ed6fcbbc915c4440255e3a262bdbd4f) - GitHub
* **ci**: Bump docker/build-push-action from 6.5.0 to 6.6.1 (#21030) [abc2133](https://github.com/answerbook/vector/commit/abc21331e59bf0492c21466c0e50af15af5fd1d9) - GitHub
* **ci**: Bump docker/build-push-action from 6.6.1 to 6.7.0 (#21064) [4884c29](https://github.com/answerbook/vector/commit/4884c295e5ea6478529e9b4d149356f28a79bbc2) - GitHub
* **ci**: Bump docker/setup-buildx-action from 3.3.0 to 3.4.0 (#20838) [3d630d9](https://github.com/answerbook/vector/commit/3d630d99b907b80c2c4522a2d404fd333e6f2283) - GitHub
* **ci**: Bump docker/setup-buildx-action from 3.4.0 to 3.5.0 (#20906) [18fad40](https://github.com/answerbook/vector/commit/18fad409ec4f01841ec98c705a955ff3276b2104) - GitHub
* **ci**: Bump docker/setup-buildx-action from 3.5.0 to 3.6.1 (#20960) [584ed34](https://github.com/answerbook/vector/commit/584ed34d33627ec93a91e04ed5ddda91f5db8f19) - GitHub
* **ci**: Bump docker/setup-qemu-action from 3.0.0 to 3.1.0 (#20787) [2544107](https://github.com/answerbook/vector/commit/25441075689e044f911c620baff86825f1f6f017) - GitHub
* **ci**: Bump docker/setup-qemu-action from 3.1.0 to 3.2.0 (#20905) [2c75df4](https://github.com/answerbook/vector/commit/2c75df4f829970d248441b1201d7272e296d9e15) - GitHub
* **ci**: Bump regression workflow timeout to 60 minutes (#21136) [08a1a4c](https://github.com/answerbook/vector/commit/08a1a4cf5c9388dae3bd909c18ab0d859a8b7a37) - GitHub
* **ci**: Bump Ruby to v3 [536f9e2](https://github.com/answerbook/vector/commit/536f9e2aca99a0910509265633900770caea6f0e) - Jesse Szwedko
* **ci**: Correct username fetching in comment trigger [cfadf34](https://github.com/answerbook/vector/commit/cfadf34971d841232df73c59a0c089f0ba6745d3) - Jesse Szwedko
* **ci**: Fix and reenable eventstoredb integration test (#20873) [1032408](https://github.com/answerbook/vector/commit/103240846e93b48213cda50e0025c756d217647c) - GitHub
* **ci**: Fix fetching PR number for regression workflow (#21183) [3bf93ef](https://github.com/answerbook/vector/commit/3bf93ef1f19a5a5afcc1679a002e8b0f46cdf67e) - GitHub
* **ci**: Fix finding of merge-base for regression workflow (#21186) [ebc5337](https://github.com/answerbook/vector/commit/ebc53379fb443291e6ca56c6ec72f9262110d73c) - GitHub
* **ci**: Fix review comment trigger context (#21010) [f1a1e1c](https://github.com/answerbook/vector/commit/f1a1e1c93c12857644c85f89a77bae3eab1f8a43) - GitHub
* **ci**: Fix review workflow trigger (#21182) [c132f35](https://github.com/answerbook/vector/commit/c132f35336aab3922e8595bfc910d103a074ac61) - GitHub
* **ci**: Mark file_to_blackhole soak as erratic (#20822) [c448b23](https://github.com/answerbook/vector/commit/c448b237e0c5ac649e538230a7f85cb1a9b6bab5) - GitHub
* **ci**: Regenerate certificates used for nats integration tests (#21113) [03b030e](https://github.com/answerbook/vector/commit/03b030eb1cb2af277c49ceed0f8e809b0bbf1543) - GitHub
* **ci**: Reinstall rustup in MacOS bootstrap (#20911) [3b6a738](https://github.com/answerbook/vector/commit/3b6a73845ed5bbd6be12e36b113127db589196f0) - GitHub
* **ci**: Remove reference to the deleted workload checks workflow (#20734) [c8be561](https://github.com/answerbook/vector/commit/c8be561b5c4f3e9778275339d4a81de2a7abd02f) - GitHub
* **ci**: Replace usages of `docker-compose` with `docker compose` (#21009) [2fbb072](https://github.com/answerbook/vector/commit/2fbb072155008b54cb064ab62d9ebc9783b30479) - GitHub
* **ci**: Restrict comment trigger workflow to repository (#20736) [d0dcf35](https://github.com/answerbook/vector/commit/d0dcf356ed922917179451a6920b389cadd0768e) - GitHub
* **ci**: Restrict integration comment workflow to PRs from maintainers (#20743) [099b043](https://github.com/answerbook/vector/commit/099b04304ccc5e7f69cae991edc18161ae4e2d0c) - GitHub
* **ci**: Revert "Add trailing slash to aws endpoint examples" (#20791) [0088883](https://github.com/answerbook/vector/commit/0088883e0c21a15e0228d2dd53d94494a382c21d) - GitHub
* **ci**: Run component docs check if component cue files change (#20793) [6680b15](https://github.com/answerbook/vector/commit/6680b15d9e4018f5ec7967910a6737d45bee4029) - GitHub
* **ci**: Swap out mockwatchlogs for localstack (#21114) [46fccce](https://github.com/answerbook/vector/commit/46fccce76c8d579611d5bb4c148d64f27919e550) - GitHub
* **ci**: Switch to PR Reviews for triggering CI (#20892) [6e803a3](https://github.com/answerbook/vector/commit/6e803a3aa24dc00ca232b152c275e41da1adc090) - GitHub
* **ci**: Temporarily disable eventstoredb integration tests (#20869) [7916ad5](https://github.com/answerbook/vector/commit/7916ad5c55a8ad0265d58429272a9c0d1e6c9c2c) - GitHub
* **ci**: Update version of cue to latest (0.10.0) (#21217) [9693f52](https://github.com/answerbook/vector/commit/9693f5253c2ce306de3cfa40d0c52439df98fc71) - GitHub
* **ci**: Use NodeJS v16 for package verification workflows (#20818) [bf91664](https://github.com/answerbook/vector/commit/bf916643212c542059513b265d2344e01540ebf3) - GitHub
* **ci**: Validate PR author instead of repository (#20741) [37e0c1d](https://github.com/answerbook/vector/commit/37e0c1dc5c532d8dc3b4c175566663c057158c2a) - GitHub
* **config**: Configure `datadog_search` condition directly (#21174) [1cc1761](https://github.com/answerbook/vector/commit/1cc17611c0a71f4d6afbfe72b4da835ae91874cd) - GitHub
* **config**: Refactor secrets loading to avoid use of futures::executor::block_on (#21073) [e601b9b](https://github.com/answerbook/vector/commit/e601b9b636df9a8e1214f4c8b02559d31d979fc0) - GitHub
* **core**: Add helper function to add a semantic meaning to an event metadata (#20439) [67a5e46](https://github.com/answerbook/vector/commit/67a5e4682138721575b22369cf5ba629cd86ce55) - GitHub
* **deps**: Bump assert_cmd from 2.0.14 to 2.0.15 (#20936) [18d1ce8](https://github.com/answerbook/vector/commit/18d1ce8de8c0ac06c737621260e8f21894b25971) - GitHub
* **deps**: Bump assert_cmd from 2.0.15 to 2.0.16 (#21034) [e99ed52](https://github.com/answerbook/vector/commit/e99ed52449bae79e095d1857d7a06e18a5446fc9) - GitHub
* **deps**: Bump async-compression from 0.4.11 to 0.4.12 (#20897) [b24e9ef](https://github.com/answerbook/vector/commit/b24e9efa12cf8550d96bf37a86c2d5ae69abf805) - GitHub
* **deps**: Bump async-trait from 0.1.80 to 0.1.81 (#20803) [e3a2abd](https://github.com/answerbook/vector/commit/e3a2abde44a0999f4b9c20ea811f14f0c9b9a71d) - GitHub
* **deps**: Bump async-trait from 0.1.81 to 0.1.82 (#21197) [944217c](https://github.com/answerbook/vector/commit/944217c137ff14724850d73fcf08b0e3838d4b0e) - GitHub
* **deps**: Bump aws-smithy-runtime-api from 1.7.1 to 1.7.2 in the aws group (#21033) [645804a](https://github.com/answerbook/vector/commit/645804a72438197c8e8ccd02ad29e4024f9a59ce) - GitHub
* **deps**: Bump aws-types from 1.3.1 to 1.3.2 in the aws group (#20702) [7f4a4c2](https://github.com/answerbook/vector/commit/7f4a4c2436f75b601c42f694a8d8f3086d994e52) - GitHub
* **deps**: Bump aws-types from 1.3.2 to 1.3.3 in the aws group (#20894) [16884c3](https://github.com/answerbook/vector/commit/16884c394f73e6d6b12b2ce3b73f26c3e496258d) - GitHub
* **deps**: Bump bstr from 1.9.1 to 1.10.0 (#20937) [cf8eecf](https://github.com/answerbook/vector/commit/cf8eecfa9db313da44fc09e1bf2b6fad0ca4d6f6) - GitHub
* **deps**: Bump bytes from 1.6.0 to 1.6.1 (#20855) [12f3741](https://github.com/answerbook/vector/commit/12f37413c5aab79f24a912ef5cc28396123b1cb8) - GitHub
* **deps**: Bump bytes from 1.6.1 to 1.7.1 (#20987) [e78af3b](https://github.com/answerbook/vector/commit/e78af3b7767ba010cdcec988ea7e3c7da3ebed32) - GitHub
* **deps**: Bump cargo_toml from 0.20.2 to 0.20.3 (#20695) [b99ffcb](https://github.com/answerbook/vector/commit/b99ffcba01d6d6997d92e13bf0e65613d5ae0c92) - GitHub
* **deps**: Bump cargo_toml from 0.20.3 to 0.20.4 (#20899) [3a17206](https://github.com/answerbook/vector/commit/3a172066c8e30438308e5bb56f2abd6566d5cdba) - GitHub
* **deps**: Bump chrono from 0.4.37 to 0.4.38 (#20309) [886f4e1](https://github.com/answerbook/vector/commit/886f4e1f5978ef652c8dbca9981c73f566efd0f8) - GitHub
* **deps**: Bump clap_complete from 4.5.13 to 4.5.14 in the clap group (#21046) [e595dcc](https://github.com/answerbook/vector/commit/e595dccd98d31af403b69e5268d5421b5d541740) - GitHub
* **deps**: Bump clap_complete from 4.5.18 to 4.5.19 in the clap group (#21116) [b262eec](https://github.com/answerbook/vector/commit/b262eec6c5038e6a7c4cb817fa5dd6296c2970bd) - GitHub
* **deps**: Bump clap_complete from 4.5.19 to 4.5.20 in the clap group (#21125) [7774c5f](https://github.com/answerbook/vector/commit/7774c5f8cd110b2a4bca2b4ced8b11e598053d6a) - GitHub
* **deps**: Bump clap_complete from 4.5.20 to 4.5.22 in the clap group (#21130) [fe2cc26](https://github.com/answerbook/vector/commit/fe2cc26a217364d5dd3f8c00289ce45af2446f24) - GitHub
* **deps**: Bump clap_complete from 4.5.22 to 4.5.23 in the clap group (#21138) [c3c0ec0](https://github.com/answerbook/vector/commit/c3c0ec0cf34f4d5defa19458a76e4e69678ee2a2) - GitHub
* **deps**: Bump clap_complete from 4.5.23 to 4.5.24 in the clap group (#21170) [1d896da](https://github.com/answerbook/vector/commit/1d896daa18c2e87c6f92a147f5bb74956a64dc05) - GitHub
* **deps**: Bump clap_complete from 4.5.5 to 4.5.6 in the clap group (#20700) [0cd2710](https://github.com/answerbook/vector/commit/0cd2710f500cd39fb807bfe1451bb9ca4d1b2952) - GitHub
* **deps**: Bump cue to 0.9.1 (#20666) [199f88a](https://github.com/answerbook/vector/commit/199f88ae39f5bbcba392907a063dedd30eb79235) - GitHub
* **deps**: Bump curve25519-dalek from 4.1.1 to 4.1.3 (#20692) [21c3e68](https://github.com/answerbook/vector/commit/21c3e68bfd2406e703b6587ed474e4e08a3c9c0f) - GitHub
* **deps**: Bump dashmap from 5.5.3 to 6.0.0 (#20696) [064c77b](https://github.com/answerbook/vector/commit/064c77bd11632de654207ce3dec8273f936566f7) - GitHub
* **deps**: Bump dashmap from 6.0.0 to 6.0.1 (#20728) [e982f66](https://github.com/answerbook/vector/commit/e982f6679e9d1526d856efa462e728774b52cf34) - GitHub
* **deps**: Bump databend-client from 0.18.3 to 0.19.3 (#20887) [037229e](https://github.com/answerbook/vector/commit/037229e717b6c25acbcd4d485ee13465be1cb073) - GitHub
* **deps**: Bump databend-client from 0.19.3 to 0.19.5 (#20913) [787e1ec](https://github.com/answerbook/vector/commit/787e1ec3a3c5d89e6fb3fd28201b847d873c78f5) - GitHub
* **deps**: Bump databend-client from 0.19.5 to 0.20.0 (#20981) [0ed8caa](https://github.com/answerbook/vector/commit/0ed8caa16f0f235c47365f127ae1219c3ffb3bbf) - GitHub
* **deps**: Bump databend-client from 0.20.0 to 0.20.1 (#21049) [601fdf9](https://github.com/answerbook/vector/commit/601fdf98e71fa1556926f2b8bbbc4ddfd8936080) - GitHub
* **deps**: Bump dunce from 1.0.4 to 1.0.5 (#21001) [16d2300](https://github.com/answerbook/vector/commit/16d2300bd8def42248e49e42643c8c1a604835c8) - GitHub
* **deps**: Bump env_logger from 0.11.3 to 0.11.4 (#20918) [621f843](https://github.com/answerbook/vector/commit/621f843c08b420c4b8ad708ef60c0192bf8d9361) - GitHub
* **deps**: Bump env_logger from 0.11.4 to 0.11.5 (#20935) [d85394f](https://github.com/answerbook/vector/commit/d85394f687742cce6fc29e83f0c39de6f7a6edc6) - GitHub
* **deps**: Bump flate2 from 1.0.30 to 1.0.31 (#21000) [0ae182c](https://github.com/answerbook/vector/commit/0ae182c8221c82a316a51dfb455f68fb7f7fd3c0) - GitHub
* **deps**: Bump flate2 from 1.0.31 to 1.0.32 (#21126) [82fd2a5](https://github.com/answerbook/vector/commit/82fd2a55a68a97eb3a9043ee9273bca6faf61d81) - GitHub
* **deps**: Bump flate2 from 1.0.32 to 1.0.33 (#21151) [ff2e505](https://github.com/answerbook/vector/commit/ff2e505e2bcb46c6561d8e1d95c30a7b6932d70c) - GitHub
* **deps**: Bump h2 from 0.4.5 to 0.4.6 (#21119) [143e017](https://github.com/answerbook/vector/commit/143e01714c2e1011fa00f12b014ecd37c3dd4050) - GitHub
* **deps**: Bump heim from `a66c440` to `4925b53` (#21220) [00ef42b](https://github.com/answerbook/vector/commit/00ef42baf0a22fbbff2d31b514e0012501356bc8) - GitHub
* **deps**: Bump indexmap from 2.4.0 to 2.5.0 (#21190) [42ca811](https://github.com/answerbook/vector/commit/42ca811ac6e06b5536d0ddd0a548e74d906b515a) - GitHub
* **deps**: Bump lapin from 2.3.4 to 2.4.0 (#20870) [22141b7](https://github.com/answerbook/vector/commit/22141b725c125daa5558b7ab5c41d5f5b5a7defa) - GitHub
* **deps**: Bump lapin from 2.4.0 to 2.5.0 (#20956) [742c612](https://github.com/answerbook/vector/commit/742c6121bd431006006d55606fbffd64933bb5ea) - GitHub
* **deps**: Bump libc from 0.2.155 to 0.2.156 (#21093) [af57155](https://github.com/answerbook/vector/commit/af57155653520b6248a07164858a30897ed868a9) - GitHub
* **deps**: Bump libc from 0.2.156 to 0.2.157 (#21110) [73f5b97](https://github.com/answerbook/vector/commit/73f5b97cf77d8b32a7b44a6dff59b8ab1db95bfd) - GitHub
* **deps**: Bump libc from 0.2.157 to 0.2.158 (#21118) [0f396ac](https://github.com/answerbook/vector/commit/0f396ac784999deb7ae18bd844447bf8ee4b7a53) - GitHub
* **deps**: Bump log from 0.4.21 to 0.4.22 (#20751) [0c1d3b1](https://github.com/answerbook/vector/commit/0c1d3b1ca1250e73237ba7b9e20218c2e53cdfad) - GitHub
* **deps**: Bump lru from 0.12.3 to 0.12.4 (#20975) [65c7287](https://github.com/answerbook/vector/commit/65c7287d1886b8cfdd13bd1b736b5c2c8d2ff8ae) - GitHub
* **deps**: Bump memchr from 2.7.2 to 2.7.4 (#20673) [4918991](https://github.com/answerbook/vector/commit/4918991f9711f3f7587c35768083efe2c3f44f37) - GitHub
* **deps**: Bump metrics from 0.21.1 to 0.22.0 (#19463) [b2aea48](https://github.com/answerbook/vector/commit/b2aea48a374259fd289f3ee3bc9a23eb0446b025) - GitHub
* **deps**: Bump micromatch from 4.0.4 to 4.0.8 in /website (#21156) [e3d0ebf](https://github.com/answerbook/vector/commit/e3d0ebfca572f7d9b70f1944db593d5c735ef47e) - GitHub
* **deps**: Bump mlua from 0.9.8 to 0.9.9 (#20693) [52759aa](https://github.com/answerbook/vector/commit/52759aa7bf9ec0785e186b978adcb0a6103cc70a) - GitHub
* **deps**: Bump ndarray from 0.15.6 to 0.16.0 (#21002) [aa248cb](https://github.com/answerbook/vector/commit/aa248cb268a66aef0ffd5917ce1d07d6918bc967) - GitHub
* **deps**: Bump ndarray from 0.16.0 to 0.16.1 (#21080) [c86bdcc](https://github.com/answerbook/vector/commit/c86bdcc40ce7fadd92b2eb59db6c4f65788b315f) - GitHub
* **deps**: Bump ndarray-stats from 0.5.1 to 0.6.0 (#21177) [0cd16eb](https://github.com/answerbook/vector/commit/0cd16eb7e52ac73166af771ea746ef1fe599aa08) - GitHub
* **deps**: Bump nkeys from 0.4.1 to 0.4.2 (#20898) [ab497bf](https://github.com/answerbook/vector/commit/ab497bf5b9bfbd43d976c14c0b5cb2872e725396) - GitHub
* **deps**: Bump nkeys from 0.4.2 to 0.4.3 (#20938) [c621470](https://github.com/answerbook/vector/commit/c621470b0315ac238aec3a8e4a2692ee9e15ecba) - GitHub
* **deps**: Bump num_enum from 0.7.2 to 0.7.3 (#20963) [2640247](https://github.com/answerbook/vector/commit/264024798b0deca2bdc9dca07fc760c708ace940) - GitHub
* **deps**: Bump openssl from 0.10.64 to 0.10.66 (#20896) [cd60697](https://github.com/answerbook/vector/commit/cd6069795de7d735b366b52862b4d639a68223b8) - GitHub
* **deps**: Bump openssl-src from 300.3.1+3.3.1 to 300.3.2+3.3.2 (#21205) [0cb5cf4](https://github.com/answerbook/vector/commit/0cb5cf41d689d675d47568a004eb9813d7f0ac22) - GitHub
* **deps**: Bump ordered-float from 4.2.0 to 4.2.1 (#20758) [a6f82fb](https://github.com/answerbook/vector/commit/a6f82fb740bc3d763581b906df6cfc777c4c338b) - GitHub
* **deps**: Bump ordered-float from 4.2.1 to 4.2.2 (#20962) [a358576](https://github.com/answerbook/vector/commit/a358576f968e4b363f52a54aed0e43d6b0b744f4) - GitHub
* **deps**: Bump proc-macro2 from 1.0.85 to 1.0.86 (#20704) [9ac371d](https://github.com/answerbook/vector/commit/9ac371d8543c1dc80db1de4c509a767b78783c64) - GitHub
* **deps**: Bump proptest from 1.4.0 to 1.5.0 (#20720) [a6f4586](https://github.com/answerbook/vector/commit/a6f45862049def7f4ab52c97bdfb3a9ab28a0e47) - GitHub
* **deps**: Bump quote from 1.0.36 to 1.0.37 (#21139) [f6d8f72](https://github.com/answerbook/vector/commit/f6d8f72ba5f65984d74f7c462e19b9cf20e3019a) - GitHub
* **deps**: Bump ratatui from 0.26.3 to 0.27.0 (#20730) [84d87f4](https://github.com/answerbook/vector/commit/84d87f4041343cf9f77bb26abf3ed93ce19f0964) - GitHub
* **deps**: Bump regex from 1.10.5 to 1.10.6 (#21003) [9ad96a3](https://github.com/answerbook/vector/commit/9ad96a358ce65fe2ace7d91a6ae3d05ea247d5a0) - GitHub
* **deps**: Bump rkyv from 0.7.44 to 0.7.45 (#21117) [2862006](https://github.com/answerbook/vector/commit/286200632ed2acf451d662dda84e290e30416530) - GitHub
* **deps**: Bump roaring from 0.10.5 to 0.10.6 (#20781) [81211ba](https://github.com/answerbook/vector/commit/81211bab72a607d83ee9b494e8628e3764d239a4) - GitHub
* **deps**: Bump rstest from 0.21.0 to 0.22.0 (#21004) [1beda05](https://github.com/answerbook/vector/commit/1beda059ce8b55cc37fdc6d2370ba4949d20e205) - GitHub
* **deps**: Bump serde from 1.0.203 to 1.0.204 (#20804) [ecf3762](https://github.com/answerbook/vector/commit/ecf3762bb292f12199656b9e2700d872fb2bf691) - GitHub
* **deps**: Bump serde from 1.0.204 to 1.0.205 (#21022) [1111d0c](https://github.com/answerbook/vector/commit/1111d0c49fb050d759d2a3bbd9ffc62ee67eef2a) - GitHub
* **deps**: Bump serde from 1.0.205 to 1.0.206 (#21047) [9c6275d](https://github.com/answerbook/vector/commit/9c6275d0eda069d34a45a8a372b96d89c4f3c35e) - GitHub
* **deps**: Bump serde from 1.0.206 to 1.0.209 (#21149) [f87fc24](https://github.com/answerbook/vector/commit/f87fc24870dff3566c8b2db998bd18ff6fc17db8) - GitHub
* **deps**: Bump serde_bytes from 0.11.14 to 0.11.15 (#20729) [e1c0801](https://github.com/answerbook/vector/commit/e1c08014fc5decec32ff8f26e51423dd11491d91) - GitHub
* **deps**: Bump serde_json from 1.0.117 to 1.0.120 (#20766) [be6b42b](https://github.com/answerbook/vector/commit/be6b42ba1d4fe86b12b1f395747188b2b786b0fd) - GitHub
* **deps**: Bump serde_json from 1.0.120 to 1.0.121 (#20957) [8bf5d49](https://github.com/answerbook/vector/commit/8bf5d494f31daa1456a54d613c5dc9f5d5bd8924) - GitHub
* **deps**: Bump serde_json from 1.0.121 to 1.0.125 (#21092) [ae3696f](https://github.com/answerbook/vector/commit/ae3696f8ec4677970b2e0e023a56a8a28df44c27) - GitHub
* **deps**: Bump serde_json from 1.0.125 to 1.0.127 (#21150) [bac72a8](https://github.com/answerbook/vector/commit/bac72a8821a17a0688a75bbbb556b122fb4d6b79) - GitHub
* **deps**: Bump serde_with from 3.8.1 to 3.8.2 (#20760) [34e35c6](https://github.com/answerbook/vector/commit/34e35c6633a6031bce7d440d6a9646d4332fadd2) - GitHub
* **deps**: Bump serde_with from 3.8.2 to 3.8.3 (#20795) [4307dad](https://github.com/answerbook/vector/commit/4307dad3b567373116d13bd5ee5330e7bde23b2d) - GitHub
* **deps**: Bump serde_with from 3.8.3 to 3.9.0 (#20857) [4d6e44f](https://github.com/answerbook/vector/commit/4d6e44f28285a44f50ecefed949dd7c4fd8a86dc) - GitHub
* **deps**: Bump syn from 2.0.66 to 2.0.70 (#20826) [fadf0a9](https://github.com/answerbook/vector/commit/fadf0a903d35102abbfa24ab40a0e0299c41d952) - GitHub
* **deps**: Bump syn from 2.0.70 to 2.0.72 (#20895) [e89661c](https://github.com/answerbook/vector/commit/e89661ce9866b2f2ad5d80303144db1abdccd5bc) - GitHub
* **deps**: Bump syn from 2.0.72 to 2.0.74 (#21051) [b859bf4](https://github.com/answerbook/vector/commit/b859bf4ee86c8cd55d08efa43889339308e90f55) - GitHub
* **deps**: Bump syn from 2.0.74 to 2.0.75 (#21108) [1549cf0](https://github.com/answerbook/vector/commit/1549cf04e3b560c89885332b6e6fbfd87386eca4) - GitHub
* **deps**: Bump tempfile from 3.10.1 to 3.12.0 (#21016) [87e6636](https://github.com/answerbook/vector/commit/87e6636f404dc3d001a3fe90638fc2d4ad909224) - GitHub
* **deps**: Bump the aws group across 1 directory with 2 updates (#21091) [41901f6](https://github.com/answerbook/vector/commit/41901f626baa3575b884b3cfa48ebb58c2601369) - GitHub
* **deps**: Bump the aws group across 1 directory with 4 updates (#20832) [9260083](https://github.com/answerbook/vector/commit/9260083004431f882400faca4fa355a26acbbc0f) - GitHub
* **deps**: Bump the aws group across 1 directory with 4 updates (#21187) [6128d76](https://github.com/answerbook/vector/commit/6128d76e4b341c4ea8866cdbbfe89fa7b6c5acda) - GitHub
* **deps**: Bump the clap group across 1 directory with 2 updates (#20841) [d1799d8](https://github.com/answerbook/vector/commit/d1799d8f3ed6cbe7e9ef17c28883a2907319ff74) - GitHub
* **deps**: Bump the clap group across 1 directory with 2 updates (#21032) [34b502d](https://github.com/answerbook/vector/commit/34b502d2279d571255482d104edef26cab5758b6) - GitHub
* **deps**: Bump the clap group across 1 directory with 2 updates (#21101) [4909c52](https://github.com/answerbook/vector/commit/4909c522edc90a782580a0d302ec437a507380ba) - GitHub
* **deps**: Bump the clap group with 2 updates (#20757) [29d8dbe](https://github.com/answerbook/vector/commit/29d8dbef8bad0531713981517a6a50600c3461df) - GitHub
* **deps**: Bump the clap group with 2 updates (#20917) [84692d4](https://github.com/answerbook/vector/commit/84692d489fafa83b994be8326a281d7cd176c6de) - GitHub
* **deps**: Bump the graphql group with 2 updates (#20854) [9771a45](https://github.com/answerbook/vector/commit/9771a45ba69418612910c4e58239c12d444fae4d) - GitHub
* **deps**: Bump the tonic group with 2 updates (#19837) [e90cece](https://github.com/answerbook/vector/commit/e90cecef2598ae267ed6f645356da6f8a494a827) - GitHub
* **deps**: Bump thiserror from 1.0.61 to 1.0.63 (#20880) [4ec2b4c](https://github.com/answerbook/vector/commit/4ec2b4c2d5e5e363ee687b0914d56d8083ebb7e8) - GitHub
* **deps**: Bump tikv-jemallocator from 0.5.4 to 0.6.0 (#20856) [535bd0d](https://github.com/answerbook/vector/commit/535bd0ded17f4ac21a2a06918471010ebe46a77d) - GitHub
* **deps**: Bump tokio from 1.38.0 to 1.39.1 (#20916) [930006c](https://github.com/answerbook/vector/commit/930006c26e2989182feb939772a843b3592e25da) - GitHub
* **deps**: Bump tokio from 1.39.1 to 1.39.2 (#20953) [f56a1e0](https://github.com/answerbook/vector/commit/f56a1e086121a445e56a2514e971fa7f8a176639) - GitHub
* **deps**: Bump tokio from 1.39.2 to 1.39.3 (#21109) [bf93758](https://github.com/answerbook/vector/commit/bf9375816b4a0e6c694caf49df44432dd8313c5f) - GitHub
* **deps**: Bump tokio from 1.39.3 to 1.40.0 (#21189) [17fba1a](https://github.com/answerbook/vector/commit/17fba1a980e233997654a38b8f9936e9fde0bba1) - GitHub
* **deps**: Bump tokio-postgres from 0.7.10 to 0.7.11 (#20900) [fcafdd2](https://github.com/answerbook/vector/commit/fcafdd2c4e0bd847796bd342059c8840520de132) - GitHub
* **deps**: Bump toml from 0.8.14 to 0.8.15 (#20881) [4610062](https://github.com/answerbook/vector/commit/46100625a7007f6c791be5bb5fd2aeee6cb59f8f) - GitHub
* **deps**: Bump toml from 0.8.15 to 0.8.16 (#20934) [1b584bd](https://github.com/answerbook/vector/commit/1b584bda5fdf03b48212f6feaffbef0cf6a21734) - GitHub
* **deps**: Bump toml from 0.8.16 to 0.8.19 (#20982) [84666a4](https://github.com/answerbook/vector/commit/84666a4a41302e665651db68c7d2ce866b96d78c) - GitHub
* **deps**: Bump typetag from 0.2.16 to 0.2.17 (#20954) [acd34aa](https://github.com/answerbook/vector/commit/acd34aaa9400b367ac8dc1769688b433d045e374) - GitHub
* **deps**: Bump typetag from 0.2.17 to 0.2.18 (#21050) [94fd34f](https://github.com/answerbook/vector/commit/94fd34fefe21a0522f33e149504a96eea66b5874) - GitHub
* **deps**: Bump url from 2.5.1 to 2.5.2 (#20694) [b2d3fe1](https://github.com/answerbook/vector/commit/b2d3fe18d13bc4c5160e41810f87da09308967f6) - GitHub
* **deps**: Bump uuid from 1.8.0 to 1.9.1 (#20727) [0a72db7](https://github.com/answerbook/vector/commit/0a72db7b74f935f4040d18273d4a78dedc25c5a1) - GitHub
* **deps**: Bump uuid from 1.9.1 to 1.10.0 (#20842) [a4124ce](https://github.com/answerbook/vector/commit/a4124ce1354b1476ae9cd60cd5cf2004a3ce6f63) - GitHub
* **deps**: bump VRL to 0.16.1 (#20821) [43cfd5a](https://github.com/answerbook/vector/commit/43cfd5a316b2a392241270aeb7bf14186f88b066) - GitHub
* **deps**: Bump VRL to 0.18.0 (#21214) [cf8f94b](https://github.com/answerbook/vector/commit/cf8f94b7d4b2d126650210f8bab62a6dba242c9a) - GitHub
* **deps**: Bump VRL to the latest ref (#21171) [5060aa5](https://github.com/answerbook/vector/commit/5060aa5c223643d5198825896f62109353eb3e2f) - GitHub
* **deps**: Bump wasm-bindgen from 0.2.92 to 0.2.93 (#21060) [c28e947](https://github.com/answerbook/vector/commit/c28e947b7781cde74a54658e9ae079020ecc2e30) - GitHub
* **deps**: Bump wiremock from 0.5.22 to 0.6.1 (#20908) [7685b6f](https://github.com/answerbook/vector/commit/7685b6f1306350d610370dbd7c8cc36034102454) - GitHub
* **deps**: Group dependabot tower updates (#21096) [f371bc2](https://github.com/answerbook/vector/commit/f371bc2ce5a806a4dc9fa8ba573594dabb2a60ea) - GitHub
* **deps**: Regenerate Cargo.lock (#21083) [ac4e194](https://github.com/answerbook/vector/commit/ac4e1944fc29bf7d3ab25af029b8fdfdba0dc910) - GitHub
* **deps**: Update fork of tokio-util to 0.7.11 (latest) (#21066) [fb9e6d2](https://github.com/answerbook/vector/commit/fb9e6d26e743ae73cf43aeff5a53f375aca9989d) - GitHub
* **deps**: Upgrade VRL to v0.17.0 (#20922) [70eb470](https://github.com/answerbook/vector/commit/70eb470b647b703472bbdd26b173203f6855d5fc) - GitHub
* **dev**: Add a note to CONTRIBUTING.md about running clippy (#20775) [361a32b](https://github.com/answerbook/vector/commit/361a32b16de67c7b67587fa5d2e0c8d2ca8b6667) - GitHub
* **docs**: Clarify behavior of sources when any sink has acks enabled (#20910) [6a77cc3](https://github.com/answerbook/vector/commit/6a77cc3ff72f9016a0816f7a347b1bb8eb056742) - GitHub
* **docs**: Fix link to unit tests (#20819) [28b7e35](https://github.com/answerbook/vector/commit/28b7e35bf9cbb105d1fecc18fc852f78b78f0c5b) - GitHub
* **docs**: Regenerate component docs (#20789) [bfcf95b](https://github.com/answerbook/vector/commit/bfcf95b4eeeb67cd50364d7e684903419c7de9b8) - GitHub
* **docs**: Run `cue fmt` (#20777) [d778c6f](https://github.com/answerbook/vector/commit/d778c6f016f862a2d05ba8fda435d0c9cceb2715) - GitHub
* **elasticsearch**: make multiple modules public (#20683) [146de92](https://github.com/answerbook/vector/commit/146de92dfd90d20451308d77f4da0c8afc3a4e45) - GitHub
* Increases default warmup duration for Regression Detector jobs (#20828) [0953a90](https://github.com/answerbook/vector/commit/0953a90118474e3affd09dc91f886e32ddcf643b) - GitHub
* **relasing**: Regenerate k8s manifsts for chart 0.35.0 (#21199) [beceef4](https://github.com/answerbook/vector/commit/beceef4a964a77acc3064c064eb27d265261c17d) - GitHub
* **releasing**: Add known issue around `source_event_id` for v0.41.0 (#21255) [2970a9d](https://github.com/answerbook/vector/commit/2970a9dec59a6b87bb31b988715eba4f208795f9) - Jesse Szwedko
* **releasing**: Bump k8s manifests to v0.34.0 of the chart (#20686) [9084efa](https://github.com/answerbook/vector/commit/9084efa434885d2618656713390e12c9a30bf3d6) - GitHub
* **releasing**: Clarify the removal of the `enterprise` configuration in 0.39.0 (#20772) [73dd278](https://github.com/answerbook/vector/commit/73dd278b89b3835a3a393065db4b51fed94d1cce) - GitHub
* **releasing**: Prepare v0.39.0 release [f345942](https://github.com/answerbook/vector/commit/f3459425fbb73ff835c08faeb14d1c1200170f34) - Jesse Szwedko
* **releasing**: Prepare v0.40.0 release [4f4b4ea](https://github.com/answerbook/vector/commit/4f4b4ea3231e418e4d64665e7bdfb7d2667a468d) - Jesse Szwedko
* **releasing**: Prepare v0.40.1 release [ec3a82a](https://github.com/answerbook/vector/commit/ec3a82aa68e331e1707baf82211db12e6042fb7c) - Jesse Szwedko
* **releasing**: Prepare v0.40.2 release [272db76](https://github.com/answerbook/vector/commit/272db76f3ab974cfb99afd481dc9b302eeec93bf) - Jesse Szwedko
* **releasing**: Prepare v0.41.0 release [97952b5](https://github.com/answerbook/vector/commit/97952b50dba345068b43928ede174865e5962782) - Jesse Szwedko
* **releasing**: Prepare v0.41.1 release [745babd](https://github.com/answerbook/vector/commit/745babdc7dfa183feaab26ce915c15b10cd5bb6b) - Jesse Szwedko
* **releasing**: Update the base image used for x86_64 artifacts to Ubuntu 16.04 (#20765) [951d726](https://github.com/answerbook/vector/commit/951d726a70df7a2045524e7ef9517fdc6423d9f2) - GitHub
* Run cue fmt with 0.9.0 (#20678) [b3276b4](https://github.com/answerbook/vector/commit/b3276b4cc73dee6d3854469562f1b1fcf15a419c) - GitHub
* **tap**: surface send error (#21056) [38fdd46](https://github.com/answerbook/vector/commit/38fdd46f434e6688665bb436a2d02d195ac0d280) - GitHub
* **transforms**: Add conversion helpers for `DatadogSearchConfig` (#21181) [360780d](https://github.com/answerbook/vector/commit/360780dcaa1d79e115048864917bf30512012b9b) - GitHub


### Code Refactoring

* **tap**: Refactor internal logic for vector tap into lib (#21200) [695bb47](https://github.com/answerbook/vector/commit/695bb476d6116ba65d57a77aa0ac655af76f063a) - GitHub
* **tap**: Refactor vector tap into library (#20850) [be37a33](https://github.com/answerbook/vector/commit/be37a33a2e6294051e86ad50674e2857fb188985) - GitHub
* **tap**: tap filters pass around rust closures [7be3b97](https://github.com/answerbook/vector/commit/7be3b97de048e313c93f5f0d1b60a382a57459d0) - Dan Hable [LOG-20746](https://logdna.atlassian.net/browse/LOG-20746)


### Features

* **codec,sources**: influxdb line protcol decoder (#19637) [3fbb66c](https://github.com/answerbook/vector/commit/3fbb66c8c94f9d95986e4a695a21681736025c93) - GitHub
* **events**: Add internal event id to event metadata (#21074) [06ad674](https://github.com/answerbook/vector/commit/06ad674ff4fbf2df82c50b787c45316c5b5c8d50) - GitHub
* **honeycomb sink**: endpoint is now configurable (#21147) [d76330b](https://github.com/answerbook/vector/commit/d76330b8efed0ad9278f48f0430ba644c04dbd73) - GitHub
* **loki sink**: add support for structured metadata (#20576) [d174d55](https://github.com/answerbook/vector/commit/d174d55fadaaee8e8a55223f43a6000e2814382d) - GitHub
* **new sink**: add greptime log sink (#20812) [35e82bd](https://github.com/answerbook/vector/commit/35e82bd9d9983097e5af75a98300b74b4fc433e9) - GitHub
* **new sink**: Add possibility to use nats jetstream in nats sink (#20834) [56bd0dd](https://github.com/answerbook/vector/commit/56bd0dd57e76c4bb0f4ad43370e44ed74f962a47) - GitHub
* **playground**: redesign vrl playground and make responsive (#21078) [eb2d786](https://github.com/answerbook/vector/commit/eb2d786617ad8afdf799323fe3f0ae7cddd25936) - GitHub
* **source**: add new `static_metrics` source (#20889) [789848f](https://github.com/answerbook/vector/commit/789848fdb9137a9d291f4254de62a1cb58518e0f) - GitHub
* **statsd source**: add option to disable key sanitization (#20717) [e144ac6](https://github.com/answerbook/vector/commit/e144ac674973649dd786ec2be4b4bad4bea17163) - GitHub
* **tap**: Add duration flag to vector tap (#20815) [1579627](https://github.com/answerbook/vector/commit/1579627ee3c63347fa68fe8ab3e0105bbe6ff3d9) - GitHub
* **tap**: Implement async output channel type for vector-tap lib (#20876) [ef4f175](https://github.com/answerbook/vector/commit/ef4f1752b40af6a405df718320feff55e042e0b4) - GitHub


### Miscellaneous

* Merge pull request #621 from answerbook/feature/LOG-20746 [b041855](https://github.com/answerbook/vector/commit/b04185568e31bf2be15144a9e7b449bcf4ca1ce1) - GitHub [LOG-20746](https://logdna.atlassian.net/browse/LOG-20746)
* Merge remote-tracking branch 'dhable/upstream-0.41.1' into dhable/LOG-20746 [d03c379](https://github.com/answerbook/vector/commit/d03c3794da4f2c075cdaf07307ef1750ddafb4d7) - Dan Hable [LOG-20746](https://logdna.atlassian.net/browse/LOG-20746)
* <fix>: Correct snafu features (#21007) [1c6db37](https://github.com/answerbook/vector/commit/1c6db376890fb07b4ed58f6a19585c0ff383a8fe) - GitHub
* fix!(http_server source): only single query param when namespace is enabled (#20920) [12b684e](https://github.com/answerbook/vector/commit/12b684e48066412a897c0b5dde2bd55856dc16c0) - GitHub
* fix!(codecs): Use '\0' delimiter as default stream decode framer (#20778) [21548fc](https://github.com/answerbook/vector/commit/21548fce90d2c0451060237695e54d0b43bcbea7) - GitHub
* fix(codecs) csv documentation defaults as ascii_char (#20498) [5a10aa2](https://github.com/answerbook/vector/commit/5a10aa2b0556563d11244b04829a72d86a8e338b) - GitHub
* Add trailing slash to aws endpoint examples (#20774) [e1ca0f1](https://github.com/answerbook/vector/commit/e1ca0f15bfa535eafaca3debdc3b1c77613684e9) - GitHub
* chore(smp upgrade) - Upgrade SMP to latest release (#20713) [5e7981c](https://github.com/answerbook/vector/commit/5e7981ce4362e3491283c2ac9fcd19147fda69cb) - GitHub
* add docs for the `parse_influxdb` vrl function (#21105) [fbcda67](https://github.com/answerbook/vector/commit/fbcda67644757f4d63ce829c435af2031b2911a8) - GitHub
* add documentation for the new `lossy` option of the `parse_json` vrl function (#21076) [7656ce0](https://github.com/answerbook/vector/commit/7656ce071ea691ad0cc556ce1a161ef2f8c2f2b3) - GitHub
* **AWS config**: Make SQS client timeout settings configurable (#20120) [abf2b7d](https://github.com/answerbook/vector/commit/abf2b7d88e019833d5f5f6a18c9c3e1b98c2db59) - GitHub
* **config**: Remove the watcher's coupling to SIGHUP / prepare for automatic Windows config reload (#20989) [c3cd232](https://github.com/answerbook/vector/commit/c3cd2325e180907c1c9e1e8547547e0e9b240df8) - GitHub
* **datadog_agent source**: enable zstd decompression (#20732) [def8b7f](https://github.com/answerbook/vector/commit/def8b7f251ec60acc4ec673f5c6acc4065e99e18) - GitHub
* **demo_logs source**: Add `host` to emitted logs (#20754) [3de6f0b](https://github.com/answerbook/vector/commit/3de6f0b8175c8a08da1974d6b6e9634075244a06) - GitHub
* **enrichment_tables**: Changes to support GeoIP Anonymous IP database (#20946) [7f286c4](https://github.com/answerbook/vector/commit/7f286c4798d9313b2602a4a94e8f0932c3025383) - GitHub
* **external docs**: add comment to the configuration because it conflicts with what is mentioned in the helm deployment docs (#19523) [8ffc0db](https://github.com/answerbook/vector/commit/8ffc0db2e58901e8f2cc791baf5e1dd9219ba290) - GitHub
* **external docs**: Emphasize the $$ rather than $ in config file. (#20991) [210ff09](https://github.com/answerbook/vector/commit/210ff0925d391213556f07bf6ce621967f0368ca) - GitHub
* **external docs**: Fix the wrong log-namespace key. (#21006) [8a77d7c](https://github.com/answerbook/vector/commit/8a77d7cea16db8cf527fe992cc1495da3ef7c952) - GitHub
* **external docs**: Update ChangLog doc (#21029) [7be4be6](https://github.com/answerbook/vector/commit/7be4be622fb1e4308266d90aa1c244be636ffb8a) - GitHub
* Fix link to commit scopes in CONTRIBUTING.md (#20847) [77ea00f](https://github.com/answerbook/vector/commit/77ea00f558d6b69fab9d13457d62d8f70c481a5a) - GitHub
* Fix link to VRL playground in SUPPORT.md (#20846) [fd35e1c](https://github.com/answerbook/vector/commit/fd35e1c54c33e297cdfd48ecfeae344947f303fb) - GitHub
* **internal_metrics**: reuse `BytesReceived` event for internal metrics (#20977) [70e61fc](https://github.com/answerbook/vector/commit/70e61fc344b9db2f191cfba9ad58ee08e79d30b4) - GitHub
* **kafka sink**: Allow OIDC usage for Kafka (#21103) [97558d3](https://github.com/answerbook/vector/commit/97558d3d59aecb6bc71c82a18953c3537233c934) - GitHub
* **kafka sink**: update service to set Errored status on events (#21036) [dc0b408](https://github.com/answerbook/vector/commit/dc0b4087095b4968cca0201e233919de8cff9918) - GitHub
* **remap transform**: Fix example syntax (#20783) [270bdc5](https://github.com/answerbook/vector/commit/270bdc5a715a25e3a7e687a4c50e698e2baac367) - GitHub
* **vrl**: Add `unflatten` vrl function documentation (#21142) [c23653f](https://github.com/answerbook/vector/commit/c23653ff1650c15892147feb4ae4986903c2d77b) - GitHub
* **vrl**: add casing functions (https://github.com/vectordotdev/vrl/… (#21021) [7560931](https://github.com/answerbook/vector/commit/75609314dd61b6923550411f60635f443a854f98) - GitHub
* **vrl**: Update documentation for community_id to mention ICMP (#20677) [e52f312](https://github.com/answerbook/vector/commit/e52f312d208dc1b2f49127a8a5786cdf9f9b5912) - GitHub


### Tests

* **codecs**: add missing assert in codec decoding tests (#20998) [aab836b](https://github.com/answerbook/vector/commit/aab836b502d3cc23594655d0d72224768122905a) - GitHub


### **BREAKING CHANGES**

* **releasing:** Update the base image used for x86_64 artifacts to Ubuntu 16.04 (#20765)

## [7.0.1](https://github.com/answerbook/vector/compare/v7.0.0...v7.0.1) (2024-10-07)


### Bug Fixes

* **alloc-tracking**: Check group id bounds correctly [fdd92cf](https://github.com/answerbook/vector/commit/fdd92cf75dc31531b9d445245c6c07b714335c23) - Dan Hable [LOG-20705](https://logdna.atlassian.net/browse/LOG-20705)

# [7.0.0](https://github.com/answerbook/vector/compare/v6.0.0...v7.0.0) (2024-10-07)


### Bug Fixes

* **datadog sinks**: Compute proper validate endpoint (#20644) [9be2eeb](https://github.com/answerbook/vector/commit/9be2eeb111e8de36937770c55b81c13b1dd7b681) - GitHub
* **datadog sinks**: Fix regex typo in computation of API endpoint (#20656) [30901f0](https://github.com/answerbook/vector/commit/30901f08ae5b46a04af94c02a68aebc285638035) - GitHub
* **external docs**: Fix parsing-csv-logs-with-lua.md example for postgres > 13 (#20513) [006844e](https://github.com/answerbook/vector/commit/006844e661995fb0fce6e21f7e2b0ffae131bff5) - GitHub
* **kafka source**: Reorder message consume loop to avoid memory growth (#20467) [8301101](https://github.com/answerbook/vector/commit/8301101968a3b3f4cf6c42afb25b2f6d49ded93e) - GitHub
* **reduce**: correct feature gate (#20554) [14d8f31](https://github.com/answerbook/vector/commit/14d8f31bf288fa12bf464278d6917b0d651181c2) - GitHub
* **website**: Update action for previews (#20661) [7714c68](https://github.com/answerbook/vector/commit/7714c684e35c64505101c1dc0afd333b09a7e663) - GitHub


### Chores

* bump rust version to 1.78 (#20624) [58114f1](https://github.com/answerbook/vector/commit/58114f1345c1fe33807d14f0c8f8ae9087919d19) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.31.0 to 1.32.0 (#20516) [879150e](https://github.com/answerbook/vector/commit/879150ed348d9071b000497c0e9e84e357cbe47f) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.32.0 to 1.32.1 (#20549) [12160b1](https://github.com/answerbook/vector/commit/12160b10e033e193e6a80e7ac1e1ef7821554a26) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.32.1 to 1.32.2 (#20578) [843f186](https://github.com/answerbook/vector/commit/843f1864343463630a1a6108e4c53bdece0e115f) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.32.2 to 1.33.0 (#20667) [bcbcd40](https://github.com/answerbook/vector/commit/bcbcd402c1c48b848f397f9ee1d878c25c7ec2f7) - GitHub
* **ci**: Bump docker/build-push-action from 5.3.0 to 5.4.0 (#20632) [9bbb991](https://github.com/answerbook/vector/commit/9bbb991dcc6d47aa2d3bbd06bcba745f1803d9f9) - GitHub
* **ci**: Bump Ruby to v3 [73da9bb](https://github.com/answerbook/vector/commit/73da9bbf89d49ce7abc5ad8c8526c7679f289dae) - Jesse Szwedko
* **ci**: Bump timeout for publishing new environment (#20646) [d9c16a2](https://github.com/answerbook/vector/commit/d9c16a27bb8ef0ad6abfac662df07d9c895225cf) - GitHub
* **ci**: Fix nextest / rustup dll issue (#20544) [50ef76b](https://github.com/answerbook/vector/commit/50ef76b66633ff06c1b81f826c7f8fcec53ebe52) - GitHub
* **ci**: Replace kafka integration test filter (#20534) [45be7ad](https://github.com/answerbook/vector/commit/45be7ad78fad7edbde5b9fe359f3cd674a85b643) - GitHub
* **ci**: Switch to Confluent docker images since wurstmeister ones disappeared (#20465) [3a115c5](https://github.com/answerbook/vector/commit/3a115c517fe91f4c70eb8211b6dfdd1899adf07c) - GitHub
* **ci**: Update minikube to v1.33.1 (#20672) [0de41f9](https://github.com/answerbook/vector/commit/0de41f9fc018c74d30333a5c0233aab568f14d4f) - GitHub
* **ci**: Update versions of OSes used for testing the RPM package (#20611) [8652efe](https://github.com/answerbook/vector/commit/8652efe35f4f384b16cb1b7779fbac8f037cf043) - GitHub
* **config**: Drop support for missing example configs (#20550) [8b4a3ba](https://github.com/answerbook/vector/commit/8b4a3ba57851eb1d051961e504b9e7bd9b7e0f8a) - GitHub
* **deps**: Allow unicode-3 license (#20647) [b81118a](https://github.com/answerbook/vector/commit/b81118ab581f853791bae9c8d891aeaaa861ee51) - GitHub
* **deps**: Bump Alpine Linux base image to 3.20 (#20668) [99d2035](https://github.com/answerbook/vector/commit/99d203503896cec835ec0b84d27d72cd38d1dc91) - GitHub
* **deps**: Bump anyhow from 1.0.82 to 1.0.83 (#20446) [6d8ca9c](https://github.com/answerbook/vector/commit/6d8ca9c24b70a436348f63d3cb9116b9ac595846) - GitHub
* **deps**: Bump anyhow from 1.0.83 to 1.0.86 (#20520) [b03a8fa](https://github.com/answerbook/vector/commit/b03a8fa93aad614da4f69249dfaf8359dbb96ce4) - GitHub
* **deps**: Bump async-compression from 0.4.10 to 0.4.11 (#20586) [62e1cb3](https://github.com/answerbook/vector/commit/62e1cb30d059d389fdabe502ee74b528aad777b1) - GitHub
* **deps**: Bump async-compression from 0.4.9 to 0.4.10 (#20474) [9d94280](https://github.com/answerbook/vector/commit/9d94280f55fb208e4279f5e341f3838e30649784) - GitHub
* **deps**: Bump aws-sigv4 from 1.2.1 to 1.2.2 in the aws group (#20650) [094d289](https://github.com/answerbook/vector/commit/094d2899de9277b713b3b006a036bc1c512c3d5b) - GitHub
* **deps**: Bump aws-types from 1.2.0 to 1.2.1 in the aws group (#20472) [021f645](https://github.com/answerbook/vector/commit/021f6458b2414846f23efeb480308d7781cc0487) - GitHub
* **deps**: Bump aws-types from 1.3.0 to 1.3.1 in the aws group (#20616) [9575d65](https://github.com/answerbook/vector/commit/9575d653d75cda3ba81c0e9d001d9057c80a3775) - GitHub
* **deps**: Bump bitmask-enum from 2.2.3 to 2.2.4 (#20528) [72d1373](https://github.com/answerbook/vector/commit/72d13735460c1c4bb34ace4f53aa2fc0f7e950db) - GitHub
* **deps**: Bump braces from 3.0.2 to 3.0.3 in /website (#20636) [6c8889e](https://github.com/answerbook/vector/commit/6c8889eba7b313874cd356bb220b437c6b8ba959) - GitHub
* **deps**: Bump cargo-nextest (#20572) [5bd264b](https://github.com/answerbook/vector/commit/5bd264b57a0b87bf7ef1f34950468db7a5418c7d) - GitHub
* **deps**: Bump clap from 4.5.6 to 4.5.7 in the clap group (#20639) [3aeaf45](https://github.com/answerbook/vector/commit/3aeaf456cdaece7c05b8ee2c77b57d86cf15c73e) - GitHub
* **deps**: Bump console-subscriber from 0.2.0 to 0.3.0 (#20641) [90f8a70](https://github.com/answerbook/vector/commit/90f8a70aca6f01ee9c893d2ec3ba382c293bf193) - GitHub
* **deps**: Bump crc32fast from 1.4.0 to 1.4.2 (#20539) [3031e9b](https://github.com/answerbook/vector/commit/3031e9b307b021f27c3930b5e89dd0043479483f) - GitHub
* **deps**: Bump crossbeam-utils from 0.8.19 to 0.8.20 in the crossbeam group (#20518) [eacf1e8](https://github.com/answerbook/vector/commit/eacf1e8ec126cde432d6537b005566e79b9a8cc4) - GitHub
* **deps**: Bump databend-client from 0.17.1 to 0.17.2 (#20450) [b5191f6](https://github.com/answerbook/vector/commit/b5191f695ea1309b27cd0abd6bd1adf9524f1860) - GitHub
* **deps**: Bump databend-client from 0.17.2 to 0.18.1 (#20540) [16e3550](https://github.com/answerbook/vector/commit/16e3550c0a13feb7e92117bb71afc40ec46c7c0c) - GitHub
* **deps**: Bump databend-client from 0.18.1 to 0.18.2 (#20569) [7206fa4](https://github.com/answerbook/vector/commit/7206fa454bb570783f95ae3fb4820adc18a39773) - GitHub
* **deps**: Bump databend-client from 0.18.2 to 0.18.3 (#20585) [8eff9bc](https://github.com/answerbook/vector/commit/8eff9bcdf10bcda7403a9b05b88ffb5159c9ae53) - GitHub
* **deps**: Bump encoding_rs from 0.8.33 to 0.8.34 (#20283) [ceca9a1](https://github.com/answerbook/vector/commit/ceca9a102aa510fbec15f4ad1d56523e1dd51025) - GitHub
* **deps**: Bump enumflags2 from 0.7.9 to 0.7.10 (#20628) [a01c198](https://github.com/answerbook/vector/commit/a01c198f8dae6930ddc55d0471e753203f56ecbd) - GitHub
* **deps**: Bump getrandom from 0.2.14 to 0.2.15 (#20442) [19def61](https://github.com/answerbook/vector/commit/19def61b63aae1dafd9d9f655e8213c3d85fca0d) - GitHub
* **deps**: Bump h2 from 0.4.4 to 0.4.5 (#20527) [58047ad](https://github.com/answerbook/vector/commit/58047ad2c710a20f98a5634f73b5de1a3eea40a6) - GitHub
* **deps**: Bump infer from 0.15.0 to 0.16.0 (#20597) [2451cc0](https://github.com/answerbook/vector/commit/2451cc0eaa2e070426185553474ac670204f9187) - GitHub
* **deps**: Bump itertools from 0.12.1 to 0.13.0 (#20508) [d5c23fe](https://github.com/answerbook/vector/commit/d5c23fe1a0fd09a7e82921bf5065ae2fba83bd67) - GitHub
* **deps**: Bump libc from 0.2.154 to 0.2.155 (#20523) [8645cd9](https://github.com/answerbook/vector/commit/8645cd9eec4eb59f65dced7c8e9f37b3dcc03a5d) - GitHub
* **deps**: Bump mlua from 0.9.7 to 0.9.8 (#20503) [7c745d2](https://github.com/answerbook/vector/commit/7c745d2babc32d52848631b53f77a7a68255d65a) - GitHub
* **deps**: Bump mock_instant from 0.4.0 to 0.5.1 (#20598) [1caa112](https://github.com/answerbook/vector/commit/1caa1122d1b77289034d11f90bd9a51c43b50085) - GitHub
* **deps**: Bump num-traits from 0.2.18 to 0.2.19 (#20433) [f2be33e](https://github.com/answerbook/vector/commit/f2be33ef1604883a56702b3e97fef60eb8a83183) - GitHub
* **deps**: Bump openssl-src from 300.2.3+3.2.1 to 300.3.0+3.3.0 (#20546) [1b57acd](https://github.com/answerbook/vector/commit/1b57acd6bb806a09556a8d5cd3a64709c5f44354) - GitHub
* **deps**: Bump openssl-src from 300.3.0+3.3.0 to 300.3.1+3.3.1 (#20612) [8a73968](https://github.com/answerbook/vector/commit/8a73968ad0bb728e88c50b60d78d4b11c797213d) - GitHub
* **deps**: Bump parking_lot from 0.12.2 to 0.12.3 (#20564) [632ecd7](https://github.com/answerbook/vector/commit/632ecd7553ce5a5bcfed3f8620e9986ad9056590) - GitHub
* **deps**: Bump paste from 1.0.14 to 1.0.15 (#20448) [99446fa](https://github.com/answerbook/vector/commit/99446fa5f44520b89bc487b85cd3a2f2561737cd) - GitHub
* **deps**: Bump prettydiff from 0.6.4 to 0.7.0 (#20460) [13aaea1](https://github.com/answerbook/vector/commit/13aaea12c25db21037a85c2b79095d4a2f0b11b2) - GitHub
* **deps**: Bump proc-macro2 from 1.0.81 to 1.0.82 (#20447) [f5f16cc](https://github.com/answerbook/vector/commit/f5f16cc9a7c0a9c1ba8895b6e054d96a6b1504c4) - GitHub
* **deps**: Bump proc-macro2 from 1.0.83 to 1.0.84 (#20565) [082e80f](https://github.com/answerbook/vector/commit/082e80ffaa166529e057ceed9b5993f8beed207f) - GitHub
* **deps**: Bump proc-macro2 from 1.0.84 to 1.0.85 (#20599) [3da355b](https://github.com/answerbook/vector/commit/3da355b0bde93ed2afa643c53e32b84ac387fd4f) - GitHub
* **deps**: Bump pulsar from 6.1.0 to 6.2.0 (#20458) [e02118a](https://github.com/answerbook/vector/commit/e02118a35901fb27c59176608e0c15a0070825aa) - GitHub
* **deps**: Bump pulsar from 6.2.0 to 6.3.0 (#20524) [2a52d22](https://github.com/answerbook/vector/commit/2a52d2212a91cd001361f918ce19924285ff4dbc) - GitHub
* **deps**: Bump ratatui from 0.26.2 to 0.26.3 (#20538) [16ef571](https://github.com/answerbook/vector/commit/16ef571da08e78a170c588ec6981c6f648a9eec4) - GitHub
* **deps**: Bump regex from 1.10.4 to 1.10.5 (#20629) [60673c7](https://github.com/answerbook/vector/commit/60673c714dcc31e0f44fec00fcfa1a09fe437b8c) - GitHub
* **deps**: Bump roaring from 0.10.4 to 0.10.5 (#20630) [974970e](https://github.com/answerbook/vector/commit/974970e4a62986b56721fb37508c24983935a27e) - GitHub
* **deps**: Bump rstest from 0.19.0 to 0.21.0 (#20596) [a32a022](https://github.com/answerbook/vector/commit/a32a02295d211ac3e57bb597ad33d8d8dc379280) - GitHub
* **deps**: Bump Rust version to 1.79 (#20670) [b11ca5d](https://github.com/answerbook/vector/commit/b11ca5d138292e85bba9e110442ee5f1d7a75abf) - GitHub
* **deps**: Bump ryu from 1.0.17 to 1.0.18 (#20445) [5015eaa](https://github.com/answerbook/vector/commit/5015eaaf5d9be5c6324cfd2a1af006134b70e9ad) - GitHub
* **deps**: Bump semver from 1.0.22 to 1.0.23 (#20449) [9cad948](https://github.com/answerbook/vector/commit/9cad9484ca84b9330ad2ad3eb6eab368fd70ff0b) - GitHub
* **deps**: Bump serde from 1.0.200 to 1.0.201 (#20459) [df26f56](https://github.com/answerbook/vector/commit/df26f56a09607f90aa25dba16aade2e3a5a656e9) - GitHub
* **deps**: Bump serde from 1.0.201 to 1.0.202 (#20502) [ff5af87](https://github.com/answerbook/vector/commit/ff5af874069ee0be90fca949bd51640369507a1a) - GitHub
* **deps**: Bump serde from 1.0.202 to 1.0.203 (#20563) [5b59039](https://github.com/answerbook/vector/commit/5b59039b4a4176bd777a5ef3b37dcd1c8ad20e3f) - GitHub
* **deps**: Bump serde_derive_internals from 0.29.0 to 0.29.1 (#20504) [cddc180](https://github.com/answerbook/vector/commit/cddc1808d1a49e2766770fc06b25d2be6ddc0234) - GitHub
* **deps**: Bump serde_json from 1.0.116 to 1.0.117 (#20461) [d777dc5](https://github.com/answerbook/vector/commit/d777dc55039e58ea9aeedd8781ba75320d0ee044) - GitHub
* **deps**: Bump serde-toml-merge from 0.3.6 to 0.3.7 (#20501) [d1c2aec](https://github.com/answerbook/vector/commit/d1c2aecd47ae90d14a133af03914160366a6e66c) - GitHub
* **deps**: Bump serde-toml-merge from 0.3.7 to 0.3.8 (#20607) [72ba4d5](https://github.com/answerbook/vector/commit/72ba4d5dd544487e2df6bf38f78a427f3d301bd2) - GitHub
* **deps**: Bump syn from 2.0.60 to 2.0.61 (#20444) [911e63d](https://github.com/answerbook/vector/commit/911e63d4ee35c1de5082b6c2df6bc4ac0678ff31) - GitHub
* **deps**: Bump syn from 2.0.61 to 2.0.65 (#20526) [bd15a6b](https://github.com/answerbook/vector/commit/bd15a6b130cd6833f03ad288ad32e3f2e61fe4e8) - GitHub
* **deps**: Bump syn from 2.0.65 to 2.0.66 (#20559) [1418455](https://github.com/answerbook/vector/commit/1418455e7119f0ff37a825f0f0ce910b7c3ef6c9) - GitHub
* **deps**: Bump the aws group with 2 updates (#20551) [fb1af6a](https://github.com/answerbook/vector/commit/fb1af6aa0daec5e2ccf429f15d84d2eb3c7978ed) - GitHub
* **deps**: Bump the aws group with 2 updates (#20606) [7d48cd2](https://github.com/answerbook/vector/commit/7d48cd20a900fe228f6537540e3af650ffd4a4e7) - GitHub
* **deps**: Bump the aws group with 2 updates (#20659) [e8fd823](https://github.com/answerbook/vector/commit/e8fd823c9eed5df3b05c7cd86a799ce49880d206) - GitHub
* **deps**: Bump the aws group with 3 updates (#20456) [f7561c0](https://github.com/answerbook/vector/commit/f7561c0812b8d07eea4dbf1b9c3364a07ab1f9fe) - GitHub
* **deps**: Bump the aws group with 3 updates (#20545) [e418dd5](https://github.com/answerbook/vector/commit/e418dd551e876f68279fa06d7feff6a59921869f) - GitHub
* **deps**: Bump the aws group with 3 updates (#20638) [1a137cf](https://github.com/answerbook/vector/commit/1a137cf625e7881b02ef287febb80dc4edb244bb) - GitHub
* **deps**: Bump the clap group across 1 directory with 2 updates (#20626) [b266a95](https://github.com/answerbook/vector/commit/b266a95c881bcefb6b0da4821b3018f93b2c4afb) - GitHub
* **deps**: Bump the graphql group with 2 updates (#20473) [3770249](https://github.com/answerbook/vector/commit/3770249bfb31663fc3d3bfe2c52646055361e346) - GitHub
* **deps**: Bump the graphql group with 2 updates (#20627) [9a18b0f](https://github.com/answerbook/vector/commit/9a18b0fd12c02208da4f6edd9067a95ad37ff3d0) - GitHub
* **deps**: Bump the prost group with 2 updates (#20552) [b5ec6ae](https://github.com/answerbook/vector/commit/b5ec6ae8ddbdd37718375d33da42353bd344ee78) - GitHub
* **deps**: Bump the prost group with 3 updates (#20519) [d62fcdd](https://github.com/answerbook/vector/commit/d62fcdd64cb50b969144db6cfd2c63c354021485) - GitHub
* **deps**: Bump thiserror from 1.0.59 to 1.0.60 (#20441) [cdda013](https://github.com/answerbook/vector/commit/cdda013f35fa530abd02c469386da8cb8925b0f7) - GitHub
* **deps**: Bump thiserror from 1.0.60 to 1.0.61 (#20521) [c880b96](https://github.com/answerbook/vector/commit/c880b960eb960ebfcfe8ecb1c943cb3b35f0b5c6) - GitHub
* **deps**: Bump tokio from 1.37.0 to 1.38.0 (#20588) [d34e619](https://github.com/answerbook/vector/commit/d34e6197ffb0589996b32213a1e536547b7d054d) - GitHub
* **deps**: Bump toml from 0.8.12 to 0.8.13 (#20500) [a725d74](https://github.com/answerbook/vector/commit/a725d740398226946b4476fa14c490a83d2733b2) - GitHub
* **deps**: Bump toml from 0.8.13 to 0.8.14 (#20608) [55d48d5](https://github.com/answerbook/vector/commit/55d48d5ed1668789ec413296e6dd0016489c22b5) - GitHub
* **deps**: Bump url from 2.5.0 to 2.5.1 (#20642) [662d1d0](https://github.com/answerbook/vector/commit/662d1d09501601a4c7356a4698dc7f3b4dc980d4) - GitHub
* **deps**: Drop `cached` (#20455) [783ed1f](https://github.com/answerbook/vector/commit/783ed1fe1a4cf0de415fde71706b1e315a58d215) - GitHub
* **deps**: Ensure prometheus::remote_write::Errors is appropriately gated (#20657) [3a03a54](https://github.com/answerbook/vector/commit/3a03a54bf3121226db74d00dc3adeccbdb4b692e) - GitHub
* **dev**: Update sasl2-sys to fix building on GCC 14+ / CLang environments (#20645) [f6527ec](https://github.com/answerbook/vector/commit/f6527ec5869b4502ea1dbb68e85634a331d2eb3d) - GitHub
* **docs**: Fix link in release notes (#20454) [0da155d](https://github.com/answerbook/vector/commit/0da155d49f7fb01a4499c7e52f8357779c723888) - GitHub
* **docs**: Remove references to HTTP Content-Length from component spec (#20615) [187f119](https://github.com/answerbook/vector/commit/187f1199ecf05d4123f35ec881dd0b85d4f543b1) - GitHub
* **elasticsearch**: add arbitrary trait to ElasticsearchApiVersion (#20580) [4a4fc2e](https://github.com/answerbook/vector/commit/4a4fc2e9162ece483365959f5222fc5a38d1dad9) - GitHub
* **enrichment_tables**: Make configuration fields public (#20614) [322c7df](https://github.com/answerbook/vector/commit/322c7dfa13fa44280d8d199d96d6cf4c92b73750) - GitHub
* **enterprise**: Remove `enterprise` feature (#20468) [4f5c99d](https://github.com/answerbook/vector/commit/4f5c99d1afbab12e7d86349f7f269c74fcc7814b) - GitHub
* **gcp_chronicle**: allow enabling google chronicle separately from… (#20557) [5abaa32](https://github.com/answerbook/vector/commit/5abaa3209b5a294e4181e06976c72d5dc700e9f1) - GitHub
* **kubernetes_logs source**: Remove unreported metrics from the docs (#20530) [4cc1ecf](https://github.com/answerbook/vector/commit/4cc1ecfb9aee23500611eaf0c02bb78d62fc1a34) - GitHub
* **loki sink**: Changed OutOfOrderAction default to accept (#20469) [02de739](https://github.com/answerbook/vector/commit/02de73907ab5a205aae0bd04ebcf092b45e3eaf3) - GitHub
* Move third-party proto files into their own module (#20556) [378f3b0](https://github.com/answerbook/vector/commit/378f3b09b9f1adb8a03a772134e8a56f61fb99cd) - GitHub
* **reduce**: add arbitrary trait (#20537) [98a795e](https://github.com/answerbook/vector/commit/98a795eaf12132a4dd6cd506781cf3127b9ad6f9) - GitHub
* **reduce**: expose reduce logic (#20543) [27393b5](https://github.com/answerbook/vector/commit/27393b5c2a8ce4b71072db41df3210a72031735c) - GitHub
* **releasing**: Clarify the removal of the `enterprise` configuration in 0.39.0 [2d4c202](https://github.com/answerbook/vector/commit/2d4c202db75ad5d334d4872f6119feef7636bb6c) - Jesse Szwedko
* **releasing**: Prepare v0.38.0 release [a7b9f9e](https://github.com/answerbook/vector/commit/a7b9f9e3bd4b4805d2f3cdc05ee732eba85f88e5) - Jesse Szwedko
* **releasing**: Prepare v0.39.0 release [897120b](https://github.com/answerbook/vector/commit/897120bcbc51e49d17f25e6338b3e7ccf6026409) - Jesse Szwedko
* **releasing**: Regenerate manifests for chart v0.33.0 (#20453) [e1d1e85](https://github.com/answerbook/vector/commit/e1d1e851e71bd8c20f9c53a8340f0cdc3d0e7c12) - GitHub
* Update buf config to v2 (#20558) [2e884c5](https://github.com/answerbook/vector/commit/2e884c518838e7893c5baaf7d097dbfb8614e28e) - GitHub
* **vrl**: update to crate version to 0.16.0 (#20634) [36abc45](https://github.com/answerbook/vector/commit/36abc4578c1b825f5222e8a59fce9c80142cd5be) - GitHub


### Documentation

* Improve NixOS documentation (#20497) [234b126](https://github.com/answerbook/vector/commit/234b126f733472df87caa4cec23be6e4396c05de) - GitHub


### Features

* **#19183**: make Chronicle namespace a templateable field (#20579) [e780cde](https://github.com/answerbook/vector/commit/e780cde204e625739f4d068dce8ad2d66668759d) - GitHub
* **kafka source**: use `msg.payload_len()` to initialize `FramedRead` (#20529) [f5abce9](https://github.com/answerbook/vector/commit/f5abce9b144604ae9d49251cba39a9f8ea717497) - GitHub
* **new encoding**: add pretty json encoding  (#20384) [86f0a88](https://github.com/answerbook/vector/commit/86f0a8840458dade9676a7581ca60cc95c6dad7d) - GitHub
* **opentelemetry source**: support trace ingestion (#19728) [d1d122e](https://github.com/answerbook/vector/commit/d1d122e2aa8a29c1d505ca71c9e382eb3ad06691) - GitHub


### Miscellaneous

* Merge pull request #618 from answerbook/feature/LOG-20715 [d046cab](https://github.com/answerbook/vector/commit/d046cab392ac20c47a45a8f63e5f17a5b6d4d595) - GitHub [LOG-20715](https://logdna.atlassian.net/browse/LOG-20715)
* Merge remote-tracking branch 'origin/master' into feature/LOG-20715 [fb9aa81](https://github.com/answerbook/vector/commit/fb9aa81308a6dadafcdec0407e109957fa2e5721) - Dan Hable [LOG-20715](https://logdna.atlassian.net/browse/LOG-20715)
* Merge branch 'upstream-0.38' into upstream-0.39 [a028b97](https://github.com/answerbook/vector/commit/a028b97b61815f6d55b13e5e156f467a11334301) - Chris Nixon
* Merge upstream/v0.39 into upstream-0.39 [3c83903](https://github.com/answerbook/vector/commit/3c8390352316932717d360a753ef12bbfbacd69f) - Chris Nixon
* Fix link to remap event data model (#20536) [082d935](https://github.com/answerbook/vector/commit/082d93575ad83ff2c824d3597d6cf2c64c3af57e) - GitHub
* **http**: log source IP as `host` key in the `http` source (#19667) [49867fd](https://github.com/answerbook/vector/commit/49867fd5c7c3307a0134e605f69d1cf87b137dc5) - GitHub
* **kafka sink**: Make healthcheck topic configurable (#20373) [c2fc5ef](https://github.com/answerbook/vector/commit/c2fc5ef43a19b803a613e5a3d163da635d45e644) - GitHub
* **prometheus_exporter sink**: add CompressionLayer support (#20065) [eb9f423](https://github.com/answerbook/vector/commit/eb9f4232159ee44038c1fa99439963fba28df5bd) - GitHub
* **reduce transform**: New setting for reduce transform: end_every_period_ms (#20440) [0e034ee](https://github.com/answerbook/vector/commit/0e034ee3c52fafb7d81923c3ac1d2050ae5b6358) - GitHub
* **remap transform**: add caching to remap vrl compilation (#20555) [48a29b3](https://github.com/answerbook/vector/commit/48a29b3c1363f54998fd3fb2a4cac204b33a1c35) - GitHub
* **vrl**: add docs for `psl` argument for `parse_etld` (#20542) [6db92ac](https://github.com/answerbook/vector/commit/6db92acaa085dbb271b650adc42c2e8b36d53fa3) - GitHub
* **vrl**: Correct documentation of algorithms supported by `decrypt` (#20658) [ebec095](https://github.com/answerbook/vector/commit/ebec09578e49e97019b13fc8297b432e4ae9bd49) - GitHub


### **BREAKING CHANGES**

* **enterprise:** Remove `enterprise` feature (#20468)

# [6.0.0](https://github.com/answerbook/vector/compare/v5.5.1...v6.0.0) (2024-10-02)


### Bug Fixes

* **aws service**: use http client when building assume role for AccessKey (#20285) [1b0bdcf](https://github.com/answerbook/vector/commit/1b0bdcf022397f78df5522fe369eb457e5bae9dc) - GitHub
* **ci**: component features comment trigger one runner label (#20430) [c2765f4](https://github.com/answerbook/vector/commit/c2765f45e243636b40d22a819d1496549eb40ed4) - GitHub
* **datadog_agent**: align `ddtags` parsing with DD logs intake (#20184) [1f72366](https://github.com/answerbook/vector/commit/1f72366dd96ce40ba11c0fea1c044b2184dbb616) - GitHub
* **datadog_logs sink**: reconstruct `ddtags` if not already in format expected by DD logs intake (#20198) [1fb257e](https://github.com/answerbook/vector/commit/1fb257efb4fc9e9fc28304281292b5fab5601614) - GitHub
* **enrichment_tables**: bring back support for `GeoLite2-City` db (#20192) [6f35125](https://github.com/answerbook/vector/commit/6f351255f49c66c02298a6b5652bd9cfce2cde9a) - GitHub
* **host_metrics source**: Include cgroups2 root memory metrics (#20294) [0efb520](https://github.com/answerbook/vector/commit/0efb52048217904862ad3b5264ebe742951c69f9) - GitHub
* **kafka source**: fix source span instrumentation (#20242) [9080d05](https://github.com/answerbook/vector/commit/9080d053edff3aa520be857f9619eac5345b8654) - GitHub
* **log_to_metric transform**: set not working in log-to-metric transform when all_metrics=true (#20228) [b634432](https://github.com/answerbook/vector/commit/b63443213f4efb7129e253b397f84c3bf552c41a) - GitHub
* **metrics**: normalizer doesn't handle mix of absolute and incremental metrics (#20193) [d8b6717](https://github.com/answerbook/vector/commit/d8b67177e8544020a85cdc080cac5ee5f6328bae) - GitHub
* **splunk_hec_logs sink**: don't attempt to remove timestamp if auto extract is enabled (#20213) [9e7f6e3](https://github.com/answerbook/vector/commit/9e7f6e3a7b095bcea285bd02740c5bc5605e0f08) - GitHub


### Chores

* Add future deprecation note for `component_(sent|received)_bytes_total` metrics (#20412) [61b0b36](https://github.com/answerbook/vector/commit/61b0b368546b37b2b6f07aa8ad3009caf69d1d39) - GitHub
* **ci**: Bump actions/add-to-project from 0.6.1 to 1.0.0 (#20194) [5ffb1a5](https://github.com/answerbook/vector/commit/5ffb1a5557f1a257d841687b4d3ba48547dfd8d4) - GitHub
* **ci**: Bump actions/add-to-project from 1.0.0 to 1.0.1 (#20295) [32dedb4](https://github.com/answerbook/vector/commit/32dedb474a981d71a98d521c7416317aa8a600a3) - GitHub
* **ci**: Bump bufbuild/buf-breaking-action from 1.1.3 to 1.1.4 (#20171) [656e220](https://github.com/answerbook/vector/commit/656e2207c2f3057d72c19667b1673f621be56ade) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.30.0 to 1.30.1 (#20238) [ad71ab3](https://github.com/answerbook/vector/commit/ad71ab38615fd2bd23165fe0ab5a11d803bc163e) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.30.1 to 1.31.0 (#20361) [d463a76](https://github.com/answerbook/vector/commit/d463a76365c46d81c8b7bdaaaefa70ae4dfc77d7) - GitHub
* **ci**: Bump docker/setup-buildx-action from 3.2.0 to 3.3.0 (#20260) [7b2d389](https://github.com/answerbook/vector/commit/7b2d389bc3502108c6de0d3432d6c8cf537508f5) - GitHub
* **ci**: Bump timeout for `publish-new-environment` in CI (#20426) [3813260](https://github.com/answerbook/vector/commit/381326077ca7fb856c0dd645927ea652c0d70a37) - GitHub
* **ci**: Bump timeouts for test-misc and integration tests (#20438) [04a6b55](https://github.com/answerbook/vector/commit/04a6b55be891a7062ef526779aa0457d5d5b2972) - GitHub
* **ci**: Download SHA256SUMS to correct location (#20269) [665ab39](https://github.com/answerbook/vector/commit/665ab39dce1cfcad46cbae746c82973df957bfdf) - GitHub
* **ci**: Drop `apt-get upgrade` (#20203) [5ebcc88](https://github.com/answerbook/vector/commit/5ebcc8874d283765593bed7683bd76ec27ebcc00) - GitHub
* **ci**: Only use one label for selecting GHA runner (#20210) [5f981d4](https://github.com/answerbook/vector/commit/5f981d46f1a890fa84e92465b94061fde01efd5d) - GitHub
* **ci**: peg `fakeintake` docker image (#20196) [2e7a3ca](https://github.com/answerbook/vector/commit/2e7a3ca03138896d850acdcee47e6deba6f3d29c) - GitHub
* **ci**: Remove pip install of modules (#20204) [11c968d](https://github.com/answerbook/vector/commit/11c968d350d2b18337664384be87d7cba345a1c6) - GitHub
* **config**: Remove deprecated `--strict-env-vars` flag (#20422) [8ed9ec2](https://github.com/answerbook/vector/commit/8ed9ec24c1eba0b2191d7c1f24ec2a7540b5bebf) - GitHub
* **core**: expose semantic meaning log event helper fn (#20178) [5349313](https://github.com/answerbook/vector/commit/5349313dacc77f6798b51574c0639220b58f4284) - GitHub
* **core**: Implement `LogEvent::get_mut_by_meaning` (#20358) [b025ba7](https://github.com/answerbook/vector/commit/b025ba778ac62937c0ff6c3411b8770d0d3f7cbc) - GitHub
* **datadog logs**: expose internals and test utils (#20429) [e5a32a3](https://github.com/answerbook/vector/commit/e5a32a3fbf615a7a31d07eff2b31abea7ef6e560) - GitHub
* **datadog_logs sink**: properly encode all semantically defined DD reserved attributes (#20226) [38f4868](https://github.com/answerbook/vector/commit/38f4868a9e35dade00098ff71bf5c3c294c335d0) - GitHub
* **datadog**: make consts public (#20257) [95a987b](https://github.com/answerbook/vector/commit/95a987bd718c89254dff6460947097f3c90d41a9) - GitHub
* **deps**: Bump anyhow from 1.0.81 to 1.0.82 (#20275) [f391fe2](https://github.com/answerbook/vector/commit/f391fe2d2f3e8c3c5462d908d08813ff1a47ff66) - GitHub
* **deps**: Bump arc-swap from 1.7.0 to 1.7.1 (#20166) [1befcd9](https://github.com/answerbook/vector/commit/1befcd9a6f8d0115fcc11ac866ba380c9893ee25) - GitHub
* **deps**: Bump async-compression from 0.4.6 to 0.4.7 (#20221) [562f7b7](https://github.com/answerbook/vector/commit/562f7b780ad339f933d49ee119b2646467a401f5) - GitHub
* **deps**: Bump async-compression from 0.4.7 to 0.4.8 (#20255) [a568cf9](https://github.com/answerbook/vector/commit/a568cf90a4b369a31a1d4d17c581ae2a01976082) - GitHub
* **deps**: Bump async-compression from 0.4.8 to 0.4.9 (#20391) [a53251c](https://github.com/answerbook/vector/commit/a53251c77d664f7889de218e6838bcc5ef5bc2bb) - GitHub
* **deps**: Bump async-recursion from 1.1.0 to 1.1.1 (#20376) [e672483](https://github.com/answerbook/vector/commit/e6724838e9fb48451b2a799ddbcf3b229ce5a199) - GitHub
* **deps**: Bump async-trait from 0.1.78 to 0.1.79 (#20163) [04813b9](https://github.com/answerbook/vector/commit/04813b9eee62d7dc3c55b761b144478af73a1a95) - GitHub
* **deps**: Bump async-trait from 0.1.79 to 0.1.80 (#20290) [177cd0e](https://github.com/answerbook/vector/commit/177cd0e7ca174e5c8e9ba9b60c7faa728bbd4981) - GitHub
* **deps**: Bump base64 from 0.22.0 to 0.22.1 (#20408) [d549edd](https://github.com/answerbook/vector/commit/d549edd95b232aae28f3b663fe128b11b7e9366a) - GitHub
* **deps**: Bump bollard from 0.16.0 to 0.16.1 (#20158) [cf61b90](https://github.com/answerbook/vector/commit/cf61b90a8cb6c80a541fad0689867f8ae55bae5a) - GitHub
* **deps**: Bump bytes from 1.5.0 to 1.6.0 (#20167) [fead132](https://github.com/answerbook/vector/commit/fead132341ac52d487abd85bb74b4cab57eafaff) - GitHub
* **deps**: Bump cached from 0.49.2 to 0.49.3 (#20247) [c3c6edc](https://github.com/answerbook/vector/commit/c3c6edc65a41e61ceac50d2d3d0d335f02acadb8) - GitHub
* **deps**: Bump cached from 0.49.3 to 0.50.0 (#20379) [056c2df](https://github.com/answerbook/vector/commit/056c2df60a7f776eca2fad6a4686deee35f9b340) - GitHub
* **deps**: Bump cargo_toml from 0.19.2 to 0.20.0 (#20350) [334913e](https://github.com/answerbook/vector/commit/334913e8860e7a167d6e90662bc6fe83deffdf5d) - GitHub
* **deps**: Bump cargo_toml from 0.20.0 to 0.20.2 (#20375) [5393854](https://github.com/answerbook/vector/commit/539385408ac20787f00d3847dc8af2b754c3d76a) - GitHub
* **deps**: Bump chrono from 0.4.34 to 0.4.37 (#20195) [044e29d](https://github.com/answerbook/vector/commit/044e29daf4b57832481224703c1c6e085c05da80) - GitHub
* **deps**: Bump clap_complete from 4.5.1 to 4.5.2 in the clap group (#20272) [d6ae1ae](https://github.com/answerbook/vector/commit/d6ae1ae5c68875f079d482c9a95dd47582613a77) - GitHub
* **deps**: Bump crc from 3.0.1 to 3.2.1 (#20263) [2840305](https://github.com/answerbook/vector/commit/28403059420fb3fe6f8188dc0b63db6c41c61d53) - GitHub
* **deps**: Bump data-encoding from 2.5.0 to 2.6.0 (#20389) [f68cacf](https://github.com/answerbook/vector/commit/f68cacf7d73b867d8aeb8d006e2f3beecc76fc08) - GitHub
* **deps**: Bump databend-client from 0.17.0 to 0.17.1 (#20377) [3ae189f](https://github.com/answerbook/vector/commit/3ae189f60f09630e5c1da264e2fd3c7ce69a877b) - GitHub
* **deps**: Bump enum_dispatch from 0.3.12 to 0.3.13 (#20207) [be56139](https://github.com/answerbook/vector/commit/be56139a9c662ac09bd294f801b93073cccb0754) - GitHub
* **deps**: Bump env_logger from 0.10.2 to 0.11.3 (#20416) [562bb68](https://github.com/answerbook/vector/commit/562bb686a016dc84c1adf7f05592712f09b509f5) - GitHub
* **deps**: Bump express from 4.18.2 to 4.19.2 in /website (#20183) [b05fb60](https://github.com/answerbook/vector/commit/b05fb602bb734adb82f17032e91c44ae9fc5a4df) - GitHub
* **deps**: Bump fakedata_generator from 0.4.0 to 0.5.0 (#20351) [6cf2e8e](https://github.com/answerbook/vector/commit/6cf2e8e015a652da463c2c7786144a74703a9e87) - GitHub
* **deps**: Bump flate2 from 1.0.28 to 1.0.30 (#20399) [2dc53ff](https://github.com/answerbook/vector/commit/2dc53ff6f862d95274d6932f974c9f9163830449) - GitHub
* **deps**: Bump getrandom from 0.2.12 to 0.2.14 (#20264) [eedc623](https://github.com/answerbook/vector/commit/eedc623bfc0663f9240bd10f30748c6a3e3e3611) - GitHub
* **deps**: Bump governor from 0.6.0 to 0.6.3 (#20419) [81861f3](https://github.com/answerbook/vector/commit/81861f3b1d9e2ae55c443d57a9b5f53f99d23262) - GitHub
* **deps**: Bump graphql_client from 0.13.0 to 0.14.0 (#20187) [8a78b82](https://github.com/answerbook/vector/commit/8a78b8270b61136a37e7a8b5c364b912e6515d2b) - GitHub
* **deps**: Bump hashbrown from 0.14.3 to 0.14.5 (#20392) [2346266](https://github.com/answerbook/vector/commit/2346266d0cbd0ea0f748bd73d62bec3bee028497) - GitHub
* **deps**: Bump hickory-proto from 0.24.0 to 0.24.1 (#20341) [cf6469e](https://github.com/answerbook/vector/commit/cf6469e0c6bb2ee12b0a3e33aef718203cd2dda6) - GitHub
* **deps**: Bump hostname from 0.3.1 to 0.4.0 (#20222) [6673314](https://github.com/answerbook/vector/commit/667331435254c84ebe8a3d2b5173cebfcbcc7507) - GitHub
* **deps**: Bump indexmap from 2.2.5 to 2.2.6 (#20161) [4615977](https://github.com/answerbook/vector/commit/461597721247195853910c3b3d7cba9a8a16b3cc) - GitHub
* **deps**: Bump indoc from 2.0.4 to 2.0.5 (#20159) [3c4eb68](https://github.com/answerbook/vector/commit/3c4eb68abb16c81fa7170926256785433b6a3553) - GitHub
* **deps**: Bump lapin from 2.3.1 to 2.3.3 (#20371) [dcc6cc8](https://github.com/answerbook/vector/commit/dcc6cc82100d238068ae7fb26ff9ecab30f5ccff) - GitHub
* **deps**: Bump lapin from 2.3.3 to 2.3.4 (#20418) [8599966](https://github.com/answerbook/vector/commit/859996626ff91c15285970266889284c2fe9caaf) - GitHub
* **deps**: Bump libc from 0.2.153 to 0.2.154 (#20402) [8a571d1](https://github.com/answerbook/vector/commit/8a571d12b72503936f070e99e6d4878eb06234f6) - GitHub
* **deps**: Bump memchr from 2.7.1 to 2.7.2 (#20200) [ae56733](https://github.com/answerbook/vector/commit/ae5673367d3db51700b3c1b3264859f4f18cc2db) - GitHub
* **deps**: Bump mlua from 0.9.6 to 0.9.7 (#20251) [954af0f](https://github.com/answerbook/vector/commit/954af0f2942edbd68f54bef95abcb5797c23d905) - GitHub
* **deps**: Bump mock_instant from 0.3.2 to 0.4.0 (#20249) [94ed61e](https://github.com/answerbook/vector/commit/94ed61e0d17359fccd66fb8b3d36d935df6caae3) - GitHub
* **deps**: Bump MSRV to reduce usage of `async_trait` (#20155) [41bb21e](https://github.com/answerbook/vector/commit/41bb21ef711d55884b02eee42b0126c25e97dd5e) - GitHub
* **deps**: Bump nkeys from 0.4.0 to 0.4.1 (#20289) [9e03021](https://github.com/answerbook/vector/commit/9e03021fcb4e714d6d9dd326434f6711691cee70) - GitHub
* **deps**: Bump os_info from 3.8.1 to 3.8.2 (#20162) [eb7ab42](https://github.com/answerbook/vector/commit/eb7ab42940d1743ad2fa2f44318c0b25240f3595) - GitHub
* **deps**: Bump parking_lot from 0.12.1 to 0.12.2 (#20378) [11ace46](https://github.com/answerbook/vector/commit/11ace462d514b3e06478a7cb7cfe9694d897701a) - GitHub
* **deps**: Bump quanta from 0.12.2 to 0.12.3 (#20218) [a8e17a5](https://github.com/answerbook/vector/commit/a8e17a5e46ccdf55826afa6057fc4e6c1347f017) - GitHub
* **deps**: Bump quote from 1.0.35 to 1.0.36 (#20274) [eaacd2f](https://github.com/answerbook/vector/commit/eaacd2f2763519397850df42ce209ccc977ecfcc) - GitHub
* **deps**: Bump ratatui from 0.26.1 to 0.26.2 (#20310) [1049616](https://github.com/answerbook/vector/commit/1049616f522f08d2045b840012541805203b7e3f) - GitHub
* **deps**: Bump regex from 1.10.3 to 1.10.4 (#20168) [6c3003d](https://github.com/answerbook/vector/commit/6c3003dc0592f9fb7ff121a971db9af44c793a68) - GitHub
* **deps**: Bump rmp-serde from 1.1.2 to 1.2.0 (#20319) [cf7542d](https://github.com/answerbook/vector/commit/cf7542d38deba3c44a6066961cc12d9f62f57916) - GitHub
* **deps**: Bump rmp-serde from 1.2.0 to 1.3.0 (#20407) [ca00cc8](https://github.com/answerbook/vector/commit/ca00cc8835ff70c3c5865aa06925efe9ac25706f) - GitHub
* **deps**: Bump rmpv from 1.0.1 to 1.0.2 (#20318) [7869a78](https://github.com/answerbook/vector/commit/7869a7895d202d3f6eecbf70d40074995947c0dd) - GitHub
* **deps**: Bump rmpv from 1.0.2 to 1.3.0 (#20410) [b22cea7](https://github.com/answerbook/vector/commit/b22cea70f8df8d1d2b48f0ddb1733de9e8782cb3) - GitHub
* **deps**: Bump roaring from 0.10.3 to 0.10.4 (#20409) [fa0b5b3](https://github.com/answerbook/vector/commit/fa0b5b388c26fb75f4665621f1962cf304e7fc38) - GitHub
* **deps**: Bump rstest from 0.18.2 to 0.19.0 (#20273) [a56da92](https://github.com/answerbook/vector/commit/a56da92ffce00913092135ca05a1c441a4fd28bb) - GitHub
* **deps**: Bump rust-toolchain to 1.77.2 (#20344) [cff9c88](https://github.com/answerbook/vector/commit/cff9c88c44bf3e7447801addee729779ccced34a) - GitHub
* **deps**: Bump security-framework from 2.9.2 to 2.10.0 (#20220) [b460f74](https://github.com/answerbook/vector/commit/b460f7406c3480da5347f1e9753129173658555e) - GitHub
* **deps**: Bump serde from 1.0.197 to 1.0.198 (#20321) [45a853c](https://github.com/answerbook/vector/commit/45a853c7245c6eea5099541c0bf28c8e22819d4b) - GitHub
* **deps**: Bump serde from 1.0.198 to 1.0.199 (#20390) [26ece36](https://github.com/answerbook/vector/commit/26ece36c9fb9ebd4b189bbf0213ed50d33750d61) - GitHub
* **deps**: Bump serde from 1.0.199 to 1.0.200 (#20420) [73d7caa](https://github.com/answerbook/vector/commit/73d7caacb7ad8d28177291c49ebd8894eb5b666a) - GitHub
* **deps**: Bump serde_json from 1.0.114 to 1.0.115 (#20188) [173deda](https://github.com/answerbook/vector/commit/173deda35dbf90377a82aacbc5cca273e0468e73) - GitHub
* **deps**: Bump serde_json from 1.0.115 to 1.0.116 (#20320) [5a4a2b2](https://github.com/answerbook/vector/commit/5a4a2b2a10131af7ef4ca32ff13b9040e231f5a6) - GitHub
* **deps**: Bump serde_with from 3.7.0 to 3.8.1 (#20388) [539333d](https://github.com/answerbook/vector/commit/539333df3256ae360eab1f5d2d28d06170b08db8) - GitHub
* **deps**: Bump serde_yaml from 0.9.33 to 0.9.34+deprecated (#20165) [e25efa1](https://github.com/answerbook/vector/commit/e25efa1c270d5cb008240edb4ad2413535a332e7) - GitHub
* **deps**: Bump socket2 from 0.5.6 to 0.5.7 (#20401) [57aed6c](https://github.com/answerbook/vector/commit/57aed6c9b4f96dcfa7ecca5b25d72cab1ff70e0a) - GitHub
* **deps**: Bump syn from 2.0.53 to 2.0.55 (#20164) [52a15a0](https://github.com/answerbook/vector/commit/52a15a0548a03baa59eb4e107bf5a4041505bd29) - GitHub
* **deps**: Bump syn from 2.0.55 to 2.0.57 (#20219) [333ed14](https://github.com/answerbook/vector/commit/333ed14f71c1acaaeb0936f4a3cdefd9e89518f9) - GitHub
* **deps**: Bump syn from 2.0.57 to 2.0.60 (#20329) [14e9b47](https://github.com/answerbook/vector/commit/14e9b478913bda2ffc53899618c4ff317f922929) - GitHub
* **deps**: Bump syslog from 6.1.0 to 6.1.1 (#20340) [8c124ac](https://github.com/answerbook/vector/commit/8c124ac162d22e6b7ffe50b13f385d78b0cb9f11) - GitHub
* **deps**: Bump temp-dir from 0.1.12 to 0.1.13 (#20151) [742b883](https://github.com/answerbook/vector/commit/742b883b5881b1b1f88d01c023c277b293500ee3) - GitHub
* **deps**: Bump the aws group across 1 directory with 4 updates (#20359) [c211ca0](https://github.com/answerbook/vector/commit/c211ca0c799caff292975452ba4bd5e8dbaeaaf5) - GitHub
* **deps**: Bump the aws group with 1 update (#20175) [06f3ad3](https://github.com/answerbook/vector/commit/06f3ad3416b4afe89a02fb29704cda40f8e71da3) - GitHub
* **deps**: Bump the aws group with 1 update (#20229) [1c42df3](https://github.com/answerbook/vector/commit/1c42df30138bee5ce93c60ef76ac39979e507591) - GitHub
* **deps**: Bump the aws group with 2 updates (#20406) [57f602b](https://github.com/answerbook/vector/commit/57f602b460af1e7e2c5cd5cb3a7b70bb9db27f75) - GitHub
* **deps**: Bump the aws group with 3 updates (#20299) [ceb50fb](https://github.com/answerbook/vector/commit/ceb50fba5daf43a8765303b89d09f2c44c399071) - GitHub
* **deps**: Bump the clap group with 1 update (#20176) [5a6b670](https://github.com/answerbook/vector/commit/5a6b670ce05cc7c34b0af0cf2a58810d47b2f71c) - GitHub
* **deps**: Bump the prost group with 4 updates (#20248) [e4e7321](https://github.com/answerbook/vector/commit/e4e7321e7e0a872612458d6f9d1ec083126b76be) - GitHub
* **deps**: Bump the zstd group with 1 update (#20199) [0599d60](https://github.com/answerbook/vector/commit/0599d60758386b631533adce250b4820f5434e59) - GitHub
* **deps**: Bump thiserror from 1.0.58 to 1.0.59 (#20352) [8197f4e](https://github.com/answerbook/vector/commit/8197f4eedd3f3415f83effe2fbb23b69e63fa6d6) - GitHub
* **deps**: Bump tokio from 1.36.0 to 1.37.0 (#20208) [4b4068f](https://github.com/answerbook/vector/commit/4b4068fa3d0a1372b3d6a90c4b5ae73da3fc0f02) - GitHub
* **deps**: Bump vrl from 0.14.0 to 0.15.0 (#20417) [4472d49](https://github.com/answerbook/vector/commit/4472d498b9ba4f10b407284908a7a51f1e70e354) - GitHub
* **deps**: Bump VRL to 0.14.0 (#20398) [b17b420](https://github.com/answerbook/vector/commit/b17b420047031c25834b133e3361c1c03ae5efa6) - GitHub
* **deps**: Bump VRL to 0.15.0 (#20415) [51dd03f](https://github.com/answerbook/vector/commit/51dd03f1ff54ee6810a0842a78fa25b535e8d52f) - GitHub
* **deps**: Bump warp from 0.3.6 to 0.3.7 (#20253) [21578e7](https://github.com/answerbook/vector/commit/21578e786bc9293f3a1f59c1eb11a6e59a70e821) - GitHub
* **deps**: Bump windows-service from 0.6.0 to 0.7.0 (#20317) [431a3c0](https://github.com/answerbook/vector/commit/431a3c034c1316d0d1d6265a96d36fa3462f7cba) - GitHub
* **deps**: Update h2 (#20236) [36d9cce](https://github.com/answerbook/vector/commit/36d9ccebf621bf969841245a66f292c15f83dda1) - GitHub
* **dev**: Bump down zstd-sys from 2.0.10 to 2.0.9 (#20369) [e4e0ea5](https://github.com/answerbook/vector/commit/e4e0ea591cb97ed8308a2aabdde35ec309cf5178) - GitHub
* **dev**: Bump Vector to 0.38.0 (#20180) [c1fc9f0](https://github.com/answerbook/vector/commit/c1fc9f03b816c54b0efb57198e5a16159a324ac9) - GitHub
* **docs**: note for 0.37 about incorrect ddtags parsing behavior (#20186) [5e3984a](https://github.com/answerbook/vector/commit/5e3984a51695511312e7db7cfcad5dc9e90c8e84) - GitHub
* **docs**: Remove package deprecation banner (#20181) [9148785](https://github.com/answerbook/vector/commit/9148785f9232b53792a1dcf8fcb73d97691669cc) - GitHub
* **enterprise**: Deprecate the `enterprise` feature (#20437) [79a2294](https://github.com/answerbook/vector/commit/79a22946c4018d0a07af662ecb4b6fad5d493e45) - GitHub
* fix some typos in comments (#20334) [0cf927d](https://github.com/answerbook/vector/commit/0cf927dccf9cd88e9ffdb8d208e858ff0aef7082) - GitHub
* **releasing**: Bump Kubernetes manifsts to chart version 0.32.0 (#20182) [3378ada](https://github.com/answerbook/vector/commit/3378adad7b5b1819ffd52cf9563f519bb13ebdad) - GitHub
* **releasing**: Prepare v0.37.0 release [a4bb025](https://github.com/answerbook/vector/commit/a4bb02556f5ca3ade33d99f2f62f79c596fb6ee4) - Jesse Szwedko
* **releasing**: Prepare v0.37.1 release [b3b4dce](https://github.com/answerbook/vector/commit/b3b4dce04660c481421723e89f2d4b09e806ae4a) - Jesse Szwedko
* **releasing**: Prepare v0.38.0 release [ea0ec6f](https://github.com/answerbook/vector/commit/ea0ec6f4f871c6cfae354b4c00403b646ba144bb) - Jesse Szwedko
* **releasing**: Regenerate k8s manifests for chart 0.32.1 (#20280) [4850a9a](https://github.com/answerbook/vector/commit/4850a9ae033d474e94c8cec161c8f912a9b7c0d2) - GitHub
* remove unnecessary clone (#20245) [59b3d74](https://github.com/answerbook/vector/commit/59b3d7467777992e94c193d4e32321b179d20b22) - GitHub
* Run unit test tests in CI (#20313) [cf11a01](https://github.com/answerbook/vector/commit/cf11a013d292512d98c6d5f534a7463087dc55ba) - GitHub
* **security**: update rustls crate for RUSTSEC-2024-0336 (#20343) [b4b96aa](https://github.com/answerbook/vector/commit/b4b96aa47feda7101e93dd55367f41d0d9147b4a) - GitHub
* **splunk hec**: add semantic meaning for Vector log namespace (#20292) [fae2ebf](https://github.com/answerbook/vector/commit/fae2ebfcb53035eeccdf399837228c7373d3007f) - GitHub
* **splunk_hec_logs sink**: support log namespaced host and timestamp attributes (#20211) [7b85728](https://github.com/answerbook/vector/commit/7b85728c474abc2ff691624ac253ff1777d450b7) - GitHub
* **testing**: support LogNamespace in Component Validation Framework (#20148) [c7f0a85](https://github.com/answerbook/vector/commit/c7f0a85fbfc6bdcf17c5a3bf1ad571c80731b701) - GitHub


### Documentation

* nixos.md: Corrected to showcase module usage (#20413) [7f46231](https://github.com/answerbook/vector/commit/7f462310b6b74c29d73e26ca75a400f6836f4866) - GitHub


### Features

* **#19183**: add namespace input to chronicle sink (#19398) [cb8f3de](https://github.com/answerbook/vector/commit/cb8f3def4fb63f9d72582701d5e96cfb2f63eff9) - GitHub
* **codecs**: add options to `length_delimited` framing (#20154) [c7dde03](https://github.com/answerbook/vector/commit/c7dde0312a6d04201eb9641fe8b8cc8967ffd3fe) - GitHub
* **config**: Support loading secrets from AWS Secrets Manager (#20142) [19ba841](https://github.com/answerbook/vector/commit/19ba8419eef8f73ff881f382aa655318875bb5ad) - GitHub
* **databend sink**: add config missing_field_as for ndjson insert (#20331) [d2c8809](https://github.com/answerbook/vector/commit/d2c88090322006584b5e9152ba4d635c38dc8ef5) - GitHub
* **databend sink**: service use databend-client (#20244) [0550286](https://github.com/answerbook/vector/commit/05502862d63dd27cf316560ed6d35b627147bc3f) - GitHub
* **elasticsearch sink**: allow external document versioning (#20102) [8a20f05](https://github.com/answerbook/vector/commit/8a20f055b4f799b6ddd318d1ae29b6c47b6d5421) - GitHub


### Miscellaneous

* Merge pull request #609 from answerbook/feature/LOG-20642 [e0c94d0](https://github.com/answerbook/vector/commit/e0c94d008b1306ba7399d293636a236cf4725720) - GitHub [LOG-20642](https://logdna.atlassian.net/browse/LOG-20642)
* Merge remote-tracking branch 'origin/master' into feature/LOG-20642 [f013ff5](https://github.com/answerbook/vector/commit/f013ff5dfc6a38309b626932053194cedb4a7f5a) - Dan Hable [LOG-20642](https://logdna.atlassian.net/browse/LOG-20642)
* Merge remote-tracking branch 'origin/master' into feature/LOG-20642 [80839ee](https://github.com/answerbook/vector/commit/80839ee9a58d65b3de5b5b04fe1a7970cc704b7c) - Dan Hable [LOG-20642](https://logdna.atlassian.net/browse/LOG-20642)
* Merge remote-tracking branch 'origin/master' into upstream-0.38 [ca047ea](https://github.com/answerbook/vector/commit/ca047eadf5a7abbb9f1edbe58873f002fc64eec2) - Dan Hable
* Merge branch 'feature/LOG-20555' into upstream-0.38 [ae9a9f4](https://github.com/answerbook/vector/commit/ae9a9f45ef8904b332f7a17e7e1dcee3b65fb787) - Chris Nixon [LOG-20555](https://logdna.atlassian.net/browse/LOG-20555)
* Merge upstream/v0.38 into upstream-0.37 [c0c0b29](https://github.com/answerbook/vector/commit/c0c0b29c5bb69ad59dab325349a8a392dddd65a9) - Chris Nixon
* added option to insert to random shard (#20336) [69f985d](https://github.com/answerbook/vector/commit/69f985dd170cbb332d417b79700b7a3022747e4e) - GitHub
* Use new VRL protobuf library to remove duplication in the protobuf codec (#20074) [1eb18e2](https://github.com/answerbook/vector/commit/1eb18e26c61e4eeb0df425c1987be60aa40b4501) - GitHub
* **amqp sink**: add expiration option to AMQP messages (#20215) [7d7b1a2](https://github.com/answerbook/vector/commit/7d7b1a2620c65540183298aac804aa4f7abe48ec) - GitHub
* **aws_s3 source**: Adds an option `max_number_of_messages` for the aws_s3 source (#20261) [153919d](https://github.com/answerbook/vector/commit/153919d77e6efa24d7b7573e00b42ce5a0e9b747) - GitHub
* **chronicle sink**: support labels (#20307) [a5a6c6f](https://github.com/answerbook/vector/commit/a5a6c6f99c23484bac6321abe36a56546d314f74) - GitHub
* Correct docker.md so the command can be executed (#20227) [f1fdfd0](https://github.com/answerbook/vector/commit/f1fdfd0a3c00227544d250654ee567925c0a0a79) - GitHub
* **dnstap source, releasing**: Add breaking change note for dnstap source mode (#20202) [81b7b85](https://github.com/answerbook/vector/commit/81b7b854a91bcccbde01677824aa6fed349144cf) - GitHub
* Document new uuid_from_friendly_id function (#20357) [edcbf43](https://github.com/answerbook/vector/commit/edcbf4382e383c29c76afc927d511706bb23f040) - GitHub
* fix grammar in what-is-observability-pipelines.md (#20387) [5f08ce8](https://github.com/answerbook/vector/commit/5f08ce8d9a6437243fe7164947aa6f765f3e27e0) - GitHub
* fix type cardinality docs (#20209) [7d705e0](https://github.com/answerbook/vector/commit/7d705e0c1045f062e80bd2fb5215c0911bf3bf7d) - GitHub
* **kubernetes platform**: fix example kustomization file (#20085) [f2f16b6](https://github.com/answerbook/vector/commit/f2f16b61825be20c9d9e15328d33880c6addaa9a) - GitHub
* **lib,website**: remove repetitive words (#20315) [974c7a9](https://github.com/answerbook/vector/commit/974c7a93092959fcc3828ac4bbcf04d927380866) - GitHub
* **releasing**: Bump distroless base image to debian12 from debian11 (#20267) [304ed46](https://github.com/answerbook/vector/commit/304ed46976c71e3f1313ab9f3233c14e32ed59dc) - GitHub
* **source metrics**: Adding a histogram for event byte size (#19686) [f1439bc](https://github.com/answerbook/vector/commit/f1439bc42e8a9498b169c14eb030d8d6f2530ac8) - GitHub
* Update docker.md (#20346) [25b22d7](https://github.com/answerbook/vector/commit/25b22d7ee272176704826003f8266b2b012d8a46) - GitHub
* Update pacman.md as vector is available now in extra repository (#20241) [c9a6864](https://github.com/answerbook/vector/commit/c9a686473bc544acc63cb3148dba3cab46b8f5b0) - GitHub


### Tests

* Install dd-pkg in *-verify workflows and lint in verify-install.sh (#20397) [bcaba0e](https://github.com/answerbook/vector/commit/bcaba0e22d830dbe058956d833fbf973a6e8d818) - GitHub


### **BREAKING CHANGES**

* **config:** Remove deprecated `--strict-env-vars` flag (#20422)
* **datadog_agent:** align `ddtags` parsing with DD logs intake (#20184)

## [5.5.1](https://github.com/answerbook/vector/compare/v5.5.0...v5.5.1) (2024-10-02)


### Bug Fixes

* **mtp**: Expose connection pool param in template [83aec51](https://github.com/answerbook/vector/commit/83aec516ef64ddce1d4ac73764d8b0ac0943a738) - Dan Hable [LOG-20694](https://logdna.atlassian.net/browse/LOG-20694)


### Chores

* **deps**: bump deadpool_postgres [e9c8bf1](https://github.com/answerbook/vector/commit/e9c8bf11c5a09e3ebbca92a4b36a792f89f160f9) - Dan Hable [LOG-20694](https://logdna.atlassian.net/browse/LOG-20694)

# [5.5.0](https://github.com/answerbook/vector/compare/v5.4.0...v5.5.0) (2024-09-30)


### Features

* Bump VRL version to 0.20.0 (#615) [3273917](https://github.com/answerbook/vector/commit/3273917ebe3aa38f56bc980533d36ead7026767e) - GitHub [LOG-20673](https://logdna.atlassian.net/browse/LOG-20673)

# [5.4.0](https://github.com/answerbook/vector/compare/v5.3.0...v5.4.0) (2024-09-30)


### Features

* **tracing**: disable event tracing by default [9874e48](https://github.com/answerbook/vector/commit/9874e4841ca828973690db825e6ee54c317ca44b) - Dan Hable [LOG-20623](https://logdna.atlassian.net/browse/LOG-20623)

# [5.3.0](https://github.com/answerbook/vector/compare/v5.2.4...v5.3.0) (2024-09-27)


### Features

* **tracing**: Mezmo transform tracing [94f66ca](https://github.com/answerbook/vector/commit/94f66ca99036f673f01a56a4e9c4fdf543e167e5) - Dan Hable [LOG-20623](https://logdna.atlassian.net/browse/LOG-20623)

## [5.2.4](https://github.com/answerbook/vector/compare/v5.2.3...v5.2.4) (2024-09-26)


### Bug Fixes

* **contextual logs**: sample overflow fix [3c58d31](https://github.com/answerbook/vector/commit/3c58d312292bd4e75902f93fd2f1120e501b6739) - Sergey Opria [LOG-20654](https://logdna.atlassian.net/browse/LOG-20654)


### Miscellaneous

* Merge pull request #614 from answerbook/sopria/LOG-20654 [7d74831](https://github.com/answerbook/vector/commit/7d7483170684968eb1f25d534ce7bedcd97c17fa) - GitHub [LOG-20654](https://logdna.atlassian.net/browse/LOG-20654)

## [5.2.3](https://github.com/answerbook/vector/compare/v5.2.2...v5.2.3) (2024-09-26)


### Bug Fixes

* **postgres**: fixed stupid logic mistake [603953d](https://github.com/answerbook/vector/commit/603953d4a3eed43c4f94d064e267b2583425e52a) - Dan Hable [LOG-20615](https://logdna.atlassian.net/browse/LOG-20615)

## [5.2.2](https://github.com/answerbook/vector/compare/v5.2.1...v5.2.2) (2024-09-26)


### Chores

* **tests**: adding some additional test cases [26cd427](https://github.com/answerbook/vector/commit/26cd42721ff5777342d595aded188e1bed268f99) - Dan Hable [LOG-20615](https://logdna.atlassian.net/browse/LOG-20615)

## [5.2.1](https://github.com/answerbook/vector/compare/v5.2.0...v5.2.1) (2024-09-26)


### Bug Fixes

* **postgres**: Use a single connection pool [638cc3e](https://github.com/answerbook/vector/commit/638cc3ebb809b529506ffea2cb17df6a63b87603) - Dan Hable [LOG-20615](https://logdna.atlassian.net/browse/LOG-20615)


### Miscellaneous

* Merge pull request #610 from answerbook/dhable/LOG-20615 [8cf4c0f](https://github.com/answerbook/vector/commit/8cf4c0f949603dcc19c67708ffceddb93a13d013) - GitHub [LOG-20615](https://logdna.atlassian.net/browse/LOG-20615)

# [5.2.0](https://github.com/answerbook/vector/compare/v5.1.0...v5.2.0) (2024-09-26)


### Features

* **sources**: `demo-logs` can work with user-provided data [cf3680e](https://github.com/answerbook/vector/commit/cf3680e162abd2af0eb2e3300d17c86b771d5808) - Darin Spivey [LOG-20639](https://logdna.atlassian.net/browse/LOG-20639)

# [5.1.0](https://github.com/answerbook/vector/compare/v5.0.0...v5.1.0) (2024-09-24)


### Features

* **contextual logs**: add contextual logs for data profiling [4d1e5a2](https://github.com/answerbook/vector/commit/4d1e5a24ecf96a245bb217852aefe19ff1b08500) - Sergey Opria [LOG-20584](https://logdna.atlassian.net/browse/LOG-20584)


### Miscellaneous

* Merge pull request #604 from answerbook/sopria/LOG-20584 [3311491](https://github.com/answerbook/vector/commit/33114913e7acc599f035efea69a67622a79d1c61) - GitHub [LOG-20584](https://logdna.atlassian.net/browse/LOG-20584)

# [5.0.0](https://github.com/answerbook/vector/compare/v4.6.1...v5.0.0) (2024-09-20)


### Bug Fixes

* **aws provider**: Enable `credentials-process` for `aws-config` (#20030) [1b87cce](https://github.com/answerbook/vector/commit/1b87cce1f96ddb89163bc0d6f41d78b403ea47c6) - GitHub
* **aws service**: determine region using our http client (#19972) [a9cee3f](https://github.com/answerbook/vector/commit/a9cee3f796624e1a18a3ae9243a0666cffd27aa5) - GitHub
* **codecs**: expose VRL deserializer options (#19862) [79ab389](https://github.com/answerbook/vector/commit/79ab38947f5869afe154f83cf15868c01b43ac4b) - GitHub
* **compression**: Fix gzip and zlib performance degradation (#20032) [d07a435](https://github.com/answerbook/vector/commit/d07a435a3ef0002919a3d2d140843b998f9689d3) - GitHub
* **datadog_agent source**: bugs in internal component metric reporting (#20044) [ad6a48e](https://github.com/answerbook/vector/commit/ad6a48efc0f79b2c18a5c1394e5d8603fdfd1bab) - GitHub
* **datadog_logs sink**: relax required input semantic meanings (#20086) [0be97cd](https://github.com/answerbook/vector/commit/0be97cdae0d97d9ccd9fb2e14501c9dd82fb6e10) - GitHub
* **docs**: Use `component_kind` rather than `kind` for Hugo (#20058) [d2aca62](https://github.com/answerbook/vector/commit/d2aca62f1edcedd76bb818dc936a54b0928b0786) - GitHub
* **docs**: Use `component_kind` rather than `kind` in templates (#20063) [4804e17](https://github.com/answerbook/vector/commit/4804e1745170dab2075fe6ef27534d57033ec2f7) - GitHub
* **docs**: Use correct how_it_works section for Vector sink (#20095) [4671ccb](https://github.com/answerbook/vector/commit/4671ccbf0a6359ef8b752fa99fae9eb9c60fdee5) - GitHub
* **elasticsearch sink**: Readd error log for elasticsearch sink (#19846) [ebdc64d](https://github.com/answerbook/vector/commit/ebdc64dbfc0ac71a1ff73ab9080849eca718a442) - GitHub
* **enrichment_tables**: bring back support for `GeoLite2-City` db (#20192) [17fb71c](https://github.com/answerbook/vector/commit/17fb71c10950357fbcfff818458b6cbd4fe6e0fa) - Jesse Szwedko
* **file source**: Set ignored_header_bytes default to `0` (#20076) [f7380e4](https://github.com/answerbook/vector/commit/f7380e45e4e1af63dd1bb3ecefac50ff45376a3c) - GitHub
* **pulsar source**: PulsarErrorEvent only occurs for the source (#19950) [f33169d](https://github.com/answerbook/vector/commit/f33169d6aa7d130f8a6a47a7060eeb3c69e22e98) - GitHub
* **splunk_hec source**: calculate `EstimatedJsonSizeOf` for `component_received_event_bytes_total` before enrichment (#19942) [8db6288](https://github.com/answerbook/vector/commit/8db6288b4cc2ecf070649e0dc53879f267f41c32) - GitHub
* **splunk_hec_logs sink**: don't remove timestamp for `raw` endpoint (#19975) [fa99d6c](https://github.com/answerbook/vector/commit/fa99d6c2cdc6457d6f70f00dccf8e03d57ffce3a) - GitHub


### Chores

* **api**: Fix API address example (#19858) [8d897af](https://github.com/answerbook/vector/commit/8d897af2f621a0402678141a3a94e1196ea56037) - GitHub
* **api**: Move host_metrics feature gate (#20134) [58a4a2e](https://github.com/answerbook/vector/commit/58a4a2ef52e606c0f9b9fa975cf114b661300584) - GitHub
* **ci**: Add a timeout to all CI jobs (#19895) [50a0c9b](https://github.com/answerbook/vector/commit/50a0c9bc118ee282144b14b3ed49f84cb5ce7c93) - GitHub
* **ci**: add component validation (#19932) [4415040](https://github.com/answerbook/vector/commit/44150403903915f0fa8b31e8fd20b2d8cb33b480) - GitHub
* **ci**: Bump actions/add-to-project from 0.5.0 to 0.6.0 (#19960) [11f6491](https://github.com/answerbook/vector/commit/11f6491f77bd9fc98c3e19859d87aa036184a1d3) - GitHub
* **ci**: Bump actions/add-to-project from 0.6.0 to 0.6.1 (#20137) [db9c681](https://github.com/answerbook/vector/commit/db9c681fd99234f6cd4799185bace2f351e0712d) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.29.0 to 1.30.0 (#20056) [0ec279d](https://github.com/answerbook/vector/commit/0ec279d2a1b6a113f6e62d1f755a29a371862307) - GitHub
* **ci**: Bump docker/build-push-action from 5.1.0 to 5.2.0 (#20057) [52d72da](https://github.com/answerbook/vector/commit/52d72dae521be48260c82a5e9fdb9ef81629e24c) - GitHub
* **ci**: Bump docker/build-push-action from 5.2.0 to 5.3.0 (#20098) [cb4a5e6](https://github.com/answerbook/vector/commit/cb4a5e6257508534295dc79c8af2768c7e74284d) - GitHub
* **ci**: Bump docker/setup-buildx-action from 3.0.0 to 3.1.0 (#19961) [cae37e9](https://github.com/answerbook/vector/commit/cae37e99d8dba79c943e9cdf6af862523141f71c) - GitHub
* **ci**: Bump docker/setup-buildx-action from 3.1.0 to 3.2.0 (#20097) [494d7e2](https://github.com/answerbook/vector/commit/494d7e2a7bff5c7bebb90925b5f451a99e3f0d5c) - GitHub
* **ci**: Bump myrotvorets/set-commit-status-action from 2.0.0 to 2.0.1 (#19924) [23ffe88](https://github.com/answerbook/vector/commit/23ffe8812cd7df603cf3cf310773ee356c96c002) - GitHub
* **ci**: Default env vars for enterprise_http_to_http regression case (#20073) [f0d3037](https://github.com/answerbook/vector/commit/f0d3037541b99bfcebfabdb1796200992f0747a8) - GitHub
* **ci**: Drop `apt-get upgrade` (#20203) [4496b3d](https://github.com/answerbook/vector/commit/4496b3dba977444e39d8fdfbba56574ea5fecb7a) - Jesse Szwedko
* **ci**: increase timeout for `cross` workflow (#20002) [f34738e](https://github.com/answerbook/vector/commit/f34738e6737e79f77dc6aa9aecb8d00430f64d99) - GitHub
* **ci**: Only use one label for selecting GHA runner (#20210) [1cb0b96](https://github.com/answerbook/vector/commit/1cb0b96459ba23aa821fee2725680a42146944c6) - Jesse Szwedko
* **ci**: peg `fakeintake` docker image (#20196) [b7495c2](https://github.com/answerbook/vector/commit/b7495c271be5138dfb677ea05cbbfad0ce623584) - Jesse Szwedko
* **ci**: Remove pip install of modules (#20204) [2cb35f1](https://github.com/answerbook/vector/commit/2cb35f197f322daec498fd763706577b224a09c4) - Jesse Szwedko
* **ci**: Use gzip compression for datadog_logs regression tests (#20020) [eb30996](https://github.com/answerbook/vector/commit/eb3099657f53c8de5584b20fbb68f05c342f93c7) - GitHub
* **cli**: Update default for --strict-env-vars to true (#20062) [a7c3dbc](https://github.com/answerbook/vector/commit/a7c3dbc453dc63dd4499b8f0c3dce15f16839f46) - GitHub
* **core**: Add missing `TraceEvent::remove` function (#20023) [0f472db](https://github.com/answerbook/vector/commit/0f472db2b153566df47caec0c50b2f26ba0a2197) - GitHub
* **core**: Remove optionality from topology controller reload (#20010) [3b6066d](https://github.com/answerbook/vector/commit/3b6066d9f93e753c0c4989173eaced46b1d2c519) - GitHub
* **dedupe transform**: expose deduping logic (#19992) [676318a](https://github.com/answerbook/vector/commit/676318aa258e9b211fd6bd8330eb900788f0473f) - GitHub
* **deps**: Bump anyhow from 1.0.79 to 1.0.80 (#19914) [7fb4513](https://github.com/answerbook/vector/commit/7fb4513424aa9c3d19fa0e43c7be2360d2ac412d) - GitHub
* **deps**: Bump anyhow from 1.0.80 to 1.0.81 (#20066) [fe23c97](https://github.com/answerbook/vector/commit/fe23c97ae6a45115c9924a3ea6410c62018c5060) - GitHub
* **deps**: Bump arc-swap from 1.6.0 to 1.7.0 (#19997) [02bb9b2](https://github.com/answerbook/vector/commit/02bb9b2e7eda2326f4da9d6500c76f1b6e812b28) - GitHub
* **deps**: Bump assert_cmd from 2.0.13 to 2.0.14 (#19908) [bb4190b](https://github.com/answerbook/vector/commit/bb4190b028f24c51fa6296830aa6036f68c5596b) - GitHub
* **deps**: Bump async-recursion from 1.0.5 to 1.1.0 (#20114) [068b199](https://github.com/answerbook/vector/commit/068b19918fd723e26b9fc5c6de289493d9ad55de) - GitHub
* **deps**: Bump async-trait from 0.1.77 to 0.1.78 (#20115) [a1902c2](https://github.com/answerbook/vector/commit/a1902c2897c23e40d18dc96df333461c0f65ef4a) - GitHub
* **deps**: Bump base64 from 0.21.7 to 0.22.0 (#19999) [56f1676](https://github.com/answerbook/vector/commit/56f167629049f879429506ce34321b534cfd79da) - GitHub
* **deps**: Bump bollard from 0.15.0 to 0.16.0 (#19998) [312056c](https://github.com/answerbook/vector/commit/312056c39178c3f40369d3aeefaf059dc9611626) - GitHub
* **deps**: Bump bstr from 1.9.0 to 1.9.1 (#19946) [b9c4544](https://github.com/answerbook/vector/commit/b9c4544d83c9c4042c49b4153cb94ba062f9dfdb) - GitHub
* **deps**: Bump cached from 0.48.1 to 0.49.2 (#19948) [fb11980](https://github.com/answerbook/vector/commit/fb11980b98b5ad3358124b5ecfb24d136c6f8903) - GitHub
* **deps**: Bump cargo_toml from 0.19.1 to 0.19.2 (#20007) [d75f74c](https://github.com/answerbook/vector/commit/d75f74cd9f28621f676e5c93aefbdccd279662af) - GitHub
* **deps**: Bump chrono-tz from 0.8.5 to 0.8.6 (#19866) [99c2207](https://github.com/answerbook/vector/commit/99c2207932894d362975fa81000b4819d5e7bb52) - GitHub
* **deps**: Bump confy from 0.6.0 to 0.6.1 (#19986) [69e84b3](https://github.com/answerbook/vector/commit/69e84b335edef665264aab16a5895c3877b99b5e) - GitHub
* **deps**: Bump crc32fast from 1.3.2 to 1.4.0 (#19867) [c654207](https://github.com/answerbook/vector/commit/c654207d5a41c8ec9fff4ac497ac3cec7a40c55c) - GitHub
* **deps**: Bump darling from 0.20.5 to 0.20.6 (#19882) [342b48c](https://github.com/answerbook/vector/commit/342b48c0f7c0aa1147a3a2a1b00089a482436560) - GitHub
* **deps**: Bump darling from 0.20.6 to 0.20.8 (#19949) [3091443](https://github.com/answerbook/vector/commit/3091443aa82b31ba04ecd3727c1f6bb37a6abbb0) - GitHub
* **deps**: Bump dyn-clone from 1.0.16 to 1.0.17 (#19954) [565d93d](https://github.com/answerbook/vector/commit/565d93d35cca13c77e3105e6fa376761b23251d2) - GitHub
* **deps**: Bump enumflags2 from 0.7.8 to 0.7.9 (#19870) [f920675](https://github.com/answerbook/vector/commit/f920675d2658d5ea410847390d7ba3be435a932a) - GitHub
* **deps**: Bump h2 from 0.4.2 to 0.4.3 (#20110) [3f83ea3](https://github.com/answerbook/vector/commit/3f83ea32e06c8e3575e6b82bdf8e25a7eb97dcc0) - GitHub
* **deps**: Bump indexmap from 2.2.3 to 2.2.5 (#19987) [e2d8ad4](https://github.com/answerbook/vector/commit/e2d8ad468ba7fa96598cf8cd3cc80641861d8b30) - GitHub
* **deps**: Bump log from 0.4.20 to 0.4.21 (#19977) [9acc151](https://github.com/answerbook/vector/commit/9acc151516e8db9b8798eb80b10cee8f843b6da7) - GitHub
* **deps**: Bump lru from 0.12.2 to 0.12.3 (#19945) [ae5b06b](https://github.com/answerbook/vector/commit/ae5b06bff08d062216a1beab2f764b6b39b04b71) - GitHub
* **deps**: Bump mlua from 0.9.5 to 0.9.6 (#19985) [6ef5092](https://github.com/answerbook/vector/commit/6ef50922b302519518937008b99cba9f97a7283c) - GitHub
* **deps**: Bump mock_instant from 0.3.1 to 0.3.2 (#19900) [6a76be2](https://github.com/answerbook/vector/commit/6a76be2173ad5a3d919e20e0661a7f3fc543427d) - GitHub
* **deps**: Bump mongodb from 2.8.1 to 2.8.2 (#20117) [5c33628](https://github.com/answerbook/vector/commit/5c33628279443068365616783b6a2d5466e8a548) - GitHub
* **deps**: Bump MSRV from 1.71.1 to 1.74 (#19884) [448c9d1](https://github.com/answerbook/vector/commit/448c9d19148c3707af54c7e2be90440de3a0316c) - GitHub
* **deps**: Bump opendal from 0.45.0 to 0.45.1 (#19996) [4677102](https://github.com/answerbook/vector/commit/4677102f189dfb9f3f63ea2f03ad4008fa01b30e) - GitHub
* **deps**: Bump openssl from 0.10.63 to 0.10.64 (#19906) [4634e2f](https://github.com/answerbook/vector/commit/4634e2f167f47c6f9cfe0221cb7238b976f76091) - GitHub
* **deps**: Bump openssl-src from 300.2.2+3.2.1 to 300.2.3+3.2.1 (#19869) [4f0dbf4](https://github.com/answerbook/vector/commit/4f0dbf4d2792dc266e0b9ea74158a6a96a1adccb) - GitHub
* **deps**: Bump os_info from 3.7.0 to 3.8.0 (#20082) [d23730e](https://github.com/answerbook/vector/commit/d23730e3138c20fac276178357234135f1fc52bd) - GitHub
* **deps**: Bump os_info from 3.8.0 to 3.8.1 (#20112) [7e3e60f](https://github.com/answerbook/vector/commit/7e3e60fa447eab3b73f27e2c98ed1f2c4d19fe94) - GitHub
* **deps**: Bump pin-project from 1.1.4 to 1.1.5 (#20015) [cbebdb2](https://github.com/answerbook/vector/commit/cbebdb2689600b8515dc34430703c8281cf7caa0) - GitHub
* **deps**: Bump proc-macro2 from 1.0.78 to 1.0.79 (#20070) [8811e21](https://github.com/answerbook/vector/commit/8811e218d9d691d0d5e600d0cd2cd50cacb02c0a) - GitHub
* **deps**: Bump ratatui from 0.26.0 to 0.26.1 (#19868) [0922c3f](https://github.com/answerbook/vector/commit/0922c3f67f57e2d8c29029a91e1f60ab4d699f50) - GitHub
* **deps**: Bump reqwest from 0.11.24 to 0.11.26 (#20080) [c62ec39](https://github.com/answerbook/vector/commit/c62ec39ab159b964ec0069db5b528f0954a66c43) - GitHub
* **deps**: Bump roaring from 0.10.2 to 0.10.3 (#19889) [8223dca](https://github.com/answerbook/vector/commit/8223dca26efd790ec4fdbf5eb7626f2cc32d99a2) - GitHub
* **deps**: Bump rumqttc from 0.23.0 to 0.24.0 (#19967) [d4cf2bf](https://github.com/answerbook/vector/commit/d4cf2bf6989eee92a41e7312b63b8522fdb0444b) - GitHub
* **deps**: Bump Rust to 1.77.0 (#20149) [abd776d](https://github.com/answerbook/vector/commit/abd776d7c74ae48968fa34829d3683f68115a9e0) - GitHub
* **deps**: Bump ryu from 1.0.16 to 1.0.17 (#19912) [1d91742](https://github.com/answerbook/vector/commit/1d91742e70a3c5ef4ae3a86c26a6d89846e35157) - GitHub
* **deps**: Bump semver from 1.0.21 to 1.0.22 (#19911) [b8d89a0](https://github.com/answerbook/vector/commit/b8d89a03459a32f9c227b6fab21b5081c75d934f) - GitHub
* **deps**: Bump serde from 1.0.196 to 1.0.197 (#19910) [7311c0a](https://github.com/answerbook/vector/commit/7311c0aaa01cac20d4cdc71c21c516de7326405c) - GitHub
* **deps**: Bump serde_json from 1.0.113 to 1.0.114 (#19909) [4cd4b6a](https://github.com/answerbook/vector/commit/4cd4b6a26de5f70a687b934df7193aa9ba2d46f7) - GitHub
* **deps**: Bump serde_with from 3.6.1 to 3.7.0 (#20068) [98df316](https://github.com/answerbook/vector/commit/98df316fedbdffcf475b3ca9c51ab5ad4bdaa1ae) - GitHub
* **deps**: Bump serde_yaml from 0.9.31 to 0.9.32 (#19907) [837c64c](https://github.com/answerbook/vector/commit/837c64cffd3624e32178a1e5078ed5ed3e6ebc8a) - GitHub
* **deps**: Bump serde_yaml from 0.9.32 to 0.9.33 (#20116) [3e8c6a4](https://github.com/answerbook/vector/commit/3e8c6a48451233fb7b60b4ca0a5139986745f80e) - GitHub
* **deps**: Bump serde-toml-merge from 0.3.4 to 0.3.5 (#20081) [62de421](https://github.com/answerbook/vector/commit/62de4218e00a9907bc3c79b9e36c01066b772bb5) - GitHub
* **deps**: Bump serde-toml-merge from 0.3.5 to 0.3.6 (#20132) [e012a80](https://github.com/answerbook/vector/commit/e012a80bb5d8e4f318fb4408d9e2ab6242a8883b) - GitHub
* **deps**: Bump serde-wasm-bindgen from 0.6.3 to 0.6.4 (#19934) [5d03bf0](https://github.com/answerbook/vector/commit/5d03bf0e00b3f235cd2dfa9c88e77d7a162c0180) - GitHub
* **deps**: Bump serde-wasm-bindgen from 0.6.4 to 0.6.5 (#19966) [c1d6529](https://github.com/answerbook/vector/commit/c1d6529225b3c9dd1c3e00957361acab89fa4d50) - GitHub
* **deps**: Bump smallvec from 1.13.1 to 1.13.2 (#20145) [04bff91](https://github.com/answerbook/vector/commit/04bff918cfcba087c18766ef81a8e2316b8790f4) - GitHub
* **deps**: Bump socket2 from 0.5.5 to 0.5.6 (#19947) [7bb9716](https://github.com/answerbook/vector/commit/7bb9716ebc46bb2842e8df4b2c20775c1897d631) - GitHub
* **deps**: Bump syn from 2.0.48 to 2.0.49 (#19890) [2b0f06e](https://github.com/answerbook/vector/commit/2b0f06eb5de6dc008bd4c98e49ce82a5f0837942) - GitHub
* **deps**: Bump syn from 2.0.49 to 2.0.50 (#19913) [c9e2400](https://github.com/answerbook/vector/commit/c9e24003095f3a6271aa9a3d50c83c3b6f857014) - GitHub
* **deps**: Bump syn from 2.0.50 to 2.0.51 (#19953) [5f43cde](https://github.com/answerbook/vector/commit/5f43cde7aa6165e55091ec8372e301a03426a3e5) - GitHub
* **deps**: Bump syn from 2.0.51 to 2.0.52 (#19979) [29a9167](https://github.com/answerbook/vector/commit/29a9167c8554befaa5a56a188b3c44e18d08c638) - GitHub
* **deps**: Bump syn from 2.0.52 to 2.0.53 (#20111) [8737b24](https://github.com/answerbook/vector/commit/8737b24807ee6b00a20663f951ec0ce53682530e) - GitHub
* **deps**: Bump tempfile from 3.10.0 to 3.10.1 (#19955) [b1a2ca1](https://github.com/answerbook/vector/commit/b1a2ca11c156aa9f66125c56009e7f05bbe65d2f) - GitHub
* **deps**: Bump the aws group with 1 update (#19919) [78f0e31](https://github.com/answerbook/vector/commit/78f0e31c8445355203fb5295224af7da1de19e1b) - GitHub
* **deps**: Bump the aws group with 1 update (#19965) [26ec8f4](https://github.com/answerbook/vector/commit/26ec8f432394b966e5c48da97634738f30c949d7) - GitHub
* **deps**: Bump the aws group with 1 update (#20089) [8860644](https://github.com/answerbook/vector/commit/88606447dd9f874f27f06dc17c3e2f0b2083e221) - GitHub
* **deps**: Bump the aws group with 2 updates (#19848) [e0d5f1e](https://github.com/answerbook/vector/commit/e0d5f1e4dbd433165c525e941c95dd8eea2ebee6) - GitHub
* **deps**: Bump the aws group with 2 updates (#20129) [2a88fc0](https://github.com/answerbook/vector/commit/2a88fc06b7c958f9787a3e050c677cbe5860d62d) - GitHub
* **deps**: Bump the aws group with 3 updates (#19976) [8ca10a0](https://github.com/answerbook/vector/commit/8ca10a0232889fc8195911409d78469e50e76e12) - GitHub
* **deps**: Bump the aws group with 4 updates (#19888) [788f0c3](https://github.com/answerbook/vector/commit/788f0c30ee259d5e918be074d059085107bd69bc) - GitHub
* **deps**: Bump the aws group with 4 updates (#20079) [de4687f](https://github.com/answerbook/vector/commit/de4687ff51eda7c67a66ebe86138ab9ad7ceb54c) - GitHub
* **deps**: Bump the aws group with 6 updates (#19936) [e2e5253](https://github.com/answerbook/vector/commit/e2e5253ff42339f8c66226580a8aadf9b729e10d) - GitHub
* **deps**: Bump the clap group with 1 update (#20026) [c83e36d](https://github.com/answerbook/vector/commit/c83e36dd447ef9a4ebe8270bc295743ca3053bb6) - GitHub
* **deps**: Bump the clap group with 1 update (#20108) [7c9b4c5](https://github.com/answerbook/vector/commit/7c9b4c59c06a49c46e1f0f84faa6114dcce5c642) - GitHub
* **deps**: Bump the clap group with 3 updates (#19899) [a32895e](https://github.com/answerbook/vector/commit/a32895ec096c5c55c449c8d3ad6bed658d69b71b) - GitHub
* **deps**: Bump the graphql group with 2 updates (#20107) [80f63bb](https://github.com/answerbook/vector/commit/80f63bb6b52561ae4a9f98783ae98472c0798845) - GitHub
* **deps**: Bump thiserror from 1.0.57 to 1.0.58 (#20069) [34d3aa5](https://github.com/answerbook/vector/commit/34d3aa5b23b859d0e9e0c566c2ae3ec5bf79ceca) - GitHub
* **deps**: Bump tokio-stream from 0.1.14 to 0.1.15 (#20100) [4c7bece](https://github.com/answerbook/vector/commit/4c7becebe8ec38f2a60d25a97bafa3d6c9a12fd7) - GitHub
* **deps**: Bump tokio-test from 0.4.3 to 0.4.4 (#20101) [ad8a869](https://github.com/answerbook/vector/commit/ad8a8690b7707540dd24a85e8ada8c51bab150fe) - GitHub
* **deps**: Bump toml from 0.8.10 to 0.8.11 (#20067) [bcc6e40](https://github.com/answerbook/vector/commit/bcc6e40862ee16f4cec75b8f752c54a399bd6cbc) - GitHub
* **deps**: Bump toml from 0.8.11 to 0.8.12 (#20130) [20e56d3](https://github.com/answerbook/vector/commit/20e56d3080ec3cb04c750966c2722799ed920225) - GitHub
* **deps**: Bump typetag from 0.2.15 to 0.2.16 (#19956) [906cd65](https://github.com/answerbook/vector/commit/906cd65bb315cf658cc6c8a597c93e34de228d74) - GitHub
* **deps**: Bump uuid from 1.7.0 to 1.8.0 (#20131) [b184196](https://github.com/answerbook/vector/commit/b184196d9760539db31a5238ee7b7254329b7c8d) - GitHub
* **deps**: Bump VRL to v0.13.0 (#20126) [62297dc](https://github.com/answerbook/vector/commit/62297dcb8caba651ed60f154c36b5a4e1a63046b) - GitHub
* **deps**: Bump wasm-bindgen from 0.2.91 to 0.2.92 (#20009) [c1141f9](https://github.com/answerbook/vector/commit/c1141f9288007ec79c140d551a5ddfef483c40c5) - GitHub
* **deps**: Bump whoami to 1.5.0 (#20018) [3a495e3](https://github.com/answerbook/vector/commit/3a495e35d95c040ccc629f3ac1c2f8d696f1404a) - GitHub
* **deps**: Update h2 (#20236) [53c9a7d](https://github.com/answerbook/vector/commit/53c9a7d4e174316102588945da9091b6c58fde4e) - Jesse Szwedko
* **deps**: Update lockfree-object-pool to 0.1.5 (#20001) [9bf1872](https://github.com/answerbook/vector/commit/9bf1872fa88ae94e99e01d87696ed294ad4a3da0) - GitHub
* **deps**: Update mio (#20005) [a59aeb9](https://github.com/answerbook/vector/commit/a59aeb921bc93bc7590265f9e4335a8d824b95b4) - GitHub
* **deps**: Update VRL to v0.12.0 (#20037) [55a962a](https://github.com/answerbook/vector/commit/55a962a3c55d7b9437ec6b4b36ca42172bc9b953) - GitHub
* **dev**: Add a note that GH usernames shouldn't start with @ (#19859) [7c00726](https://github.com/answerbook/vector/commit/7c0072689fba435640e26e63d46343064c477b0f) - GitHub
* **dev**: Remove mention of handwriting changelog for patch release (#20040) [37a19fa](https://github.com/answerbook/vector/commit/37a19fab442b06be3dc73c6962578e2f083f9d88) - GitHub
* **dev**: Update changelog generation script to handle authors and whitespace (#20075) [6a6c159](https://github.com/answerbook/vector/commit/6a6c159da14b441df6dde0a3a9997a787910087a) - GitHub
* **dev**: Update CODEOWNERS to reflect consolidation (#20087) [ccaa7e3](https://github.com/answerbook/vector/commit/ccaa7e376d0167d187573c4b9b478f1c2778e359) - GitHub
* **dev**: Update CONTRIBUTING.md docs regarding how to have website… (#19926) [a68a0b5](https://github.com/answerbook/vector/commit/a68a0b5c6a1ddd33682b578163727403dd9ef296) - GitHub
* **dev**: Update release instructions for deploying vector.dev (#19925) [282a58d](https://github.com/answerbook/vector/commit/282a58d410a05f2bf0def7cfcca98e84342134ff) - GitHub
* **docs**: note for 0.37 about incorrect ddtags parsing behavior (#20186) [dd984ea](https://github.com/answerbook/vector/commit/dd984ea225b453c5a22f59cbff87f1fa6919237a) - Jesse Szwedko
* **docs**: Remove package deprecation banner (#20181) [716160d](https://github.com/answerbook/vector/commit/716160dba256255bd43a897ec57ca8359ec44f0c) - Jesse Szwedko
* **docs**: Update banner to use past tense for repository decommissioning (#20059) [38acf37](https://github.com/answerbook/vector/commit/38acf37f1d5d33f46af93f24034475e450f04b29) - GitHub
* **kubernetes**: Bump manifists to chart v0.30.2 (#19860) [1637e56](https://github.com/answerbook/vector/commit/1637e566c08f5dc2b09e5c85ad49a93762647c06) - GitHub
* **observability**: add component spec validation tests for `datadog_logs` sink (#19887) [ea377f0](https://github.com/answerbook/vector/commit/ea377f007e0657d65915f90b46e602ad6a149708) - GitHub
* **observability**: extend component validation framework for more flexible test case building (#19941) [c7e4e33](https://github.com/answerbook/vector/commit/c7e4e33ca0c479cd9c8b0c5af72f6bc804d287fe) - GitHub
* **observability**: robustly synchronize component validation framework tasks (#19927) [43a9129](https://github.com/answerbook/vector/commit/43a91293c61e67305ee175e3cf135adeec0b51b1) - GitHub
* **releases website**: 0.36 changelog fixes (#19875) [881077e](https://github.com/answerbook/vector/commit/881077e26145c853d7680993c588e4c260346deb) - GitHub
* **releasing**: Add additional note about new VRL decoder [2014b53](https://github.com/answerbook/vector/commit/2014b536f0a0af911f874802ed7fbf5237af009b) - Jesse Szwedko
* **releasing**: Add missing changelog entries (#20041) [6731b44](https://github.com/answerbook/vector/commit/6731b44d2ff7c91b37120bb6e7b0433f540d23aa) - GitHub
* **releasing**: Bump development version to v0.37.0 (#19874) [b91be34](https://github.com/answerbook/vector/commit/b91be34a3c890505e7faeaeffa4a1bea54944ebf) - GitHub
* **releasing**: Fix formatting of v0.36.1 release [7c596b4](https://github.com/answerbook/vector/commit/7c596b4825e26c5640d4fe4e96ca7e6a471f57cc) - Jesse Szwedko
* **releasing**: Fix markdown formatting of v0.36.0 description [cd6bbae](https://github.com/answerbook/vector/commit/cd6bbae5f3300acbc5607be2eaf98d718fe14ce3) - Jesse Szwedko
* **releasing**: Fix markdown formatting of v0.36.0 release description [800173f](https://github.com/answerbook/vector/commit/800173f9c57087c414059ba84d86105054e23781) - Jesse Szwedko
* **releasing**: Prepare v0.36.0 release [753466f](https://github.com/answerbook/vector/commit/753466fb4bf663796854156c0ddaedaf0cc9bc9c) - Jesse Szwedko
* **releasing**: Prepare v0.36.1 release [7e01104](https://github.com/answerbook/vector/commit/7e011047c39d2e15e5d0f41a2605c843c1189b97) - Jesse Szwedko
* **releasing**: Prepare v0.37.0 release [c1da408](https://github.com/answerbook/vector/commit/c1da408b34fe29f8c949e18f08867066b080b2f5) - Jesse Szwedko
* **releasing**: Prepare v0.37.1 release [cb6635a](https://github.com/answerbook/vector/commit/cb6635acfce641668d6640ec37c0d62773be7b03) - Jesse Szwedko
* **releasing**: Regenerate k8s manifests for Helm chart v0.31.1 (#20060) [b35eaf5](https://github.com/answerbook/vector/commit/b35eaf53315532a7668cd36342f72af2d4e00488) - GitHub
* remove repetitive words (#20091) [fafe8c5](https://github.com/answerbook/vector/commit/fafe8c50a4721fa3ddbea34e0641d3c145f14388) - GitHub
* Remove used changelog entries [c0fe642](https://github.com/answerbook/vector/commit/c0fe642ec4186e1f0f53c6cb93451fa83cee10d5) - Jesse Szwedko
* **testing**: expose component validation framework (#19964) [c71d5d1](https://github.com/answerbook/vector/commit/c71d5d16493f1662187ed6e7a11c8a88fbc4e133) - GitHub
* **testing**: further adjustments to component validation framework (#20043) [12c1866](https://github.com/answerbook/vector/commit/12c1866214e55869275afa5fc0741f2af8baa0fd) - GitHub
* **testing**: only compile ValidatableComponent in test runs (#20024) [a3bedbd](https://github.com/answerbook/vector/commit/a3bedbd70b6b297e3d7cf9868a7c82f87a86d548) - GitHub
* **tests**: caller resolves the component validation framework test case path (#20021) [44ed0d1](https://github.com/answerbook/vector/commit/44ed0d146e274c9593db17f8e9fe74de3833e58f) - GitHub
* **tests**: expose more test utils (#19885) [c890997](https://github.com/answerbook/vector/commit/c89099768af4ee63542dcb8c039e35bd7a6f2832) - GitHub
* **tests**: expose test utilities (#19894) [695f847](https://github.com/answerbook/vector/commit/695f847d1711923261acdec0ad029185c7826521) - GitHub
* **tests**: expose test utils (feature flag) (#19863) [e8401c4](https://github.com/answerbook/vector/commit/e8401c473fb0334c36ac91a411392f1ac7ae9ce5) - GitHub
* **vrl stdlib**: Fix redact doc URL templating [63a5074](https://github.com/answerbook/vector/commit/63a50740746325a4320a7c0e8da0b79dd2df3521) - Jesse Szwedko
* **website**: bump openssl version used for links in docs (#19880) [3cb9272](https://github.com/answerbook/vector/commit/3cb92727ab948c26d8df792eea1c237c0a44bc70) - GitHub


### Features

* **component validation**: add sink error path validation + multi config (#18062) [a6da1d8](https://github.com/answerbook/vector/commit/a6da1d8f4357513161520ae4c9fac96859d7de24) - GitHub
* **dnsmsg_parser**: add support for EDNS EDE fields (#19937) [070e38c](https://github.com/answerbook/vector/commit/070e38c555d7a7aaf9dda67e7dd468cfbfb949b9) - GitHub
* **dnsmsg_parser**: add support for more record types (HINFO, CSYNC, OPT, missing DNSSEC types) (#19921) [482ed3c](https://github.com/answerbook/vector/commit/482ed3cb7a9de9763d7e623c8a691ac4d9911638) - GitHub
* **enrichment_tables**: add support for custom MMDB types (#20054) [d511e89](https://github.com/answerbook/vector/commit/d511e893ad0e594231e06f25a9d35ab70248bedc) - GitHub
* **greptimedb sink**: improve tls support for greptimedb sink (#20006) [cbcb874](https://github.com/answerbook/vector/commit/cbcb874a9944801e8a89d42e44ecf551db55071a) - GitHub
* **kubernetes**: Bump manifests to chart v0.36.0 (#19877) [a935c30](https://github.com/answerbook/vector/commit/a935c30785ad50adfea5a3344e2fb3673fffb73c) - GitHub
* **mqtt sink**: add MQTT sink (#19813) [f88316c](https://github.com/answerbook/vector/commit/f88316cce7665c6dbf83a81a8261fa126b50542e) - GitHub
* **platforms**: Add ARMv6 builds (#19192) [e9815e1](https://github.com/answerbook/vector/commit/e9815e1f328a4ef59099c3d07918f167947c2e1f) - GitHub
* **sources**: add `lowercase_hostnames` option to `dnstap` source (#20035) [485dea7](https://github.com/answerbook/vector/commit/485dea71725511b997586698650e202add499183) - GitHub
* **sources**: add `permit_origin` config option for all tcp sources (#20051) [aa04ac8](https://github.com/answerbook/vector/commit/aa04ac86707ee0f1df8e7b77acbd459834ca1fa4) - GitHub
* **sources**: add TCP mode to DNSTAP source (#19892) [eb690d4](https://github.com/answerbook/vector/commit/eb690d4343e74078e4debd9f9984bcf0e89ad8a5) - GitHub
* **sources**: Initial pulsar source (#18475) [bb1b857](https://github.com/answerbook/vector/commit/bb1b8571070f38f7eee385dad92807249236d063) - GitHub
* **vrl**: add `uuid_v7` function (#20048) [314ea36](https://github.com/answerbook/vector/commit/314ea367302fb95a3ec0c2fcdfbe19df6a0e7603) - GitHub
* **website**: integrate Cargo package dependency info (#19933) [bd2f0a3](https://github.com/answerbook/vector/commit/bd2f0a33e75e624bb75cb2c311bcbfa620ab699a) - GitHub


### Miscellaneous

* Merge pull request #608 from answerbook/feature/LOG-20555 [701ca82](https://github.com/answerbook/vector/commit/701ca82734bacb2f5e5f36a250bc35fbcb6f24db) - GitHub [LOG-20555](https://logdna.atlassian.net/browse/LOG-20555)
* Biblicalph/log 20630 (#606) [7670d80](https://github.com/answerbook/vector/commit/7670d8067e2503036a6848d3cb364a348c2c9688) - GitHub [LOG-20630](https://logdna.atlassian.net/browse/LOG-20630) [LOG-20630](https://logdna.atlassian.net/browse/LOG-20630)
* Merge remote-tracking branch 'origin/master' into feature/LOG-20555 [671beb0](https://github.com/answerbook/vector/commit/671beb082d5837dac6a37c9d2db77be136a9ac69) - Chris Nixon [LOG-20555](https://logdna.atlassian.net/browse/LOG-20555)
* Merge remote-tracking branch 'origin/master' into upstream-0.37 [f2fd433](https://github.com/answerbook/vector/commit/f2fd433990ff84a5ccb2de75f2b2bc6d0f86d52f) - Chris Nixon
* Merge upstream/v0.37 into upstream-merge-0.37 [22402f2](https://github.com/answerbook/vector/commit/22402f29ffd09367cd1dae13ec2e61746970b86c) - Chris Nixon
* Merge upstream/v0.36 into merge-vector-0.36 [c2aee34](https://github.com/answerbook/vector/commit/c2aee34d75040a62bf216a0566c90c5802d96e58) - Chris Nixon
* fix (aws service): use http client so we can use openssl tls. (#19939) [2ca14ae](https://github.com/answerbook/vector/commit/2ca14aef0ee056f8f1a0763abf2859a75cad5f9c) - GitHub
* Add pre-requisite for vdev (#19668) [3f59886](https://github.com/answerbook/vector/commit/3f59886a39321570e459ba65469d933a968876f2) - GitHub
* **datadog_agent source**: add `parse_ddtags` config setting to parse the `ddtags` log event field into an object (#20003) [d5c8a77](https://github.com/answerbook/vector/commit/d5c8a77b5751c4d2277cee6ee76a1903873c5873) - GitHub
* **dnstap source, releasing**: Add breaking change note for dnstap source mode (#20202) [a81f3b3](https://github.com/answerbook/vector/commit/a81f3b3c5039e818989d425ed787e885304c8df1) - Jesse Szwedko
* **file source, kubernetes_logs source**: add rotate_wait_ms config option (#18904) [0a89cb1](https://github.com/answerbook/vector/commit/0a89cb13714876da089ea09d4881e98a890b3976) - GitHub
* fix type cardinality docs (#20209) [3f4859f](https://github.com/answerbook/vector/commit/3f4859f86cb345149a167c0aaef38b8bda2ec912) - Jesse Szwedko
* **http sink**: update HTTP request builder to return error (#19886) [a798f68](https://github.com/answerbook/vector/commit/a798f681d392e761d3e1e185ca9d7e8075a892c5) - GitHub
* **http sink**: Update HttpRequest struct to pass additional metadata (#19780) [9a0a5e4](https://github.com/answerbook/vector/commit/9a0a5e4784bf80af8be7a7e8cfa8516a70d39704) - GitHub
* **kubernetes**: add support for include_paths_glob_patterns (#19521) [a7fe0db](https://github.com/answerbook/vector/commit/a7fe0dbfbd41197bb09fb6a8f2d8562a22384c99) - GitHub
* **remap transform**: Fix `drop_on_abort` docs (#19918) [1470f1a](https://github.com/answerbook/vector/commit/1470f1ada2bbf71cfbe8fe9da683315e5472bebf) - GitHub
* **remap**: do not filter out file contents from error logs (#20125) [5e7248c](https://github.com/answerbook/vector/commit/5e7248cfaa787126cb7654e0523d6ced8c06f245) - GitHub
* **s3 sink**: add express one zone storage class (#19893) [5d8160d](https://github.com/answerbook/vector/commit/5d8160d72743df1e02fff9f69a8d4e37e1f2577a) - GitHub
* **security**: Update TLS docs for `verify_certificate` (#20153) [4d23e66](https://github.com/answerbook/vector/commit/4d23e66dc22c499ad8263b937c21800d1b68d1c7) - GitHub
* **statsd source**: Update statsd doc to mention timing conversion (#20033) [d505045](https://github.com/answerbook/vector/commit/d505045620cc5272be54b42fdd01abb8c0486d50) - GitHub
* **vrl**: add docs for new validate flag in punycode functions (#19923) [6d09613](https://github.com/answerbook/vector/commit/6d0961347b7c36115da101ab993f66a532493a16) - GitHub
* **vrl**: add documentation for `sieve` function (#20000) [4c68f96](https://github.com/answerbook/vector/commit/4c68f9699749d17fa926983e2a90bdeec92b112a) - GitHub
* **vrl**: Add documentation for parse_proto and encode_proto (#20139) [4279bf0](https://github.com/answerbook/vector/commit/4279bf0018055de68f59dffe9532fab96c80d3ac) - GitHub
* **vrl**: RFC for return expression (#19828) [2f1c785](https://github.com/answerbook/vector/commit/2f1c7850fbc039a894f51b844e919adf2fdc925d) - GitHub


### Tests

* **dnsmsg_parser**: fix tests for currently unknown rdata types (#20052) [04f7858](https://github.com/answerbook/vector/commit/04f78584d7dd10e98d81e3065fbb17483009d60f) - GitHub


### **BREAKING CHANGES**

* **cli:** Update default for --strict-env-vars to true (#20062)
* **sources:** add TCP mode to DNSTAP source (#19892)

## [4.6.1](https://github.com/answerbook/vector/compare/v4.6.0...v4.6.1) (2024-09-19)


### Bug Fixes

* Make cardinality limit config value a string (#605) [c975b89](https://github.com/answerbook/vector/commit/c975b8985829d2e08dec6d238b749f42f5ed9eab) - GitHub [LOG-20630](https://logdna.atlassian.net/browse/LOG-20630)

# [4.6.0](https://github.com/answerbook/vector/compare/v4.5.2...v4.6.0) (2024-09-19)


### Features

* Read cardinality limit from env if available (#603) [7f73a92](https://github.com/answerbook/vector/commit/7f73a927fb24c4f16a6a7064e3759d0697dee5af) - GitHub [LOG-20626](https://logdna.atlassian.net/browse/LOG-20626)

## [4.5.2](https://github.com/answerbook/vector/compare/v4.5.1...v4.5.2) (2024-09-06)


### Bug Fixes

* **throttle**: limit number of events stored [083765e](https://github.com/answerbook/vector/commit/083765ec64e66f160a979942577034cc4c047fcc) - Dan Hable [LOG-20577](https://logdna.atlassian.net/browse/LOG-20577)

## [4.5.1](https://github.com/answerbook/vector/compare/v4.5.0...v4.5.1) (2024-08-15)


### Chores

* disable dependabot [12cda61](https://github.com/answerbook/vector/commit/12cda614635af474fb021c8c6e2451726aca7b68) - Mike Del Tito [LOG-20473](https://logdna.atlassian.net/browse/LOG-20473)


### Miscellaneous

* Merge pull request #597 from answerbook/mdeltito/disable-dependabot [6f06adf](https://github.com/answerbook/vector/commit/6f06adfc2fea0397b4ecd53636925b8075e7f1cb) - GitHub

# [4.5.0](https://github.com/answerbook/vector/compare/v4.4.0...v4.5.0) (2024-08-15)


### Chores

* fix formatting [940b25c](https://github.com/answerbook/vector/commit/940b25c067ef8fc8904a511bcf29b1888d5cdffe) - Mike Del Tito [LOG-20433](https://logdna.atlassian.net/browse/LOG-20433)


### Code Refactoring

* move user_log and vrl functions into `mezmo` crate [f79c8b9](https://github.com/answerbook/vector/commit/f79c8b9920e4c21ef94d8760b9f0f6b0f4a6e6e8) - Mike Del Tito [LOG-20433](https://logdna.atlassian.net/browse/LOG-20433)


### Features

* **codecs**: wire up mezmo vrl functions to vrl decoder [124ad1c](https://github.com/answerbook/vector/commit/124ad1cbc2851a62eecdc2d04381eb63a1b65a1c) - Mike Del Tito [LOG-20433](https://logdna.atlassian.net/browse/LOG-20433)


### Miscellaneous

* Merge pull request #595 from answerbook/mdeltito/LOG-20433 [e438399](https://github.com/answerbook/vector/commit/e438399ceab6b92ff2af4eebdf76532585fcf739) - GitHub [LOG-20433](https://logdna.atlassian.net/browse/LOG-20433)

# [4.4.0](https://github.com/answerbook/vector/compare/v4.3.0...v4.4.0) (2024-08-14)


### Features

* **data-profiler**: usage metrics tracking [f34edd7](https://github.com/answerbook/vector/commit/f34edd7d01c6a27a7d952e30e0570bc983a6e86b) - dominic-mcallister-logdna [LOG-20462](https://logdna.atlassian.net/browse/LOG-20462)


### Miscellaneous

* Merge pull request #594 from answerbook/dominic/LOG-20462 [2f7bcd4](https://github.com/answerbook/vector/commit/2f7bcd4d2153bfe5c9a87858ab69f96bd828cd3c) - GitHub [LOG-20462](https://logdna.atlassian.net/browse/LOG-20462)
* Revert "chore: revert track active profilers with each revision" [22ce274](https://github.com/answerbook/vector/commit/22ce2747ca30261c6bb037ded858f0fac71a3df2) - dominic-mcallister-logdna

# [4.3.0](https://github.com/answerbook/vector/compare/v4.2.2...v4.3.0) (2024-08-14)


### Bug Fixes

* add write perms to the default data_dir (#19659) [2adf672](https://github.com/answerbook/vector/commit/2adf6726906b54e4ef30524b635830a860590310) - GitHub
* **aws provider**: Enable `credentials-process` for `aws-config` (#20030) [28760fb](https://github.com/answerbook/vector/commit/28760fbcdade2353feb506a51ef7288a570d6ca6) - Jesse Szwedko
* **aws region**: remove cfg test attribute (#19684) [8a82a3b](https://github.com/answerbook/vector/commit/8a82a3b1347c25efbb06b9ad300fd9d7a779b202) - GitHub
* **aws service**: determine region using our http client (#19972) [9def84e](https://github.com/answerbook/vector/commit/9def84e0de3831f0add61c9b2cb4e880fcf8aa7d) - Jesse Szwedko
* **aws_s3 sink**: remove trailing dot from s3 filename extension (#19616) [20b4fc7](https://github.com/answerbook/vector/commit/20b4fc72bcb8f605e044e05ae3df0e26aa637875) - GitHub
* **compression**: Fix gzip and zlib performance degradation (#20032) [6313331](https://github.com/answerbook/vector/commit/63133311baa0df60d08e22bb1e4bec858438e268) - Jesse Szwedko
* **config api**: emit graphql field of api config (#19692) [7cf2f00](https://github.com/answerbook/vector/commit/7cf2f009dbd9be4177dfbce7950cd82d57f93448) - GitHub
* **config**: acquire exclusive lock to global data dir (#19595) [58a37b2](https://github.com/answerbook/vector/commit/58a37b24dad42fc8aa0bd4737786a6aae780a3c5) - GitHub
* **datadog_agent source**: Remove warning for unused outputs when output is disabled (#19629) [50a86ef](https://github.com/answerbook/vector/commit/50a86ef4fb59b9f9ac5e3179d6e8892019d552ee) - GitHub
* **datadog_logs sink**: abort serialization and split batch when payload is too large (#19189) [9f7c92d](https://github.com/answerbook/vector/commit/9f7c92d8d4b605f14f9d65ee9f9e34dcedf297d8) - GitHub
* **docs, http_server source**: Update docs for disabling `max_connection_age_secs` (#19802) [bf1f2c7](https://github.com/answerbook/vector/commit/bf1f2c726ea04f1c858a9fd18a2a7c14dbbdeb17) - GitHub
* **http_server source**: Conditionally send Connection: Close header based on HTTP version (#19801) [86c5e54](https://github.com/answerbook/vector/commit/86c5e5475a6a6ee213198851251b8b035481a011) - GitHub
* **journald source**: correctly emit metadata to log namespace (#19812) [0e6cf3e](https://github.com/answerbook/vector/commit/0e6cf3e439e484f3e4e29d8a90b9250ebb274e95) - GitHub
* **observability**: propagate tracing span context in stream sink request building (#19712) [4195071](https://github.com/answerbook/vector/commit/4195071d984a4d2107a2f5888bca82db0bab4b5c) - GitHub
* **sample transform**: clippy lint on feature flag case (#19822) [92b83cd](https://github.com/answerbook/vector/commit/92b83cd2bea0c075134ea33bb2b204d333e4f27e) - GitHub


### Chores

* **ci**: Bump actions/cache from 3 to 4 (#19642) [9b024b9](https://github.com/answerbook/vector/commit/9b024b9564b24524ce9a305b3c00080779f63250) - GitHub
* **ci**: Bump aws-actions/configure-aws-credentials from 4.0.1 to 4.0.2 (#19823) [fa2c194](https://github.com/answerbook/vector/commit/fa2c1941b3cf98316a94575a5faa9f0a025e8a9c) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.28.1 to 1.29.0 (#19709) [a4aff31](https://github.com/answerbook/vector/commit/a4aff31d54a3c820f50aa94acef66e0938f3c77e) - GitHub
* **ci**: Bump docker/metadata-action from 5.4.0 to 5.5.0 (#19526) [b8c268c](https://github.com/answerbook/vector/commit/b8c268cf7e8853b41b50285f8959f87a99939f01) - GitHub
* **ci**: Bump docker/metadata-action from 5.5.0 to 5.5.1 (#19755) [00a9480](https://github.com/answerbook/vector/commit/00a94801025a215a78ce684422b0a986727ccc50) - GitHub
* **ci**: Bump dorny/paths-filter from 2 to 3 (#19708) [b72217c](https://github.com/answerbook/vector/commit/b72217cf40d3216625cf274fe79b669f823a1c8a) - GitHub
* **ci**: Bump dorny/paths-filter from 2 to 3 (#19768) [a247c51](https://github.com/answerbook/vector/commit/a247c515f768ef2293821e802ec3c7793cd5a1d5) - GitHub
* **ci**: Bump nick-fields/retry from 2 to 3 (#19756) [abb292a](https://github.com/answerbook/vector/commit/abb292a8c6179eb5650cc2a88f18897aa71509cf) - GitHub
* **ci**: Bump peter-evans/create-or-update-comment from 3 to 4 (#19710) [cacba25](https://github.com/answerbook/vector/commit/cacba25ea31a394663169b253dba747f6f8a89f6) - GitHub
* **ci**: checkout full depth for changelog workflow (#19844) [4ab4c4a](https://github.com/answerbook/vector/commit/4ab4c4a3c846f3d295feb890e923e9116a0b0441) - GitHub
* **ci**: enable running all int tests comment (#19581) [f4ad8bf](https://github.com/answerbook/vector/commit/f4ad8bf0978b4524305dbfddf77609cdedf8e92a) - GitHub
* **ci**: Ensure PR runs of regression and k8s e2e tests don't cancel each other (#19578) [5fb8efc](https://github.com/answerbook/vector/commit/5fb8efcef24f231589e63e16b420f7f42dda7813) - GitHub
* **ci**: exclude dependabot from changelog job steps (#19545) [9c832fd](https://github.com/answerbook/vector/commit/9c832fd2f8677ddceb15e2e3a8e5a504b1b1cea3) - GitHub
* **ci**: fix and simplify concurrency groups (#19630) [b2c9f27](https://github.com/answerbook/vector/commit/b2c9f27d4360cbdb211d9f7230ae90e6becfee8d) - GitHub
* **ci**: fix changelog workflow extern contribs (#19524) [81d22b3](https://github.com/answerbook/vector/commit/81d22b30e20ba9a250b4d9a5d56aa4216fcd7ece) - GitHub
* **ci**: fix filter out PRs for gardener issue comment workflow (#19618) [628d207](https://github.com/answerbook/vector/commit/628d207bf4769ebd0bbf2b98ddbbf162ebd5be14) - GitHub
* **ci**: Fix the check for external contributor author GH usernames (#19568) [f914cf6](https://github.com/answerbook/vector/commit/f914cf602e78685804efaf473a056bb87f612110) - GitHub
* **ci**: Look at merge base when looking for added changelog files (#19835) [7c3f91b](https://github.com/answerbook/vector/commit/7c3f91b3de204adcc154b9b0bcad1f5a85741ee3) - GitHub
* **ci**: Reduce test timeout to 2 minutes (#19845) [76ab88d](https://github.com/answerbook/vector/commit/76ab88dfcb51014986bed948f499cd51c5582bf4) - GitHub
* **ci**: Run the changelog check on the merge queue to pass required checks (#19696) [5d7ceaa](https://github.com/answerbook/vector/commit/5d7ceaa8c963bd23e6c0b066fa36c0581103575f) - GitHub
* **ci**: Update labels used by dependabot (#19760) [7cd151a](https://github.com/answerbook/vector/commit/7cd151a822f0073f9df4bf01d7aec11500f5efe1) - GitHub
* **config**: Add configurable support for `http::Uri` (#19758) [d7c615c](https://github.com/answerbook/vector/commit/d7c615c6837429d8e36cd02df8da2e7485656df2) - GitHub
* **config**: Pass the extra context to sources and transforms too (#19779) [a215d59](https://github.com/answerbook/vector/commit/a215d59f1fcef34913e4316c36ca09ebea3bf7a0) - GitHub
* **config**: Revert "Add configurable support for `http::Uri`" (#19770) [29a91a4](https://github.com/answerbook/vector/commit/29a91a44ac762f2b02938d144503849a570ec747) - GitHub
* **config**: Skip serializing default proxy config fields (#19580) [df0eafc](https://github.com/answerbook/vector/commit/df0eafce599b8c58053c0f2d68b479507824fc0b) - GitHub
* **deps**: Bump anyhow from 1.0.76 to 1.0.79 (#19500) [1eda83b](https://github.com/answerbook/vector/commit/1eda83b64c83e067c3577b9e63cc4bb28d064518) - GitHub
* **deps**: Bump assert_cmd from 2.0.12 to 2.0.13 (#19610) [af6169c](https://github.com/answerbook/vector/commit/af6169c99e2bf236b958d775bd8af868c9dac094) - GitHub
* **deps**: Bump async-compression from 0.4.5 to 0.4.6 (#19652) [52c12c3](https://github.com/answerbook/vector/commit/52c12c3fa0355dd53edfd01ffd979f5be40f09f6) - GitHub
* **deps**: Bump async-trait from 0.1.75 to 0.1.77 (#19498) [f38796d](https://github.com/answerbook/vector/commit/f38796d3a8e341a9fc5fe5499a489af33c19a3b7) - GitHub
* **deps**: Bump base64 from 0.21.5 to 0.21.6 (#19557) [c57435d](https://github.com/answerbook/vector/commit/c57435d9b142a34674e4260a9ef6ce7b044c6a4e) - GitHub
* **deps**: Bump base64 from 0.21.6 to 0.21.7 (#19611) [c2f3259](https://github.com/answerbook/vector/commit/c2f32593776f1e9304dc20ae2adbfb3efb8a8eb8) - GitHub
* **deps**: Bump cached from 0.46.1 to 0.47.0 (#19503) [bbff1b2](https://github.com/answerbook/vector/commit/bbff1b2e325df0ce706b244e73126580acd1f846) - GitHub
* **deps**: Bump cached from 0.47.0 to 0.48.0 (#19673) [9c58183](https://github.com/answerbook/vector/commit/9c581836c9a4ba1993022be918a034d50f89794e) - GitHub
* **deps**: Bump cargo_toml from 0.17.2 to 0.18.0 (#19558) [1daa0d3](https://github.com/answerbook/vector/commit/1daa0d38728665d1fd716be848544d2e2cf6579e) - GitHub
* **deps**: Bump cargo_toml from 0.18.0 to 0.19.0 (#19733) [fc09588](https://github.com/answerbook/vector/commit/fc0958863b674fbca4550c274e6f0c7711264593) - GitHub
* **deps**: Bump cargo_toml from 0.19.0 to 0.19.1 (#19744) [bf2d732](https://github.com/answerbook/vector/commit/bf2d7329c0fd41f478f974b282923f00d89cf027) - GitHub
* **deps**: Bump chrono from 0.4.31 to 0.4.33 (#19723) [b141f2e](https://github.com/answerbook/vector/commit/b141f2ea0550410989a98bef80e5863a373dca4c) - GitHub
* **deps**: Bump chrono from 0.4.33 to 0.4.34 (#19851) [6bac428](https://github.com/answerbook/vector/commit/6bac428780de7d79cd750be9cfc36c4060a00019) - GitHub
* **deps**: Bump confy from 0.5.1 to 0.6.0 (#19608) [37125b9](https://github.com/answerbook/vector/commit/37125b9af3c8dfaa6924a8f5e59cc2a37f58923a) - GitHub
* **deps**: Bump crossbeam-utils from 0.8.18 to 0.8.19 (#19560) [2be2976](https://github.com/answerbook/vector/commit/2be297649ba4e16d9b85802f2e0f69c71e2e310f) - GitHub
* **deps**: Bump darling from 0.20.3 to 0.20.4 (#19743) [83be425](https://github.com/answerbook/vector/commit/83be4258bf998e6a2741c0ddf44a5b2ff29cbc67) - GitHub
* **deps**: Bump darling from 0.20.4 to 0.20.5 (#19751) [13ac2df](https://github.com/answerbook/vector/commit/13ac2dfb981160e4f6d1541c8537e47d6ac761e9) - GitHub
* **deps**: Bump env_logger from 0.10.1 to 0.10.2 (#19651) [53f97c1](https://github.com/answerbook/vector/commit/53f97c1c61ca176ba20852d0cfc1e45e44cf2235) - GitHub
* **deps**: Bump getrandom from 0.2.11 to 0.2.12 (#19575) [e3f285c](https://github.com/answerbook/vector/commit/e3f285c32e857b1b1a8de4504e9bdfebdf0e77ec) - GitHub
* **deps**: Bump graphql crates to 7.0.0 (#19579) [dd76ca8](https://github.com/answerbook/vector/commit/dd76ca8815679d1e791b3b16400639fd815168fd) - GitHub
* **deps**: Bump h2 from 0.4.0 to 0.4.1 (#19559) [11f5037](https://github.com/answerbook/vector/commit/11f50370254b85d6ca79d8874b32a55458fa2b7c) - GitHub
* **deps**: Bump heim from `76fa765` to `a66c440` (#19840) [ed5578e](https://github.com/answerbook/vector/commit/ed5578e89c1b0237e826ce0968713d67a99febef) - GitHub
* **deps**: Bump indexmap from 2.2.2 to 2.2.3 (#19855) [51ee104](https://github.com/answerbook/vector/commit/51ee1044a1a60528c52b87e3f1f4cbd0290308fe) - GitHub
* **deps**: Bump indicatif from 0.17.7 to 0.17.8 (#19850) [9e7e658](https://github.com/answerbook/vector/commit/9e7e658fa53c25d7d78d4fff00cdb3bb06f6af19) - GitHub
* **deps**: Bump inventory from 0.3.14 to 0.3.15 (#19732) [7c27b2e](https://github.com/answerbook/vector/commit/7c27b2e5eb82150660a6066318aef2926be84ee1) - GitHub
* **deps**: Bump itertools from 0.12.0 to 0.12.1 (#19745) [9571b4e](https://github.com/answerbook/vector/commit/9571b4ec304f80a530ab312755cb93e9197ae1ba) - GitHub
* **deps**: Bump libc from 0.2.151 to 0.2.152 (#19534) [79f0fd3](https://github.com/answerbook/vector/commit/79f0fd335e6ae92b3d3dab11e04b721536b6f0e8) - GitHub
* **deps**: Bump libc from 0.2.152 to 0.2.153 (#19763) [3da1a02](https://github.com/answerbook/vector/commit/3da1a0206583500abad617147d76b3faf602a09b) - GitHub
* **deps**: Bump lru from 0.12.1 to 0.12.2 (#19731) [51c6466](https://github.com/answerbook/vector/commit/51c6466c7d848b49e9a66293ddfb8211c1f6acb5) - GitHub
* **deps**: Bump maxminddb from 0.23.0 to 0.24.0 (#19574) [8881cc4](https://github.com/answerbook/vector/commit/8881cc4a811d2253699f025f2d20fa496e38fe32) - GitHub
* **deps**: Bump memmap2 from 0.9.3 to 0.9.4 (#19719) [dd50a46](https://github.com/answerbook/vector/commit/dd50a46b92f33dfbf81ef150a7be892c896ab401) - GitHub
* **deps**: Bump mlua from 0.9.2 to 0.9.3 (#19573) [27e49e7](https://github.com/answerbook/vector/commit/27e49e7ee645da5f1bf33b49dc616a3c8592bc72) - GitHub
* **deps**: Bump mlua from 0.9.3 to 0.9.4 (#19607) [6fde186](https://github.com/answerbook/vector/commit/6fde1861fe8961b1c100c951e0752b48673fac12) - GitHub
* **deps**: Bump mlua from 0.9.4 to 0.9.5 (#19717) [0966883](https://github.com/answerbook/vector/commit/09668836bb8331e894d5c48e0376041fb92e385d) - GitHub
* **deps**: Bump mongodb from 2.8.0 to 2.8.1 (#19856) [493fb74](https://github.com/answerbook/vector/commit/493fb74d9530e8dc536e61b0e94ba327f8aac8cb) - GitHub
* **deps**: Bump num_enum from 0.7.1 to 0.7.2 (#19536) [eec7eb5](https://github.com/answerbook/vector/commit/eec7eb5a9abfdc6f63cc1b8f4ed2c8364492622d) - GitHub
* **deps**: Bump num-traits from 0.2.17 to 0.2.18 (#19831) [c4593b7](https://github.com/answerbook/vector/commit/c4593b743078762597c95c9a31430dfc2b845b37) - GitHub
* **deps**: Bump opendal from 0.44.0 to 0.44.1 (#19538) [47fcf91](https://github.com/answerbook/vector/commit/47fcf91f8935df19d93b08a8420c79f67bdcfb68) - GitHub
* **deps**: Bump opendal from 0.44.1 to 0.44.2 (#19676) [f41ca86](https://github.com/answerbook/vector/commit/f41ca86876ae9c6fb98c8edd363691cfff963daf) - GitHub
* **deps**: Bump opendal from 0.44.2 to 0.45.0 (#19788) [7746937](https://github.com/answerbook/vector/commit/774693772f6543166892c8497b3e9ab699045435) - GitHub
* **deps**: Bump openssl from 0.10.62 to 0.10.63 (#19672) [55317dc](https://github.com/answerbook/vector/commit/55317dcda1a26c533242eb3a9bd24a61dd5958e3) - GitHub
* **deps**: Bump openssl-src from 300.2.1+3.2.0 to 300.2.2+3.2.1 (#19750) [ba2b350](https://github.com/answerbook/vector/commit/ba2b3508ef5e6995d3dbd47d70977aa1763e8a34) - GitHub
* **deps**: Bump pin-project from 1.1.3 to 1.1.4 (#19718) [2b83343](https://github.com/answerbook/vector/commit/2b8334397212f749ad5ef4961d22a630568f7dd6) - GitHub
* **deps**: Bump proc-macro2 from 1.0.76 to 1.0.78 (#19671) [2f8fbd1](https://github.com/answerbook/vector/commit/2f8fbd135e1c7a683d70be0c09a8dbc43e6f5d0d) - GitHub
* **deps**: Bump ratatui from 0.25.0 to 0.26.0 (#19787) [6827204](https://github.com/answerbook/vector/commit/68272040067f5cf167d925234bdfc15b6bd60f6f) - GitHub
* **deps**: Bump regex from 1.10.2 to 1.10.3 (#19674) [c12c8e1](https://github.com/answerbook/vector/commit/c12c8e1bef9fb2f9a9a31892d7911b8637f581e7) - GitHub
* **deps**: Bump reqwest from 0.11.23 to 0.11.24 (#19762) [bd9fbd6](https://github.com/answerbook/vector/commit/bd9fbd682b673e01f712a79af326eb307883cfad) - GitHub
* **deps**: Bump rkyv from 0.7.43 to 0.7.44 (#19789) [17b2962](https://github.com/answerbook/vector/commit/17b29628c742a2841a19b19f70c5465935089b68) - GitHub
* **deps**: Bump Rust to 1.75.0 (#19518) [b2cc788](https://github.com/answerbook/vector/commit/b2cc78869c7890ab00e586ab8b34f7ec5828da4a) - GitHub
* **deps**: Bump semver from 1.0.20 to 1.0.21 (#19505) [f5bed3f](https://github.com/answerbook/vector/commit/f5bed3fd72f1239a41a82ac89b6ebb303318f5f9) - GitHub
* **deps**: Bump serde from 1.0.194 to 1.0.195 (#19533) [a3f0337](https://github.com/answerbook/vector/commit/a3f033766dab2d41f00b68f19aa97eecb5f42728) - GitHub
* **deps**: Bump serde from 1.0.195 to 1.0.196 (#19734) [ec9b2c7](https://github.com/answerbook/vector/commit/ec9b2c7df7eba02dc1c3c0252c05a0a6499d5371) - GitHub
* **deps**: Bump serde_bytes from 0.11.12 to 0.11.14 (#19495) [e08b187](https://github.com/answerbook/vector/commit/e08b187b5502b97cbbbd337c043e59227c2de291) - GitHub
* **deps**: Bump serde_json from 1.0.109 to 1.0.111 (#19520) [13a930a](https://github.com/answerbook/vector/commit/13a930afcfbe2f11a8eef9634a2229e3e8672b1f) - GitHub
* **deps**: Bump serde_json from 1.0.111 to 1.0.112 (#19730) [7056f5f](https://github.com/answerbook/vector/commit/7056f5fe02af3d11a0ac813c9043788d96ed233c) - GitHub
* **deps**: Bump serde_with from 3.4.0 to 3.5.0 (#19675) [b51085b](https://github.com/answerbook/vector/commit/b51085b1a8d0c3e7c957bf9ad1d2a8db6a661dce) - GitHub
* **deps**: Bump serde_with from 3.5.0 to 3.6.0 (#19800) [43b96ba](https://github.com/answerbook/vector/commit/43b96baa64a8cd6eefec1679f3b34ad753121d62) - GitHub
* **deps**: Bump serde_with from 3.6.0 to 3.6.1 (#19841) [86fe001](https://github.com/answerbook/vector/commit/86fe001b474cdd7cf74a63bd2f36b2fc81cf9f9f) - GitHub
* **deps**: Bump serde_yaml from 0.9.29 to 0.9.30 (#19514) [a7a4166](https://github.com/answerbook/vector/commit/a7a41661a4339c07034fb38c05ffdea4f5d3c4fc) - GitHub
* **deps**: Bump serde_yaml from 0.9.30 to 0.9.31 (#19832) [0d57ad9](https://github.com/answerbook/vector/commit/0d57ad9548dbfc97f7e6d32d81c6e179e19a465e) - GitHub
* **deps**: Bump serde-toml-merge from 0.3.3 to 0.3.4 (#19771) [9e297f6](https://github.com/answerbook/vector/commit/9e297f6c4faa503d195f29648aa5e35c7343acdd) - GitHub
* **deps**: Bump smallvec from 1.11.2 to 1.12.0 (#19623) [26f2468](https://github.com/answerbook/vector/commit/26f2468f66bc22a0d66b3a382be17a46bc4bb1a9) - GitHub
* **deps**: Bump smallvec from 1.12.0 to 1.13.1 (#19677) [ba9b4bd](https://github.com/answerbook/vector/commit/ba9b4bd7c4af1eed4cc6b7e64686a2e666a306d6) - GitHub
* **deps**: Bump syn from 2.0.46 to 2.0.48 (#19532) [1d979cc](https://github.com/answerbook/vector/commit/1d979cc6791f32b024459f5e76c503bf6947db76) - GitHub
* **deps**: Bump tempfile from 3.9.0 to 3.10.0 (#19807) [b3e0af7](https://github.com/answerbook/vector/commit/b3e0af7f268c2ef4c26299195a0aec0263df0b61) - GitHub
* **deps**: Bump the aws group with 1 update (#19586) [2448a72](https://github.com/answerbook/vector/commit/2448a72770444e4c203d7d937e1ccede22c23aed) - GitHub
* **deps**: Bump the aws group with 1 update (#19605) [1e1f2ec](https://github.com/answerbook/vector/commit/1e1f2ecdf96ec104234756efb5a47167a85bc25e) - GitHub
* **deps**: Bump the aws group with 1 update (#19688) [b56f1c3](https://github.com/answerbook/vector/commit/b56f1c3a341df729a217256fa3fefa9772583c96) - GitHub
* **deps**: Bump the aws group with 2 updates (#19556) [8f504b3](https://github.com/answerbook/vector/commit/8f504b35985b9cc1e29f1505b8fd42abd138851e) - GitHub
* **deps**: Bump the aws group with 2 updates (#19619) [521512d](https://github.com/answerbook/vector/commit/521512dcb07d4222630999e301f82ddd5fd16218) - GitHub
* **deps**: Bump the aws group with 2 updates (#19660) [3274827](https://github.com/answerbook/vector/commit/32748273fbbf3a65851f6e4f65ddaae385000cdd) - GitHub
* **deps**: Bump the aws group with 2 updates (#19697) [88c10a9](https://github.com/answerbook/vector/commit/88c10a9e0142a5aca06972ceba2e24983df631b6) - GitHub
* **deps**: Bump the aws group with 2 updates (#19720) [5f233f2](https://github.com/answerbook/vector/commit/5f233f23700fb22a031168078cdcbaee79242775) - GitHub
* **deps**: Bump the aws group with 2 updates (#19742) [f085b72](https://github.com/answerbook/vector/commit/f085b72615c7e98760aef1192b72f697d127e358) - GitHub
* **deps**: Bump the aws group with 4 updates (#19582) [2e756a1](https://github.com/answerbook/vector/commit/2e756a16dc4aaf2faca2a293cc4f99ea3ef59617) - GitHub
* **deps**: Bump the aws group with 5 updates (#19838) [382ab32](https://github.com/answerbook/vector/commit/382ab32476d5204979e2170de90adcd6087edb64) - GitHub
* **deps**: Bump the clap group with 1 update (#19552) [61b2a3f](https://github.com/answerbook/vector/commit/61b2a3f365876b4a23115d38b7817eff450afa58) - GitHub
* **deps**: Bump the clap group with 1 update (#19606) [59699f6](https://github.com/answerbook/vector/commit/59699f6cf7e4f96d2d7b3d633eb8082d85110695) - GitHub
* **deps**: Bump the clap group with 1 update (#19687) [25b1b8c](https://github.com/answerbook/vector/commit/25b1b8c7d891bbc7bbe8addbde0342c820b5424f) - GitHub
* **deps**: Bump the clap group with 1 update (#19786) [541e308](https://github.com/answerbook/vector/commit/541e3086abcb4d95b77c273f6de19d9dc326c156) - GitHub
* **deps**: Bump the clap group with 2 updates (#19626) [b540936](https://github.com/answerbook/vector/commit/b540936fc0ac132d257e168dae78e228c3cce324) - GitHub
* **deps**: Bump the clap group with 2 updates (#19634) [4e877e5](https://github.com/answerbook/vector/commit/4e877e53d112310ddee4d97417550ed0e20316d4) - GitHub
* **deps**: Bump the crossbeam group with 1 update (#19576) [86b16e0](https://github.com/answerbook/vector/commit/86b16e04a2f98701f13e7c814baf5cf837d0a82c) - GitHub
* **deps**: Bump the graphql group with 1 update (#19583) [c30a45f](https://github.com/answerbook/vector/commit/c30a45f362550c1b2989a1ca43f60bb7267ccfa0) - GitHub
* **deps**: Bump the graphql group with 2 updates (#19670) [eeab67d](https://github.com/answerbook/vector/commit/eeab67d7b86166dfeac345144aaa36d72f746253) - GitHub
* **deps**: Bump the prost group with 1 update (#19830) [1c09d09](https://github.com/answerbook/vector/commit/1c09d09cd4b9f86fd5e0a79d97fc6eb4b215cfa2) - GitHub
* **deps**: Bump thiserror from 1.0.51 to 1.0.56 (#19510) [84de179](https://github.com/answerbook/vector/commit/84de179739a45ba02878c1df0aee5cdee3b8082f) - GitHub
* **deps**: Bump thiserror from 1.0.56 to 1.0.57 (#19854) [9a610b0](https://github.com/answerbook/vector/commit/9a610b009f7809458f50b9dd7ecab5aa15347282) - GitHub
* **deps**: Bump tokio from 1.35.1 to 1.36.0 (#19790) [0dce776](https://github.com/answerbook/vector/commit/0dce77620fbc240f6e880c6f49f7ef7f8bb5e3df) - GitHub
* **deps**: Bump toml from 0.8.8 to 0.8.9 (#19761) [65acf06](https://github.com/answerbook/vector/commit/65acf06934c733bf3608387b2264b071cca27f3d) - GitHub
* **deps**: Bump toml from 0.8.9 to 0.8.10 (#19808) [f38ed15](https://github.com/answerbook/vector/commit/f38ed158f939c6acf78cd039349d897f7127f0d1) - GitHub
* **deps**: Bump uuid from 1.6.1 to 1.7.0 (#19661) [846075c](https://github.com/answerbook/vector/commit/846075c4bbe2fb982c7d289a5011ec96d4f9b0cc) - GitHub
* **deps**: Bump vrl from 0.9.1 to 0.10.0 (#19705) [509a858](https://github.com/answerbook/vector/commit/509a858e74d43a431589b21928c405ac461f6551) - GitHub
* **deps**: Bump VRL to 0.11.0 (#19827) [56486ba](https://github.com/answerbook/vector/commit/56486bafe6ce41a7c92a11ccd0e2cf6e8f7ef838) - GitHub
* **deps**: Bump wasm-bindgen from 0.2.89 to 0.2.90 (#19620) [cebe628](https://github.com/answerbook/vector/commit/cebe6284595badef5112807fd1f7e9a5f0e7d3ce) - GitHub
* **deps**: Bump wasm-bindgen from 0.2.90 to 0.2.91 (#19817) [ff246b6](https://github.com/answerbook/vector/commit/ff246b621b8c6d5c052621d4a4e86c6942a20f13) - GitHub
* **deps**: Bump whoami to 1.5.0 (#20018) [e4951cc](https://github.com/answerbook/vector/commit/e4951cc447d8a3b4896c4603a962651350b6ac37) - Jesse Szwedko
* **deps**: Document Vector's MSRV policy (#19646) [cc9203b](https://github.com/answerbook/vector/commit/cc9203b610868d5de8daff7ac1051dce9038dfe8) - GitHub
* **deps**: expose DatadogSearch (#19778) [ac80d1e](https://github.com/answerbook/vector/commit/ac80d1ed07983d203671b7c2c625715fbc06a234) - GitHub
* **deps**: Group together crossbeam updates (#19572) [14ae52e](https://github.com/answerbook/vector/commit/14ae52ed542514368495aa641e873a851c4bb2f4) - GitHub
* **deps**: Update h2 (#19648) [c119951](https://github.com/answerbook/vector/commit/c1199512c73bfd58e76daf1297cf29f7eff6aa5a) - GitHub
* **deps**: Update lockfree-object-pool to 0.1.5 (#20001) [54bcee7](https://github.com/answerbook/vector/commit/54bcee72242d06eacd355451ed62ee1029925a81) - Jesse Szwedko
* **deps**: Update mio (#20005) [a8cd2a2](https://github.com/answerbook/vector/commit/a8cd2a2df1df26de9e14d51cb84bc0bdd443a195) - Jesse Szwedko
* **dev**: Ensure changelog fragment author doesn't start with @ (#19836) [c172d50](https://github.com/answerbook/vector/commit/c172d504ea26f06a5be15c71dbfa6b135d732dc1) - GitHub
* **docs**: Add banner alerting people of package migration (#19714) [13b9147](https://github.com/answerbook/vector/commit/13b914781b3c118cbf867609ebab5d5fc6a525dc) - GitHub
* **docs**: Add pure/impure badge for VRL functions (#19571) [d6bd269](https://github.com/answerbook/vector/commit/d6bd2696d138e3499deea7db9a9ac9432a96e687) - GitHub
* **docs**: Fix link to RFC 3339 (#19509) [3525d06](https://github.com/answerbook/vector/commit/3525d062dd2387bdda8babc5a98f5a9997a0362a) - GitHub
* **docs**: Fix proofreading mistake in v0.35.0 upgrade guide (#19551) [d282d26](https://github.com/answerbook/vector/commit/d282d260ae1f950f25516498f80ee55512192866) - GitHub
* **docs**: improve source data_dir docs (#19596) [f262324](https://github.com/answerbook/vector/commit/f262324595883633a21ead16c5fc165a576c9f17) - GitHub
* **docs**: suggest make generate-component-docs (#19740) [60f5fe0](https://github.com/answerbook/vector/commit/60f5fe091dfb73139945c931a4fab2164d59cc92) - GitHub
* **docs**: update basic sink tutorial doc (#19722) [650a738](https://github.com/answerbook/vector/commit/650a738e63f3ff7d80ff872760fc8497b257e709) - GitHub
* **docs**: update GELF codec (#19602) [4c09841](https://github.com/answerbook/vector/commit/4c098417baef4c0d2d7af09beaad3dfa1483ad3f) - GitHub
* expose component test utils (#19826) [0046ee9](https://github.com/answerbook/vector/commit/0046ee9b394274bc184efd2a07e76639cebe12fb) - GitHub
* Fix aws feature error (#19567) [2b25a99](https://github.com/answerbook/vector/commit/2b25a99a7347f40043434d1337a6b960338357c0) - GitHub
* Implement an easier creator for multi-valued `ExtraContext` (#19777) [0a2dc2b](https://github.com/answerbook/vector/commit/0a2dc2bafa6e56218797a0c238118ed58fd94113) - GitHub
* **kubernetes**: Bump manifests to v0.30.0 of the chart (#19554) [a412c3c](https://github.com/answerbook/vector/commit/a412c3c013c2de24e6a1502ed1cfe19f4b511f81) - GitHub
* only export RemoteWriteConfig for remote-write feature (#19569) [05b07ab](https://github.com/answerbook/vector/commit/05b07ab196b3891ca203dd64200fa5b064b7abb1) - GitHub
* **releases website**: 0.36 changelog fixes (#19875) [a10a137](https://github.com/answerbook/vector/commit/a10a137394bda91a97bf6d1731459615af2869ad) - Jesse Szwedko
* **releasing**: Add additional note about new VRL decoder [1baa6b2](https://github.com/answerbook/vector/commit/1baa6b2e00d994339f7acc78149bc8136f10e9b2) - Jesse Szwedko
* **releasing**: Add missing changelog entries (#20041) [1ea58e4](https://github.com/answerbook/vector/commit/1ea58e47cadc4acc9d554a60653e76cbdd034105) - Jesse Szwedko
* **releasing**: Automated changelog generation (#19429) [d115e26](https://github.com/answerbook/vector/commit/d115e269dbbb06fe25977df74b10d5cd0fa04628) - GitHub
* **releasing**: Bump Vector to v0.36.0 (#19550) [e1d570d](https://github.com/answerbook/vector/commit/e1d570d99621f5b9c58423bdc1e5e8cee8ca9c0f) - GitHub
* **releasing**: Fix formatting of v0.36.1 release [b58c864](https://github.com/answerbook/vector/commit/b58c8646565c10aea6c87312352587f46f1c776c) - Jesse Szwedko
* **releasing**: Fix markdown formatting of v0.36.0 description [646c9ba](https://github.com/answerbook/vector/commit/646c9ba4bd7d82aee94d006a460edbff6300d45e) - Jesse Szwedko
* **releasing**: Fix markdown formatting of v0.36.0 release description [8c3b2ce](https://github.com/answerbook/vector/commit/8c3b2ce04ba800706e9baab2609e5ca481d62f06) - Jesse Szwedko
* **releasing**: Prepare v0.35.0 release [b6506e6](https://github.com/answerbook/vector/commit/b6506e650d360e593164419aa5d44cc94d44aa07) - Jesse Szwedko
* **releasing**: Prepare v0.35.1 release [10f5d7d](https://github.com/answerbook/vector/commit/10f5d7d99f614dfaa63d3e646cd5467343ec1a29) - Jesse Szwedko
* **releasing**: Prepare v0.36.0 release [a5e48bb](https://github.com/answerbook/vector/commit/a5e48bb5728896713ab5280ef52a7512a7892baf) - Jesse Szwedko
* **releasing**: Prepare v0.36.1 release [2857180](https://github.com/answerbook/vector/commit/2857180dbe42b7da5b21259f14283475e169b5fb) - Jesse Szwedko
* Remove used changelog entries [f243f1c](https://github.com/answerbook/vector/commit/f243f1c281c8121552386da1bff7cf23a323c809) - Jesse Szwedko
* **sample transform**: improve example for `rate` setting (#19834) [c797bc6](https://github.com/answerbook/vector/commit/c797bc69b51574778b804f9bbdeb449af4f9af19) - GitHub
* **sample transform**: make containing module pub (#19816) [c4fe134](https://github.com/answerbook/vector/commit/c4fe1342ce8b80ef822203f01ef0093751195a3d) - GitHub
* **sample transform**: re-organize to expose sampling logic (#19806) [fd76dbf](https://github.com/answerbook/vector/commit/fd76dbf0fff80f89e1b7bdbfb57cf864709e9dfa) - GitHub
* Shorten name of `skip_serializing_if_default` (#19591) [9dd9907](https://github.com/answerbook/vector/commit/9dd9907b356996d9bbb395fd4aea2a207c930914) - GitHub
* **tests**: Add end-to-end tests with the Datadog Agent (#18538) [43a9a36](https://github.com/answerbook/vector/commit/43a9a366c4dee15f0294a0cd22c2dc5b8b2daae8) - GitHub
* **tests**: Add support for proptest to lookup types (#19769) [cf1aec6](https://github.com/answerbook/vector/commit/cf1aec66cd5fd9c4d01efce646de167a079b195e) - GitHub
* Update AWS crates (#19312) [c2cc94a](https://github.com/answerbook/vector/commit/c2cc94a262ecf39798009d29751d59cc97baa0c5) - GitHub
* **vdev**: Drop dependency on `cached` crate (#19693) [650d478](https://github.com/answerbook/vector/commit/650d478fc28f79d1f075f43971cd2b54ca848652) - GitHub
* **vrl stdlib**: Fix redact doc URL templating [0894ee8](https://github.com/answerbook/vector/commit/0894ee858183c474f5fd9be38a6fc42c410985b3) - Jesse Szwedko
* **website**: bump openssl version used for links in docs (#19880) [f1f8c1b](https://github.com/answerbook/vector/commit/f1f8c1bc998ef98215dba117335b74e8e5e57b68) - Jesse Szwedko


### Features

* **clickhouse sink**: add format (#19739) [52049f8](https://github.com/answerbook/vector/commit/52049f81459d064abaf92e302414160e1ab39512) - GitHub
* **codecs**: implement VRL decoder (#19825) [9a20a12](https://github.com/answerbook/vector/commit/9a20a12be927d29e929b62e4313193d91b86f543) - GitHub
* **component validation**: add sink validator (#17980) [28a4cb4](https://github.com/answerbook/vector/commit/28a4cb4ca348287fb336f248988dd39ee9a74907) - GitHub
* **dnsmsg_parser**: add support for parsing HTTPS and SVCB records (#19819) [a5d9a27](https://github.com/answerbook/vector/commit/a5d9a2777f97d23ea880a2a9f819878d6c69cfa5) - GitHub
* **greptimedb sink**: update ingestion api for greptimedb sink (#19410) [586fb31](https://github.com/answerbook/vector/commit/586fb31a1678ca220cdeef7f37b091de41b6ce95) - GitHub
* **new source**: Add Prometheus Pushgateway source (#18143) [3b120ff](https://github.com/answerbook/vector/commit/3b120ff0c17ccedf07f423090f8c009bf7164410) - GitHub
* pub prometheus sink configs (#19540) [38d8801](https://github.com/answerbook/vector/commit/38d8801d4096f1f9e12ffd01fe8014b92682297d) - GitHub
* **s3 source**: Add `delete_failed_message` configuration option (#19748) [0f3faba](https://github.com/answerbook/vector/commit/0f3faba5ee3fae2531ce4bb9b739a1a54d860f69) - GitHub


### Miscellaneous

* Merge pull request #484 from answerbook/feature/LOG-19571 [9d969ad](https://github.com/answerbook/vector/commit/9d969adcbb0d38cef3e092a655449e2da580fec1) - GitHub [LOG-19571](https://logdna.atlassian.net/browse/LOG-19571)
* Merge remote-tracking branch 'origin/master' into upstream-v0.36 [0706333](https://github.com/answerbook/vector/commit/070633376a78fdc95ea45799abc701cf2f8869ab) - Chris Nixon
* Merge remote-tracking branch 'origin/master' into feature/LOG-19571 [d377b89](https://github.com/answerbook/vector/commit/d377b8994747436a149e5286d5153f428d195012) - Chris Nixon [LOG-19571](https://logdna.atlassian.net/browse/LOG-19571)
* Merge upstream/v0.36 into merge-vector-0.36 [d5c6ec8](https://github.com/answerbook/vector/commit/d5c6ec8f924bdcc2b0c025c3c754c7c605383b65) - Chris Nixon
* fix (aws service): use http client so we can use openssl tls. (#19939) [a467715](https://github.com/answerbook/vector/commit/a46771550d749eb40835ffe7f302eae0fd246e86) - Jesse Szwedko
* Revert "fix(config): acquire exclusive lock to global data dir" (#19701) [ee9d182](https://github.com/answerbook/vector/commit/ee9d18254398fbf027f88a8f62592e335cc40223) - GitHub
* fix(data_dir lock) improve error message when failing to create data_dir lock (#19694) [e886cdd](https://github.com/answerbook/vector/commit/e886cdd3c1014dcf26e47dfc7b99fdaa9fe0041a) - GitHub
* **administration config**: New --skip-healthchecks option for vector validate (#19691) [1825220](https://github.com/answerbook/vector/commit/18252206790c0c97863d110d0ec2cdd3bb15d24d) - GitHub
* **aws_sns sink**: Add documentation data (#19715) [cacb44f](https://github.com/answerbook/vector/commit/cacb44fe5b9b59a3b528761c6b53c7437411f036) - GitHub
* **codec**: Allow @ as valid GELF field character in decoder (#19544) [b1502ec](https://github.com/answerbook/vector/commit/b1502ec185a517f2c95078f5a70acae7baaf1c30) - GitHub
* **config api**: Add `graphql` field to toggle graphql endpoint (#19645) [5bb4926](https://github.com/answerbook/vector/commit/5bb492608d935c38a1ae6e748592f0ae9812413c) - GitHub
* **config**: Fix handling of the default value for `ProxyConfig::enabled` (#19604) [1705dfd](https://github.com/answerbook/vector/commit/1705dfd5d85b08be96a594dfbf9081ed78497ee1) - GitHub
* **datadog_metrics sink, datadog_logs sink**: improve retry behavior code quality (#19450) [aa6fd40](https://github.com/answerbook/vector/commit/aa6fd40ae9fda3279cbfd4f4ec3bdbb7debde691) - GitHub
* Fix docs for `ignore_older_secs` (#19682) [a6fb31b](https://github.com/answerbook/vector/commit/a6fb31b2bfd3fedcf53d858d5d7f99942649ea21) - GitHub
* **releasing**: Bump Alpine base image from 3.18 to 3.19 (#19804) [c5ee82f](https://github.com/answerbook/vector/commit/c5ee82faf01b543ad4db746abe5d4a305844a406) - GitHub
* **remap transform**: Fix `drop_on_abort` docs (#19918) [3057ccf](https://github.com/answerbook/vector/commit/3057ccfd7e0f58b615d756ca6541b5604053cef4) - Jesse Szwedko
* **setup**: fix inconsistency in docker configuration example (#19797) [de24167](https://github.com/answerbook/vector/commit/de24167165a026c4df387459058efe341631668e) - GitHub
* **sinks**: improve documentation of `RetryLogic` trait functions (#19617) [131ab45](https://github.com/answerbook/vector/commit/131ab453d4611699e6f6989546c4b5d289e8768a) - GitHub
* **splunk_hec source**: Clarify that this source receives data from Splunk clients (#19615) [045b384](https://github.com/answerbook/vector/commit/045b38448482a4d090b3ac0fbafa10fbf2ba0030) - GitHub
* **unit tests**: Enable population of event metadata by a VRL unit test source (#19729) [c1a39e4](https://github.com/answerbook/vector/commit/c1a39e4067362d6e699573c4be4a92cef044766f) - GitHub
* **vrl**: add documentation for `parse_etld` function (#19795) [4115c65](https://github.com/answerbook/vector/commit/4115c65587918e0f8a8ab31b1444e5c79e12e5ec) - GitHub
* **vrl**: add documentation for punycode encoding functions (#19794) [0851fca](https://github.com/answerbook/vector/commit/0851fca24799b9cd61df4eb7c7ab1838ae668236) - GitHub
* **vrl**: Add documentation for replace_with (#19638) [beb76a8](https://github.com/answerbook/vector/commit/beb76a81e8761da4eb2e0873607ba327baa81ea9) - GitHub
* **vrl**: Add VRL function get_vector_timezone (#19727) [ab9bf4e](https://github.com/answerbook/vector/commit/ab9bf4ed2aa9e00223c973e5c899b1ef8aedade0) - GitHub
* **vrl**: Documentation for redact redactor option (#19749) [c2917c1](https://github.com/answerbook/vector/commit/c2917c1e22a9642d0e0072654c40be0c385c6b9b) - GitHub
* **vrl**: fix example for high quality error messages (#19821) [9d4e89e](https://github.com/answerbook/vector/commit/9d4e89ee6304918be9a91e32a2edf89189bfe4c4) - GitHub
* **vrl**: make `parse_etld` fallible in docs (#19842) [405f3ef](https://github.com/answerbook/vector/commit/405f3ef22c3e25e196a4d9f76a8dfbb17f2e8c5c) - GitHub

## [4.2.2](https://github.com/answerbook/vector/compare/v4.2.1...v4.2.2) (2024-08-06)


### Chores

* revert track active profilers with each revision [65178a1](https://github.com/answerbook/vector/commit/65178a1dc190d11ae0453dedb5e3e57b4851aeb8) - Dan Hable [LOG-9999999](https://logdna.atlassian.net/browse/LOG-9999999)

## [4.2.1](https://github.com/answerbook/vector/compare/v4.2.0...v4.2.1) (2024-08-06)


### Bug Fixes

* **kubernetes**: pods rolling in order [b2c6c7d](https://github.com/answerbook/vector/commit/b2c6c7d8f75c7b3073d0b5e8fe3f9499a768d873) - Martin Hansen [INFRA-7234](https://logdna.atlassian.net/browse/INFRA-7234)


### Miscellaneous

* Merge pull request #493 from answerbook/martin/roll-in-parallel [964f3e4](https://github.com/answerbook/vector/commit/964f3e44bc5a60b7ea890de08c4fd8af943d5cf3) - GitHub

# [4.2.0](https://github.com/answerbook/vector/compare/v4.1.0...v4.2.0) (2024-08-02)


### Features

* track active profilers with each revision [7ee6c12](https://github.com/answerbook/vector/commit/7ee6c12a1fb83ed80ee22ba2ddfabd6e91841ada) - Jacob Hull [LOG-20395](https://logdna.atlassian.net/browse/LOG-20395)


### Miscellaneous

* Merge pull request #492 from answerbook/jakedipity/LOG-20395 [6c2bbb1](https://github.com/answerbook/vector/commit/6c2bbb163d4da979e9a150073e7f57e9d867c62c) - GitHub [LOG-20395](https://logdna.atlassian.net/browse/LOG-20395)

# [4.1.0](https://github.com/answerbook/vector/compare/v4.0.6...v4.1.0) (2024-07-30)


### Features

* **vrl**: Bump VRL to version 11 (#483) [4ac6537](https://github.com/answerbook/vector/commit/4ac6537104088b4a0a557fec25a20406a3087faf) - GitHub [LOG-20270](https://logdna.atlassian.net/browse/LOG-20270)

## [4.0.6](https://github.com/answerbook/vector/compare/v4.0.5...v4.0.6) (2024-07-24)


### Bug Fixes

* **aggregate-v2**: Consume finalizers during state persistence [c94122d](https://github.com/answerbook/vector/commit/c94122d387e187cfb65028b7ca672610b8ff4edd) - Dan Hable [LOG-19818](https://logdna.atlassian.net/browse/LOG-19818)
* **aggregate-v2**: skip flush if state persistence is enabled [d959a20](https://github.com/answerbook/vector/commit/d959a20bb51c7dce86fe75a4e518d8eb933c9de3) - Dan Hable [LOG-19913](https://logdna.atlassian.net/browse/LOG-19913)

## [4.0.5](https://github.com/answerbook/vector/compare/v4.0.4...v4.0.5) (2024-07-23)


### Bug Fixes

* **validate**: do not colorize output [7efd22f](https://github.com/answerbook/vector/commit/7efd22ff104bb3fe4a108a47ef6ed2633469d638) - Mike Del Tito [LOG-19759](https://logdna.atlassian.net/browse/LOG-19759)


### Miscellaneous

* Merge pull request #489 from answerbook/mdeltito/LOG-19759 [4de760b](https://github.com/answerbook/vector/commit/4de760bc3abce5124c1b0d7ceecfdbefdc6942ca) - GitHub [LOG-19759](https://logdna.atlassian.net/browse/LOG-19759)

## [4.0.4](https://github.com/answerbook/vector/compare/v4.0.3...v4.0.4) (2024-07-22)


### Chores

* set vector ver to release ver [806d409](https://github.com/answerbook/vector/commit/806d4096f14a78a6840e2ae73032ccee146a9eaf) - Matt March [LOG-19935](https://logdna.atlassian.net/browse/LOG-19935)

## [4.0.3](https://github.com/answerbook/vector/compare/v4.0.2...v4.0.3) (2024-07-16)


### Bug Fixes

* **performance**: set metadata only for otlp metrics [8678c3e](https://github.com/answerbook/vector/commit/8678c3ec1718ce735147a3c22b2fd6358f137833) - Sergey Opria [LOG-20272](https://logdna.atlassian.net/browse/LOG-20272)


### Miscellaneous

* Merge pull request #487 from answerbook/sopria/LOG-20272 [2e336ea](https://github.com/answerbook/vector/commit/2e336eab65bb4dc64aef4f9d3c239ee7ed6c3dc5) - GitHub [LOG-20272](https://logdna.atlassian.net/browse/LOG-20272)

## [4.0.2](https://github.com/answerbook/vector/compare/v4.0.1...v4.0.2) (2024-07-12)


### Bug Fixes

* **performance**: fixes to improve performance [cf71f6a](https://github.com/answerbook/vector/commit/cf71f6aafe64ceb98f3576c119d0ab83be4c4623) - Sergey Opria [LOG-20272](https://logdna.atlassian.net/browse/LOG-20272)


### Miscellaneous

* Merge pull request #486 from answerbook/sopria/LOG-20272 [bbb6c48](https://github.com/answerbook/vector/commit/bbb6c485cf846210b11a8c65c1236325b24e6123) - GitHub [LOG-20272](https://logdna.atlassian.net/browse/LOG-20272)

## [4.0.1](https://github.com/answerbook/vector/compare/v4.0.0...v4.0.1) (2024-07-12)


### Chores

* **kafka**: tweak debug logging in kafka sources [284b636](https://github.com/answerbook/vector/commit/284b636d1de366feda7abe10bf163855d01ddde7) - Dan Hable [LOG-20279](https://logdna.atlassian.net/browse/LOG-20279)

# [4.0.0](https://github.com/answerbook/vector/compare/v3.23.0...v4.0.0) (2024-07-10)


### Bug Fixes

* **ARC, networking**: improve request settings (#19101) [7dc7d2e](https://github.com/answerbook/vector/commit/7dc7d2e351ff4413248233c9ded3888646800f59) - GitHub
* **aws_kinesis_firehose sink**: add workaround for batching (#19108) [6d49346](https://github.com/answerbook/vector/commit/6d49346be0ceb72845eacadcc536a5ddfef5d9c6) - GitHub
* **ci**: Change regression test report path [79e4bdf](https://github.com/answerbook/vector/commit/79e4bdfe76037a937e123b001b564d65a338d5b1) - Jesse Szwedko
* **ci**: peg pulsar docker image for int tests to stable image (#19287) [c2c2dbd](https://github.com/answerbook/vector/commit/c2c2dbd7f86112ce8a0aa6a92ee92ee7e3a023e4) - GitHub
* **ci**: Use correct concurrency group settings for comment trigger & PR commit workflows (#19283) [2867458](https://github.com/answerbook/vector/commit/286745878e261ff6c61e191e2b3a2a607ae3fa7a) - GitHub
* **codecs**: cargo test compilation error (#19268) [9195f75](https://github.com/answerbook/vector/commit/9195f7555abd3f11bdd244576958092ba9a83d79) - GitHub
* **codecs**: fix 'ProtobufSerializerConfig' input type (#19264) [e5c2b69](https://github.com/answerbook/vector/commit/e5c2b695674478fed00199e079401b1b39e2f1c0) - GitHub
* component validation runner and 'sinks::datadog::test_utils' feature gates (#19334) [32e99eb](https://github.com/answerbook/vector/commit/32e99eb69aa491855dc001d8595fc391ff5d7002) - GitHub
* **config**: rustc warnings (#19075) [b68362c](https://github.com/answerbook/vector/commit/b68362c9462a2c032a8319173f6e579b75757961) - GitHub
* **datadog_agent source**: return 200 on empty object payload (#19093) [e487cdf](https://github.com/answerbook/vector/commit/e487cdfa4abd7b0b18dfb518282f35d092297a54) - GitHub
* **datadog_metrics sink**: evaluate series v1 env var at runtime (#19148) [3645a71](https://github.com/answerbook/vector/commit/3645a7131b5c442cdf554986765242a4afb11c85) - GitHub
* **datadog_metrics sink**: Revert to using v1 endpoint by default (#19138) [c023196](https://github.com/answerbook/vector/commit/c023196d70d4e5d6b47c7343eb9502a7077d8e1d) - GitHub
* **dev**: don't compile `secret-backend-example` by default (#19317) [9321949](https://github.com/answerbook/vector/commit/93219494a2d816e0e2c8469cb6ffdbfb8c92bd4f) - GitHub
* **dev**: Fix building the benches (#19235) [32245e0](https://github.com/answerbook/vector/commit/32245e0e760e70f5a1746c93fac25149aba13cfc) - GitHub
* **docs, http_server source**: Update docs for disabling `max_connection_age_secs` (#19802) [8bfb1e5](https://github.com/answerbook/vector/commit/8bfb1e51b5918ee18c398113c0a5c647dc8e08f8) - Jesse Szwedko
* **file source, kubernetes_logs source, file sink**: make `file` internal metric tag opt-out (#19084) [10b3ae7](https://github.com/answerbook/vector/commit/10b3ae7c0491d59b024b8ddfdbc3013d3ce663ba) - GitHub
* **file source, kubernetes_logs source, file sink**: make file internal metric tag opt-in (#19145) [981fb8c](https://github.com/answerbook/vector/commit/981fb8c08f0cd74440bac6710c893d99b0c698c9) - GitHub
* **file source**: emit the correct start offset for multiline aggregated lines (#19065) [c663c35](https://github.com/answerbook/vector/commit/c663c354fef5878bf35a3a9afb7bd64053437848) - GitHub
* **http_server source**: Conditionally send Connection: Close header based on HTTP version (#19801) [3bb7aea](https://github.com/answerbook/vector/commit/3bb7aea05647dd3659898497c479718f2e6055fd) - Jesse Szwedko
* **kafka source, kafka sink, auth**: fixed kafka tls config (#19117) [d4189e0](https://github.com/answerbook/vector/commit/d4189e06bd7f8efa1b0daf68f5e1a57eb5b489af) - GitHub
* **kafka source, kafka sink**: propagate span for internal events (#19082) [515ce43](https://github.com/answerbook/vector/commit/515ce43d6562f896a4ed3e710ea77e95a5f2ea91) - GitHub
* **loki sink**: update to use the global list of compression algorithms (#19099) [5a53173](https://github.com/answerbook/vector/commit/5a53173ef6de6c2d19a02372748a951c7cb90628) - GitHub
* **playground**: fix playground vrl version and link (#19119) [8dcb7db](https://github.com/answerbook/vector/commit/8dcb7db9c03610802b59a90e973c86f19c985099) - GitHub
* **releases website**: close code block (#19389) [d9de797](https://github.com/answerbook/vector/commit/d9de79728e266c6cd1c3750aab82b57639e540bf) - GitHub
* **remap**: do not emit error/discarded metrics for re-routed events (#19296) [d703e92](https://github.com/answerbook/vector/commit/d703e92a7b415368ce4e351f0c4850cbe5fb3d31) - GitHub
* **remap**: filter out file contents from error logs (#19356) [2ad7097](https://github.com/answerbook/vector/commit/2ad7097b10112f1bd086d6a58c3bce47eb5652ae) - GitHub
* **sinks**: set fixed buffer size for distributed service (#18699) [dcd9942](https://github.com/answerbook/vector/commit/dcd994201a6e91a8af3daf2d5e129c64c19dd062) - GitHub
* **sinks**: use uncompressed body size for bytes sent (#19060) [ca64f31](https://github.com/answerbook/vector/commit/ca64f310c18d9636edc78644df2797296fdf4b0a) - GitHub
* **sources**: always emit HttpBytesReceived after decompression (#19048) [40305f1](https://github.com/answerbook/vector/commit/40305f164fb4e8f6976b63d6723c50d67d6b1af6) - GitHub
* **tag_cardinality_limit transform**: mitigate performance bottleneck [42683ef](https://github.com/answerbook/vector/commit/42683effa80d2d8e30ac2dfda2f5349063c58730) - Jesse Szwedko
* **throttle transform**: make `events_discarded_total` internal metric with `key` tag opt-in (#19083) [678605c](https://github.com/answerbook/vector/commit/678605ce32005f37e908803606986251f8c7adb7) - GitHub
* unused enum variants (#19321) [4503ed6](https://github.com/answerbook/vector/commit/4503ed62fdb99d079f072dd685c8110faf058ac9) - GitHub
* **website**: Add ignored branch check to workflow (#19427) [c35831b](https://github.com/answerbook/vector/commit/c35831b2bb34460ff40f79beeb4827436c86d926) - GitHub


### Chores

* add readme and refactor protobuf test fixtures (#19277) [8abe8e5](https://github.com/answerbook/vector/commit/8abe8e5a4bb27f4191a67ce5c3869ca75e09f2e1) - GitHub
* **aws_kinesis_streams sink**: re-enable int test for partition_key (#19220) [4aaf4c2](https://github.com/answerbook/vector/commit/4aaf4c28995d24e9c3a5179a5c6c4f3f11332b34) - GitHub
* **ci**: Bump actions/github-script from 6.4.1 to 7.0.1 (#19200) [5a5ea19](https://github.com/answerbook/vector/commit/5a5ea193d6e17fefee95571dc5732c5aa20d6f8d) - GitHub
* **ci**: Bump actions/labeler from 4 to 5 (#19300) [d16a0a5](https://github.com/answerbook/vector/commit/d16a0a584e0ae903830a45fb766d608d3bde154b) - GitHub
* **ci**: Bump actions/labeler from 4 to 5 (#19358) [5e0dc25](https://github.com/answerbook/vector/commit/5e0dc25a291278bca4e8ca7ee7f93830b95f380a) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.27.2 to 1.28.0 (#19137) [6b53a06](https://github.com/answerbook/vector/commit/6b53a067b570fd18736f1c96fc8bafc58d7afa2c) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.28.0 to 1.28.1 (#19157) [9810071](https://github.com/answerbook/vector/commit/98100712673b313c5bd435822a977f43dda0bde0) - GitHub
* **ci**: Bump cloudsmith-io/action from 0.5.3 to 0.5.4 (#19254) [6262eb9](https://github.com/answerbook/vector/commit/6262eb9f2af823b726a70c90963770452598510b) - GitHub
* **ci**: Bump docker/build-push-action from 5.0.0 to 5.1.0 (#19185) [d391c43](https://github.com/answerbook/vector/commit/d391c43366de80224a07b04a7482e6e70c7ae233) - GitHub
* **ci**: Bump docker/metadata-action from 5.0.0 to 5.2.0 (#19282) [ab54db1](https://github.com/answerbook/vector/commit/ab54db131e330e01268426d611c3df361e4b8224) - GitHub
* **ci**: Bump docker/metadata-action from 5.2.0 to 5.3.0 (#19299) [0ac650b](https://github.com/answerbook/vector/commit/0ac650b6a3917c1648768674ce39219cd672f26a) - GitHub
* **ci**: Bump docker/metadata-action from 5.3.0 to 5.4.0 (#19414) [7448d3f](https://github.com/answerbook/vector/commit/7448d3febff368fe893ff13bb2c729ad0b526dd4) - GitHub
* **ci**: Bump the artifact group with 2 updates (#19391) [17fd152](https://github.com/answerbook/vector/commit/17fd152743d914a4d2bcad8cc5194c8c1eea5e7e) - GitHub
* **ci**: Configure spellchecker to ignore avro files (#19432) [5eab952](https://github.com/answerbook/vector/commit/5eab9528d68f68108feac5385aa4e24cdf3f1560) - GitHub
* **ci**: Fix test of install script (#19425) [0f40989](https://github.com/answerbook/vector/commit/0f4098970cc8eb06847527695eb0ea50f0b2b4f1) - GitHub
* **ci**: Remove Cloudsmith package publishing (#19378) [290a635](https://github.com/answerbook/vector/commit/290a635e2f50e83c5b9dad5df5d8f3187878b4d9) - GitHub
* **ci**: Revert bump the artifact group with 2 updates (#19416) [09978c9](https://github.com/answerbook/vector/commit/09978c93aa3d8c759f35154f58a9b899c1a46875) - GitHub
* **ci**: revert peg pulsar docker image for int tests to stable image (#19297) [17b672a](https://github.com/answerbook/vector/commit/17b672a5b7ac403e847c40216c5d0ef2b9729646) - GitHub
* **config**: Add option to turn missing env vars in config into an error (#19393) [f466177](https://github.com/answerbook/vector/commit/f4661777b33ca61fa6935afa798cbd2d88a62daf) - GitHub
* **deps**: Bump anyhow from 1.0.75 to 1.0.76 (#19437) [e350c6b](https://github.com/answerbook/vector/commit/e350c6b318e238c6ab5f26071322104f85b94be2) - GitHub
* **deps**: Bump async-compression from 0.4.4 to 0.4.5 (#19153) [e750622](https://github.com/answerbook/vector/commit/e75062257ffd277c5dedc65b652f5dda6860246d) - GitHub
* **deps**: Bump async-graphql from 6.0.10 to 6.0.11 (#19196) [2d1523f](https://github.com/answerbook/vector/commit/2d1523fb3610feb910172648829bc5a8564e0ea7) - GitHub
* **deps**: Bump async-graphql from 6.0.9 to 6.0.10 (#19053) [1baadd6](https://github.com/answerbook/vector/commit/1baadd6a6f77e58cac0d31df86aa328a4cb206c7) - GitHub
* **deps**: Bump async-graphql-warp from 6.0.10 to 6.0.11 (#19205) [1302132](https://github.com/answerbook/vector/commit/13021327b149b3f41bbdcab3bb2595fc05a57e41) - GitHub
* **deps**: Bump async-graphql-warp from 6.0.9 to 6.0.10 (#19088) [3902b23](https://github.com/answerbook/vector/commit/3902b2365c0b1660307e3032a5cc4538ca78bd52) - GitHub
* **deps**: Bump async-nats from 0.32.1 to 0.33.0 (#19091) [7066978](https://github.com/answerbook/vector/commit/70669785a817070966efc6e71a20ed222326c2d7) - GitHub
* **deps**: Bump async-trait from 0.1.74 to 0.1.75 (#19440) [b22dc25](https://github.com/answerbook/vector/commit/b22dc2557a1c88548e8ce0562fbde3bf202b1f8b) - GitHub
* **deps**: Bump bstr from 1.7.0 to 1.8.0 (#19111) [c4ed54d](https://github.com/answerbook/vector/commit/c4ed54d216c5cefbbf60ccefe4a14f13c44abac4) - GitHub
* **deps**: Bump bstr from 1.8.0 to 1.9.0 (#19477) [84cf99a](https://github.com/answerbook/vector/commit/84cf99a171632036581992d63012a99c5c33869f) - GitHub
* **deps**: Bump cargo_toml from 0.17.0 to 0.17.1 (#19225) [d2d2ad0](https://github.com/answerbook/vector/commit/d2d2ad0ad51d79bc9b193f2aef16efc53fd703cc) - GitHub
* **deps**: Bump cargo_toml from 0.17.1 to 0.17.2 (#19422) [a2b92f4](https://github.com/answerbook/vector/commit/a2b92f4a0c171ea677f3f2d55730e7e5be5f045b) - GitHub
* **deps**: Bump cargo-deb to 2.0.2 (#19288) [7a39ae9](https://github.com/answerbook/vector/commit/7a39ae92ff39b26a3ce51f335a8f45d5bc20d74b) - GitHub
* **deps**: Bump chrono-tz from 0.8.4 to 0.8.5 (#19479) [f6b8cb2](https://github.com/answerbook/vector/commit/f6b8cb24fc24b013a76bb83e67c8dd3cbcd70c35) - GitHub
* **deps**: Bump cidr-utils from 0.5.11 to 0.6.1 (#19276) [ccedb8c](https://github.com/answerbook/vector/commit/ccedb8c09177e64ebbe0b7a557acd951700a883e) - GitHub
* **deps**: Bump colored from 2.0.4 to 2.1.0 (#19350) [2caf6e2](https://github.com/answerbook/vector/commit/2caf6e215e679eda807340a96c6723ac7ec05a85) - GitHub
* **deps**: Bump crossbeam-utils from 0.8.16 to 0.8.17 (#19381) [2277b58](https://github.com/answerbook/vector/commit/2277b58bc89950f03b89882c64985e1c5dd988e6) - GitHub
* **deps**: Bump crossbeam-utils from 0.8.17 to 0.8.18 (#19465) [92fc726](https://github.com/answerbook/vector/commit/92fc72646bb9606ac9a456df78f4032732582852) - GitHub
* **deps**: Bump data-encoding from 2.4.0 to 2.5.0 (#19216) [5e2e4a0](https://github.com/answerbook/vector/commit/5e2e4a07856a74f508b37a923ce29925d198fbbd) - GitHub
* **deps**: Bump env_logger from 0.10.0 to 0.10.1 (#19130) [913c2ae](https://github.com/answerbook/vector/commit/913c2ae9bc9a722516ced689d8b5a61963a1b221) - GitHub
* **deps**: Bump getrandom from 0.2.10 to 0.2.11 (#19089) [e9ec783](https://github.com/answerbook/vector/commit/e9ec7835a55678631cf4ba73d45aae5bf766edff) - GitHub
* **deps**: Bump h2 from 0.3.21 to 0.4.0 (#19168) [d5f6d07](https://github.com/answerbook/vector/commit/d5f6d074fb550b9bbb8b4eae3a44c64f1353065b) - GitHub
* **deps**: Bump hashbrown from 0.14.2 to 0.14.3 (#19239) [e1d97de](https://github.com/answerbook/vector/commit/e1d97de41056ba899fe0060164e8f82a1ab88fe4) - GitHub
* **deps**: Bump hdrhistogram from 7.5.2 to 7.5.3 (#19129) [bd7df4a](https://github.com/answerbook/vector/commit/bd7df4a7b8ac25a874ec3d2cef394eaaf375628d) - GitHub
* **deps**: Bump hdrhistogram from 7.5.3 to 7.5.4 (#19194) [50eb79c](https://github.com/answerbook/vector/commit/50eb79c5e5338f34a40a1e8891f1d3b81981ace7) - GitHub
* **deps**: Bump hyper from 0.14.27 to 0.14.28 (#19419) [27ccf52](https://github.com/answerbook/vector/commit/27ccf524db221c4adc23846808a6f0942da16a3b) - GitHub
* **deps**: Bump inventory from 0.3.13 to 0.3.14 (#19452) [82c4e50](https://github.com/answerbook/vector/commit/82c4e50a16530f1cc1516cc5d682a892c1d6958f) - GitHub
* **deps**: Bump itertools from 0.11.0 to 0.12.0 (#19152) [c089b8f](https://github.com/answerbook/vector/commit/c089b8f0c44999bc46322bcfcf0c97783702f051) - GitHub
* **deps**: Bump libc from 0.2.150 to 0.2.151 (#19349) [c7b1cab](https://github.com/answerbook/vector/commit/c7b1cabc42b9fdbfca474a14aa2185fd5a4e4403) - GitHub
* **deps**: Bump lru from 0.12.0 to 0.12.1 (#19215) [5a0966a](https://github.com/answerbook/vector/commit/5a0966ad9ca4ad6a69b3a3b86cdfb92edd634c77) - GitHub
* **deps**: Bump memmap2 from 0.9.0 to 0.9.2 (#19404) [f124f86](https://github.com/answerbook/vector/commit/f124f86766031863a6a14be1031d112a7b5c48f7) - GitHub
* **deps**: Bump memmap2 from 0.9.2 to 0.9.3 (#19431) [bf2b65c](https://github.com/answerbook/vector/commit/bf2b65c151be8ef83b240e53a15256224116b1e2) - GitHub
* **deps**: Bump mlua from 0.9.1 to 0.9.2 (#19227) [c8a0d41](https://github.com/answerbook/vector/commit/c8a0d41a765115b1c0270453485627b43bc5d39a) - GitHub
* **deps**: Bump mongodb from 2.7.1 to 2.8.0 (#19365) [0803faa](https://github.com/answerbook/vector/commit/0803faaff978ca593e88c372a406a564affebaae) - GitHub
* **deps**: Bump nkeys from 0.3.2 to 0.4.0 (#19181) [9692ee0](https://github.com/answerbook/vector/commit/9692ee0a5e5b393008d7273abbb485f3aa6052e7) - GitHub
* **deps**: Bump once_cell from 1.18.0 to 1.19.0 (#19339) [a006053](https://github.com/answerbook/vector/commit/a006053c944061c5ebd8cb3be93f4438a03008c6) - GitHub
* **deps**: Bump opendal from 0.41.0 to 0.42.0 (#19169) [dcb40f6](https://github.com/answerbook/vector/commit/dcb40f62d0a8c28244b5fa259bbf130b5fe019de) - GitHub
* **deps**: Bump opendal from 0.42.0 to 0.43.0 (#19338) [77353fc](https://github.com/answerbook/vector/commit/77353fc188db935a335f69e1aae7381fc70f1411) - GitHub
* **deps**: Bump opendal from 0.43.0 to 0.44.0 (#19483) [84e1ac1](https://github.com/answerbook/vector/commit/84e1ac11d3d221710b30030224b7928d9c56c998) - GitHub
* **deps**: Bump openssl from 0.10.59 to 0.10.60 (#19226) [86689c7](https://github.com/answerbook/vector/commit/86689c7f76d15a726bfc0141f5aebad26b2c796d) - GitHub
* **deps**: Bump openssl from 0.10.60 to 0.10.61 (#19306) [63ea8a5](https://github.com/answerbook/vector/commit/63ea8a5bd1ee2f79bbc8ca3fc81c40c49656400d) - GitHub
* **deps**: Bump openssl from 0.10.61 to 0.10.62 (#19462) [2ae64f3](https://github.com/answerbook/vector/commit/2ae64f3b20bd0305383ef7339a1ee07cc8ec62c5) - GitHub
* **deps**: Bump openssl-src from 300.1.6+3.1.4 to 300.2.1+3.2.0 (#19364) [28c70a0](https://github.com/answerbook/vector/commit/28c70a0296aa3ab0e6d78a991d572f7003fe13e3) - GitHub
* **deps**: Bump ordered-float from 4.1.1 to 4.2.0 (#19307) [1bdbf6d](https://github.com/answerbook/vector/commit/1bdbf6ddd5d10d3c23559eb8b89e2ec3af68e768) - GitHub
* **deps**: Bump owo-colors from 3.5.0 to 4.0.0 (#19438) [88f5c23](https://github.com/answerbook/vector/commit/88f5c23614d2197a4eaad1c8bca993250782e566) - GitHub
* **deps**: Bump percent-encoding from 2.3.0 to 2.3.1 (#19228) [54c8c92](https://github.com/answerbook/vector/commit/54c8c92ec2af7f177a77e355299e7983ac202483) - GitHub
* **deps**: Bump proc-macro2 from 1.0.69 to 1.0.70 (#19238) [02c09a4](https://github.com/answerbook/vector/commit/02c09a417e4c165ccc7746b47b27d1234a5429ca) - GitHub
* **deps**: Bump proc-macro2 from 1.0.70 to 1.0.71 (#19453) [06a9248](https://github.com/answerbook/vector/commit/06a924882bb900ac980bd333dfdb938aa316e706) - GitHub
* **deps**: Bump proc-macro2 from 1.0.71 to 1.0.74 (#19496) [dad8a85](https://github.com/answerbook/vector/commit/dad8a850db0d4a845f0e8d3b506524a8a0617311) - GitHub
* **deps**: Bump proptest from 1.3.1 to 1.4.0 (#19131) [7c6c0d1](https://github.com/answerbook/vector/commit/7c6c0d1e6a65f32abdb47fc3b43bcdcaf63f5b34) - GitHub
* **deps**: Bump quanta from 0.12.1 to 0.12.2 (#19486) [074c257](https://github.com/answerbook/vector/commit/074c257aa1767c27416f0d00c0f45c746a4dcc35) - GitHub
* **deps**: Bump ratatui from 0.24.0 to 0.25.0 (#19420) [930611f](https://github.com/answerbook/vector/commit/930611fe3430b63c043739add7d4f1011e1dfafd) - GitHub
* **deps**: Bump rdkafka from 0.34.0 to 0.35.0 (#19090) [db66247](https://github.com/answerbook/vector/commit/db66247ed48f760807a9e0c23fd1d45105b578eb) - GitHub
* **deps**: Bump redis from 0.23.3 to 0.23.4 (#19240) [8795108](https://github.com/answerbook/vector/commit/87951086c06ab7e7bab7799ec6e559c4009a7c92) - GitHub
* **deps**: Bump redis from 0.23.4 to 0.24.0 (#19319) [09126f9](https://github.com/answerbook/vector/commit/09126f9435bec9872ae0f90f7aa209a8ce75ed7b) - GitHub
* **deps**: Bump reqwest from 0.11.22 to 0.11.23 (#19421) [1fc7a14](https://github.com/answerbook/vector/commit/1fc7a147be02103ea2df6c76cf4bd9f5e95c0c26) - GitHub
* **deps**: Bump rkyv from 0.7.42 to 0.7.43 (#19403) [0a71d9e](https://github.com/answerbook/vector/commit/0a71d9ed167c99544e1ec28df8068f6621faea2c) - GitHub
* **deps**: Bump ryu from 1.0.15 to 1.0.16 (#19351) [2d24e7d](https://github.com/answerbook/vector/commit/2d24e7d7e75463bf72087dfc09b78fd38d5f6158) - GitHub
* **deps**: Bump schannel from 0.1.22 to 0.1.23 (#19475) [32ac1cb](https://github.com/answerbook/vector/commit/32ac1cba5e75c88774f016cb6aa94da5291fa4a7) - GitHub
* **deps**: Bump serde from 1.0.190 to 1.0.192 (#19071) [49a2737](https://github.com/answerbook/vector/commit/49a27375b50e7a97d3dcb114dc1074a7f9ca3d5e) - GitHub
* **deps**: Bump serde from 1.0.192 to 1.0.193 (#19207) [3d284a6](https://github.com/answerbook/vector/commit/3d284a69dece45a1699770b7087584784bf18f30) - GitHub
* **deps**: Bump serde from 1.0.193 to 1.0.194 (#19506) [80d3bf2](https://github.com/answerbook/vector/commit/80d3bf20a9b2c9263fab1f9cd02d6f447dfa306a) - GitHub
* **deps**: Bump serde_json from 1.0.108 to 1.0.109 (#19489) [42ad075](https://github.com/answerbook/vector/commit/42ad075c215cdba5b7a88a812b9f60a40d9a30fd) - GitHub
* **deps**: Bump serde_yaml from 0.9.27 to 0.9.28 (#19439) [7ef8ccb](https://github.com/answerbook/vector/commit/7ef8ccb69c90a20017126517786667473653df0f) - GitHub
* **deps**: Bump serde_yaml from 0.9.28 to 0.9.29 (#19451) [406ec49](https://github.com/answerbook/vector/commit/406ec497488c90d7008b9764ce529f29e0d0656b) - GitHub
* **deps**: Bump serde-wasm-bindgen from 0.6.1 to 0.6.3 (#19354) [3a08e64](https://github.com/answerbook/vector/commit/3a08e64c45c1125f9ef21e9cf0ee173e1069585e) - GitHub
* **deps**: Bump smallvec from 1.11.1 to 1.11.2 (#19113) [90e5044](https://github.com/answerbook/vector/commit/90e5044c7c073545b37c573ce715b7e0544095cc) - GitHub
* **deps**: Bump snap from 1.1.0 to 1.1.1 (#19318) [c3eaf28](https://github.com/answerbook/vector/commit/c3eaf28d34fcbdf90b2cfa318d0e0f50b7781beb) - GitHub
* **deps**: Bump stream-cancel from 0.8.1 to 0.8.2 (#19407) [f5fd79f](https://github.com/answerbook/vector/commit/f5fd79f76734470656e6e8c1ccffd4f86944d204) - GitHub
* **deps**: Bump syn from 2.0.39 to 2.0.41 (#19373) [6e9bb20](https://github.com/answerbook/vector/commit/6e9bb20e6fe95452bfd62b42bb862180d8ba101b) - GitHub
* **deps**: Bump syn from 2.0.41 to 2.0.42 (#19441) [35acd3f](https://github.com/answerbook/vector/commit/35acd3f428193462530373cf955f2fed017f208b) - GitHub
* **deps**: Bump syslog_loose from 0.19.0 to 0.21.0 (#19143) [a639928](https://github.com/answerbook/vector/commit/a6399284fb944e22b8d3174988c56f4d2a1df94c) - GitHub
* **deps**: Bump temp-dir from 0.1.11 to 0.1.12 (#19454) [0a567da](https://github.com/answerbook/vector/commit/0a567da3c732f43473c815b04019deb62a6645dc) - GitHub
* **deps**: Bump tempfile from 3.8.1 to 3.9.0 (#19474) [a8629af](https://github.com/answerbook/vector/commit/a8629afeda956045c3ef87cab8ad34ab2a779632) - GitHub
* **deps**: Bump the clap group with 1 update (#19127) [4a61e36](https://github.com/answerbook/vector/commit/4a61e3650d94853d271cd6a2dbcb936ad8101f75) - GitHub
* **deps**: Bump the clap group with 1 update (#19247) [f5fe318](https://github.com/answerbook/vector/commit/f5fe318581b3b820f94f2d1b2a9a0b455a960482) - GitHub
* **deps**: Bump the clap group with 1 update (#19305) [33eef4c](https://github.com/answerbook/vector/commit/33eef4c3bea59d06c05c62ce07ae8bc035d5da75) - GitHub
* **deps**: Bump the clap group with 1 update (#19402) [62817ea](https://github.com/answerbook/vector/commit/62817ea1176a969aa93c20d055253aca359049ae) - GitHub
* **deps**: Bump the clap group with 2 updates (#19501) [0f29afa](https://github.com/answerbook/vector/commit/0f29afa442f7e9a08809867d38bf47593e1925c9) - GitHub
* **deps**: Bump the futures group with 1 update (#19461) [b00d4e3](https://github.com/answerbook/vector/commit/b00d4e33c63a8554769cb1b85cec1acb01f0736f) - GitHub
* **deps**: Bump the prost group with 3 updates (#19180) [5e0d641](https://github.com/answerbook/vector/commit/5e0d64136ecad49668f904bcd8c218a0cdc205ee) - GitHub
* **deps**: Bump the prost group with 3 updates (#19213) [6ee00fa](https://github.com/answerbook/vector/commit/6ee00fa69a4ad372239434f5a715677af3c5f930) - GitHub
* **deps**: Bump thiserror from 1.0.50 to 1.0.51 (#19406) [03cbc2e](https://github.com/answerbook/vector/commit/03cbc2ea081eab14a2adf8b1a2125b2b6e11f15b) - GitHub
* **deps**: Bump tokio from 1.33.0 to 1.34.0 (#19112) [473b720](https://github.com/answerbook/vector/commit/473b720876bc0886ea2518d86b4a05a6d9bd43c7) - GitHub
* **deps**: Bump tokio from 1.34.0 to 1.35.0 (#19348) [6b19015](https://github.com/answerbook/vector/commit/6b190157346735fbf8b2c8e4ad60450d32ec0303) - GitHub
* **deps**: Bump tokio from 1.35.0 to 1.35.1 (#19430) [5ef73ee](https://github.com/answerbook/vector/commit/5ef73ee665a395af23deb93a8138038745038c92) - GitHub
* **deps**: Bump tokio-openssl from 0.6.3 to 0.6.4 (#19366) [07cdf75](https://github.com/answerbook/vector/commit/07cdf75150b7b1587e8ff43dbc3d42ff909e56e0) - GitHub
* **deps**: Bump toml from 0.8.6 to 0.8.8 (#19070) [2913cfe](https://github.com/answerbook/vector/commit/2913cfea3f279cb12f66465ad7003cf3cf9770b7) - GitHub
* **deps**: Bump tracing-subscriber from 0.3.17 to 0.3.18 (#19144) [fdf3742](https://github.com/answerbook/vector/commit/fdf37425541ea19d95163501587c2ba2d61d6642) - GitHub
* **deps**: Bump typetag from 0.2.13 to 0.2.14 (#19353) [e82ac37](https://github.com/answerbook/vector/commit/e82ac3702305c1de8c8163b5b5f31067e1262d4c) - GitHub
* **deps**: Bump typetag from 0.2.14 to 0.2.15 (#19504) [9bdaaa2](https://github.com/answerbook/vector/commit/9bdaaa27948057c53e9a107a92b454657a67d5f7) - GitHub
* **deps**: Bump url from 2.4.1 to 2.5.0 (#19232) [5f0b2e8](https://github.com/answerbook/vector/commit/5f0b2e87d7e1f096e32c2f383fb7cdbdea120668) - GitHub
* **deps**: Bump uuid from 1.5.0 to 1.6.0 (#19195) [3ca8c1c](https://github.com/answerbook/vector/commit/3ca8c1cdaa567dfc6240be29eb13919b660ca0dd) - GitHub
* **deps**: Bump uuid from 1.6.0 to 1.6.1 (#19206) [ef8c602](https://github.com/answerbook/vector/commit/ef8c6023f99e7f49e06abc15bce85d1e152169c2) - GitHub
* **deps**: Bump wasm-bindgen from 0.2.88 to 0.2.89 (#19248) [cfd9e5e](https://github.com/answerbook/vector/commit/cfd9e5e64ebb21ebb241f262de2ec5cd5f64cc57) - GitHub
* **deps**: Bump wiremock from 0.5.21 to 0.5.22 (#19275) [f009bc1](https://github.com/answerbook/vector/commit/f009bc1ae40719598f59dba5cd6f12844655012b) - GitHub
* **deps**: Bump zerocopy from 0.7.21 to 0.7.31 (#19394) [136402a](https://github.com/answerbook/vector/commit/136402adb8dd35242c0aa2cfed69690a81ec82b9) - GitHub
* **deps**: Replace trust_dns_proto with hickory_proto (#19095) [ec31d03](https://github.com/answerbook/vector/commit/ec31d03d1b5630a49a4852cf66a747a4b14db75c) - GitHub
* **deps**: Revert update lading to 0.20.0 (#19259) [92d4102](https://github.com/answerbook/vector/commit/92d4102df59f3239b9aa062caa28e19ead7f3c57) - Jesse Szwedko
* **deps**: Update h2 [70b0702](https://github.com/answerbook/vector/commit/70b0702565a93504899aab5644d299ceea77789a) - Jesse Szwedko
* **deps**: Update lading to 0.20.0 (#19259) [ce615d0](https://github.com/answerbook/vector/commit/ce615d0d23d0a468721a26988a77c166b3288506) - GitHub
* **deps**: update nextest version to 0.9.64 (#19292) [b0aa8a0](https://github.com/answerbook/vector/commit/b0aa8a0e729fbb0ef6c7997aecc17819ef63fd70) - GitHub
* **deps**: Update rust_decimal crate and dependencies [5b89574](https://github.com/answerbook/vector/commit/5b8957408c2b18b725a6915a13e4305258035331) - Jesse Szwedko
* **deps**: Update smp to 0.11.0 (#19270) [d7edc74](https://github.com/answerbook/vector/commit/d7edc74623ce9e3b4948727aca2f382a9bc7b6de) - GitHub
* **deps**: Update VRL to 0.9.1 (#19455) [3191397](https://github.com/answerbook/vector/commit/3191397115a63ed7efb70a21035bae3bfeee96c9) - GitHub
* **deps**: Update VRL to use `KeyString` type wrapper (#19069) [9733dd6](https://github.com/answerbook/vector/commit/9733dd639fdaccbbf4df16715a3f03b61d3df5f5) - GitHub
* **deps**: update VRL to v0.9.0 (#19368) [0e600ec](https://github.com/answerbook/vector/commit/0e600ec5cbcf3b74cfdeddd1a1afc88afce00b93) - GitHub
* **deps**: Updated Fedora versions used in testing to latest (#19242) [a9b9705](https://github.com/answerbook/vector/commit/a9b97052970be51448682a27bcb1a513b79dd2a7) - GitHub
* **dev**: Bump Vector to v0.35.0 (#19077) [da06f2a](https://github.com/answerbook/vector/commit/da06f2a308e375a2dc4f4bef5b711954220ba3ce) - GitHub
* **dev**: remove component-validation-runner feature from defaults (#19324) [59dd1fa](https://github.com/answerbook/vector/commit/59dd1fad4b2ea53e47c17baabeb7ab9b922460fc) - GitHub
* **dev**: Simplify a few tiny import issues in `vector-lib` (#19066) [2a0404a](https://github.com/answerbook/vector/commit/2a0404adc35517b3b002e736cb3046557c1e0c02) - GitHub
* **docs**: Add alpha to traces and beta to metrics in descriptions (#19139) [8a1ed17](https://github.com/answerbook/vector/commit/8a1ed17a08c9a7691528e0df2e0b223fa5ca2ec6) - GitHub
* **docs**: Add banner alerting people of package migration (#19714) [48eac29](https://github.com/answerbook/vector/commit/48eac298802a0e483b4ea935478f87915fb99c25) - Jesse Szwedko
* **docs**: add timestamp comparison example (#19266) [c5d6917](https://github.com/answerbook/vector/commit/c5d6917a079f5a203b5665b598cbf0e52bbc9dfb) - GitHub
* **docs**: document new snappy vrl functions (#19081) [cbf3b78](https://github.com/answerbook/vector/commit/cbf3b783f44a952ac5d57d38e92b3df78e196274) - GitHub
* **docs**: Remove references to CloudSmith from docs (#19377) [dff8ca3](https://github.com/answerbook/vector/commit/dff8ca3aee4a628885765f3fe1d2af70d2f4b969) - GitHub
* **docs**: Remove stale docs content around schemas (#19256) [8053443](https://github.com/answerbook/vector/commit/8053443752d9b01a13b0efb322a1c52a3e0b940f) - GitHub
* **docs**: Replace setup.vector.dev references (#19080) [a4fa2c0](https://github.com/answerbook/vector/commit/a4fa2c0ca5bae349e48ba58d21ea2dec8c270198) - GitHub
* **docs**: update a few more examples to YAML (#19103) [89cd32e](https://github.com/answerbook/vector/commit/89cd32ed4f604f20d66c3e30c57cf3e63b7cba70) - GitHub
* **external docs**: Second batch of editorial edits for the Functions doc (#19284) [79f2d22](https://github.com/answerbook/vector/commit/79f2d2231633945f6b243fa3d1282ff8bb2254a9) - GitHub
* List checks to be run prior to submitting a PR in `CONTRIBUTING.md` (#19118) [b0c09e1](https://github.com/answerbook/vector/commit/b0c09e1ce16e9afaa1a13a4ba249c90d7c5453df) - GitHub
* **observability**: Remove deprecated HTTP metrics (#19447) [af4de5e](https://github.com/answerbook/vector/commit/af4de5eae6ad454fccd47fc933ac02bafa579446) - GitHub
* **observability**: Simplify the default log targets selection (#19359) [55ec7f1](https://github.com/answerbook/vector/commit/55ec7f1d3aa18081cfe44dcadf6384c0e86c8abc) - GitHub
* **performance**: Use welch consignor (#19273) [46fcbf4](https://github.com/answerbook/vector/commit/46fcbf481e93d044275fd48ccecd369b242235b7) - GitHub
* **releasing, kubernetes**: Bump manifests to v0.29.0 of Helm chart (#19178) [aa2d360](https://github.com/answerbook/vector/commit/aa2d3603bc0784279e408e3907ffe94851727b6d) - GitHub
* **releasing**: Add deprecation note about respositories.timber.io deprecation (#19078) [2dee1f2](https://github.com/answerbook/vector/commit/2dee1f2547ec98203bd21a13dbca8662606310c6) - GitHub
* **releasing**: Add known issue for Datadog Metrics sink in v0.34.0 (#19122) [1435613](https://github.com/answerbook/vector/commit/14356138ff4be7412c287be0b8c68f63ee8f58b7) - GitHub
* **releasing**: Add known issue for protobuf encoder in v0.34.0 (#19244) [79c7c6d](https://github.com/answerbook/vector/commit/79c7c6d805948c789b5c829c88535ce5bf93a155) - GitHub
* **releasing**: Add upgrade note about TOML breaking change to v0.34.0 (#19120) [c7ae2a6](https://github.com/answerbook/vector/commit/c7ae2a631a2f7a52e336970c5a8656601e63aa14) - GitHub
* **releasing**: Fix formatting for v0.34.0 release note (#19085) [8add558](https://github.com/answerbook/vector/commit/8add5585d5f7eec5c217c5db5fa6c4eaca7be188) - GitHub
* **releasing**: Prepare v0.34.0 release [66667dd](https://github.com/answerbook/vector/commit/66667dd291482a440c5eb2032ef3cbfb7377b53b) - Jesse Szwedko
* **releasing**: Prepare v0.34.1 release [6ad9c53](https://github.com/answerbook/vector/commit/6ad9c536334e6042810b1aadb7d9248e5f18e491) - Jesse Szwedko
* **releasing**: Prepare v0.34.2 release [f495661](https://github.com/answerbook/vector/commit/f495661b79769ea7df130a6a9e2fbc99db20b7b5) - Jesse Szwedko
* **releasing**: Prepare v0.35.0 release [e57c0c0](https://github.com/answerbook/vector/commit/e57c0c0e64bea6508ba0aae148d93525a4788669) - Jesse Szwedko
* **releasing**: Prepare v0.35.1 release [3ff039a](https://github.com/answerbook/vector/commit/3ff039a64634e82862734cf947812d8a3d5ff93d) - Jesse Szwedko
* **releasing**: Update minor release template step for updating vector.dev (#19109) [b58b0d4](https://github.com/answerbook/vector/commit/b58b0d43d5bc20664cd016f125ceab9331d6e5cf) - GitHub
* replace anymap with a simple hashmap (#19335) [1d5b881](https://github.com/answerbook/vector/commit/1d5b8811c7cdf08a525bf64d8015fb0f688d945e) - GitHub
* **security**: Ignore RUSTSEC-2023-0071 for now (#19263) [5194bde](https://github.com/answerbook/vector/commit/5194bdeec1f67416779cf8fc8be0e1fcccda671d) - GitHub
* Update OpenTelemetry Protobuf Definitions to v1.0.0 (#19188) [24034c7](https://github.com/answerbook/vector/commit/24034c720845fe784a4ad807b52ea386fb58a992) - GitHub
* Update the version of cue we are using to 0.7.0 (#19449) [05d827d](https://github.com/answerbook/vector/commit/05d827d0e35029d305cfb9f56af036e46b185c8a) - GitHub
* **website**: Fix commenting step on workflow (#19134) [3029558](https://github.com/answerbook/vector/commit/30295584f8382d4b194dca70616b060eeb7929ee) - GitHub
* **website**: Remove build files (#19199) [20ff4c8](https://github.com/answerbook/vector/commit/20ff4c8aee5048dcd50629e3acadb9a41e97998c) - GitHub
* **website**: Small fix to function call (#19079) [71fe94c](https://github.com/answerbook/vector/commit/71fe94c1998812fee04d4ba7a147127c5a693a1b) - GitHub
* **website**: WEB-4247 | Update references from s3 to setup.vector.dev (#19149) [52068d4](https://github.com/answerbook/vector/commit/52068d4eb9019d948539836bd5bbd5b0bc33fac9) - GitHub
* **website**: WEB-4275 | Update Navigation (#19186) [483970e](https://github.com/answerbook/vector/commit/483970e1de2fc3886ca2436220f0818efccac00e) - GitHub


### Features

* **apt platform**: Add datadog-signing-keys package as recommended (#19369) [632fe21](https://github.com/answerbook/vector/commit/632fe210ce28d178e22d8d35f4fdce348b63c212) - GitHub
* **aws_cloudwatch_logs sink**: add configurable log retention (#18865) [104984f](https://github.com/answerbook/vector/commit/104984f0718ac48725aa0c93356fc07ccb197f54) - GitHub
* **http_server source**: add all headers to the namespace metadata (#18922) [8f16a00](https://github.com/answerbook/vector/commit/8f16a00636508a8f44f42bdf9a6ecba0ec6b0d60) - GitHub
* **log_to_metric**: dynamically convert all logs to metrics (#19160) [114715e](https://github.com/answerbook/vector/commit/114715eaf5584d191e8a44688d40ff5fc6079147) - GitHub
* **new codecs**: introduce avro (#19342) [bd2cff8](https://github.com/answerbook/vector/commit/bd2cff83a6df2e0e287c40000bd9d7e9d24a59dc) - GitHub
* **unittests**: add vrl as test input (#19107) [b595fb4](https://github.com/answerbook/vector/commit/b595fb4c6b3f31e4738f6cec1c8021d6cdb5c79f) - GitHub


### Miscellaneous

* Merge pull request #482 from answerbook/feature/LOG-19380 [732872d](https://github.com/answerbook/vector/commit/732872dffdda29a7f4f99ce6a4735befd536d45a) - GitHub [LOG-19380](https://logdna.atlassian.net/browse/LOG-19380)
* Merge remote-tracking branch 'origin/master' into feature/LOG-19380 [841015a](https://github.com/answerbook/vector/commit/841015ac36a4cfd3c2f388ccc9a8d13277051853) - Chris Nixon [LOG-19380](https://logdna.atlassian.net/browse/LOG-19380)
* Merge branch 'master' into feature/LOG-19380 [169775a](https://github.com/answerbook/vector/commit/169775aada7f0a2526268834c671b560ec52dde3) - Chris Nixon [LOG-19380](https://logdna.atlassian.net/browse/LOG-19380)
* Merge remote-tracking branch 'origin/master' into feature/LOG-19380 [0f122bb](https://github.com/answerbook/vector/commit/0f122bb67c9ba764e869eee34116805880eba5bd) - Chris Nixon [LOG-19380](https://logdna.atlassian.net/browse/LOG-19380)
* Merge remote-tracking branch 'origin/master' into feature/LOG-19380 [1dd7883](https://github.com/answerbook/vector/commit/1dd7883bbe87d1ba042742aff59b6f356faaed83) - Chris Nixon [LOG-19380](https://logdna.atlassian.net/browse/LOG-19380)
* Merge remote-tracking branch 'origin/master' into feature/LOG-19380 [c7b3204](https://github.com/answerbook/vector/commit/c7b3204666debc8d56c3071639ffcba82de89f26) - Chris Nixon [LOG-19380](https://logdna.atlassian.net/browse/LOG-19380)
* Merge upstream/v0.35 into master [4049c12](https://github.com/answerbook/vector/commit/4049c1243e9965d4f20dd3ca24f5f16677e13e2a) - Chris Nixon
* Update RUM domain (#19367) [a0df2ac](https://github.com/answerbook/vector/commit/a0df2ac6f0f4a64668323c12ee6744a68e959175) - GitHub
* enhancement(packaging):Add soft dependency on datadog-signing-keys (#19343) [fba396c](https://github.com/answerbook/vector/commit/fba396c03f4621179646a4c5d07aec905493cfe4) - GitHub
* chore(docs):Add Obs Pipelines to docs (#19201) [3a090a2](https://github.com/answerbook/vector/commit/3a090a26cda65f776d8fb46aea9b4bfc83c9b134) - GitHub
* Revert "chore(ci): Bump actions/labeler from 4 to 5" (#19344) [e2dc7cc](https://github.com/answerbook/vector/commit/e2dc7ccafc086c5ff5fe622bb76d7df480fb7632) - GitHub
* Pront/config target path display (#19331) [038a715](https://github.com/answerbook/vector/commit/038a7158c3f688812e1d38bc766bfa85757a2aa2) - GitHub
* feat(splunk_hec_logs splunk_hec_metrics humio sink) Allow host and timestamp key fields to be namespaceable (#19086) [aff87f0](https://github.com/answerbook/vector/commit/aff87f0aa33c213018b6c402d79ae4b03dd264a2) - GitHub
* Update README.md (#19142) [3fe1a65](https://github.com/answerbook/vector/commit/3fe1a6567debef4849f1dbbed06765c4231f8314) - GitHub
* **aws_sns sink**: Add documentation data (#19715) [58be28d](https://github.com/answerbook/vector/commit/58be28d5b0a144de58522aa2ec67700b7878049d) - Jesse Szwedko
* **ci**: group artifact dependabot upgrades (#19390) [b7b8081](https://github.com/answerbook/vector/commit/b7b8081b63ce4fbcc74f4cb31e9f5827b6594b3a) - GitHub
* **datadog service**: Add datadog global config options (#18929) [b297070](https://github.com/answerbook/vector/commit/b297070f9ab5f90d6fdb7ffe36d7d59a86cf6f1d) - GitHub
* **file sink, aws_s3 sink, gcp_cloud_storage**: configurable filename timezone (#18506) [5a55567](https://github.com/answerbook/vector/commit/5a5556717c37dd2e5c426ef6d48d1a269e440ba3) - GitHub
* fix mismatch in config vs config file name (#19469) [9d164b6](https://github.com/answerbook/vector/commit/9d164b66122dd059c51651a174cb6a9cb62827a4) - GitHub
* fix truncate arguments (#19068) [e9e7336](https://github.com/answerbook/vector/commit/e9e7336462133f14db8ba308dbb2e85dcae547b9) - GitHub
* **install vector**: Allow downloading specific versions of Vector (#19408) [20682e7](https://github.com/answerbook/vector/commit/20682e7724c907b676d96ff7660d1c675e7fb56a) - GitHub
* **kubernetes**: Update manifests to chart v0.29.1 (#19494) [deb31e2](https://github.com/answerbook/vector/commit/deb31e2de75f9d5a1ba8713985416c4b8049e654) - GitHub
* **networking, sinks**: add full jitter to retry backoff policy (#19106) [c668def](https://github.com/answerbook/vector/commit/c668defa04ab8f499fd24ff3ba1c7bf8932122d3) - GitHub
* **observability**: add buffer `buffer_send_duration_seconds` metric (#19022) [c68b2b8](https://github.com/answerbook/vector/commit/c68b2b83b181b16cf467f6d40ca471f932d4ed10) - GitHub
* **performance**: Enable jemallocator for all non rust-code (#19340) [d2e9f65](https://github.com/answerbook/vector/commit/d2e9f65a25f490a54c24c5f57d9c205df19aa231) - GitHub
* **sources**: Add `keepalive.max_connection_age_secs` config option to HTTP-server sources (#19141) [59e6d36](https://github.com/answerbook/vector/commit/59e6d36862f13ff21902610e574fb2d83890ab47) - GitHub
* **syslog source**: specify that the unix mode supports stream sockets only (#19399) [3fb9922](https://github.com/answerbook/vector/commit/3fb9922c648f867166bd065793a573c4fb65ed83) - GitHub
* **vrl**: update tests for 2024 (#19493) [cbafbc5](https://github.com/answerbook/vector/commit/cbafbc5af24fb8be7046f32287e2c2d3827e9e4a) - GitHub


### Performance Improvements

* **tap**: avoid compiling globs frequently during tap (#19255) [d7453ca](https://github.com/answerbook/vector/commit/d7453ca03dbffecf0ca54a7bff8788838f78ac60) - GitHub


### **BREAKING CHANGES**

* **observability:** Remove deprecated HTTP metrics (#19447)
* **throttle transform:** make `events_discarded_total` internal metric with `key` tag opt-in (#19083)

# [3.23.0](https://github.com/answerbook/vector/compare/v3.22.8...v3.23.0) (2024-07-03)


### Features

* **otlp**: opentelemetry metric destination [a796ede](https://github.com/answerbook/vector/commit/a796ede474403d0bf920dd5e3ea47105dd4d0010) - Sergey Opria [LOG-19372](https://logdna.atlassian.net/browse/LOG-19372)


### Miscellaneous

* Merge pull request #479 from answerbook/sopria/LOG-19372 [93412a7](https://github.com/answerbook/vector/commit/93412a7bcff58a7f6b8debb9dce2b2ed38a379c9) - GitHub [LOG-19372](https://logdna.atlassian.net/browse/LOG-19372)

## [3.22.8](https://github.com/answerbook/vector/compare/v3.22.7...v3.22.8) (2024-07-03)


### Bug Fixes

* **remote_tasks**: Implement timeout for task execution [c1ed6b4](https://github.com/answerbook/vector/commit/c1ed6b43c24e9cef26f9fb6faeb611905c354ff6) - Dan Hable [LOG-19271](https://logdna.atlassian.net/browse/LOG-19271)

## [3.22.7](https://github.com/answerbook/vector/compare/v3.22.6...v3.22.7) (2024-07-02)


### Chores

* **kafka**: add additional debug logging around consumers [1609bbe](https://github.com/answerbook/vector/commit/1609bbe67c071405db43e5f480a06357dbff4b22) - Dan Hable [LOG-20224](https://logdna.atlassian.net/browse/LOG-20224)

## [3.22.6](https://github.com/answerbook/vector/compare/v3.22.5...v3.22.6) (2024-06-29)


### Bug Fixes

* **otlp**: grouping traces spans same as received from source [044522b](https://github.com/answerbook/vector/commit/044522b977b7d8b9898461e05dc385d512e57426) - Sergey Opria [LOG-20205](https://logdna.atlassian.net/browse/LOG-20205) [LOG-20184](https://logdna.atlassian.net/browse/LOG-20184)


### Miscellaneous

* Merge pull request #478 from answerbook/sopria/LOG-20184 [15e76ad](https://github.com/answerbook/vector/commit/15e76ad33ea6d14b3720741ece14c76537f5f086) - GitHub [LOG-20184](https://logdna.atlassian.net/browse/LOG-20184)

## [3.22.5](https://github.com/answerbook/vector/compare/v3.22.4...v3.22.5) (2024-06-27)


### Chores

* tweaking default partition to new resource defaults [fe94805](https://github.com/answerbook/vector/commit/fe9480567968c9b5d4c85662a2399c7749a85a06) - Adam Holmberg [LOG-19700](https://logdna.atlassian.net/browse/LOG-19700)


### Miscellaneous

* Merge pull request #477 from answerbook/holmberg/LOG-19700 [cfb07f2](https://github.com/answerbook/vector/commit/cfb07f2cc484d618f1124f76408426a0d95d2e8e) - GitHub [LOG-19700](https://logdna.atlassian.net/browse/LOG-19700)

## [3.22.4](https://github.com/answerbook/vector/compare/v3.22.3...v3.22.4) (2024-06-21)


### Chores

* remove unused attributes from the default partition [f2d9160](https://github.com/answerbook/vector/commit/f2d91606999290b535403f75220210be11d0c760) - Adam Holmberg [LOG-19700](https://logdna.atlassian.net/browse/LOG-19700)


### Miscellaneous

* Merge pull request #476 from answerbook/feature/LOG-19700 [032f879](https://github.com/answerbook/vector/commit/032f8793bab3371ca66fcfc3f4a1649daad439ee) - GitHub [LOG-19700](https://logdna.atlassian.net/browse/LOG-19700)

## [3.22.3](https://github.com/answerbook/vector/compare/v3.22.2...v3.22.3) (2024-06-18)


### Chores

* **otlp**: add arbitrary headers for otlp destination config [88f2759](https://github.com/answerbook/vector/commit/88f27593aabd2cfed6bbe4615d64a1cad9d432da) - Sergey Opria [LOG-20123](https://logdna.atlassian.net/browse/LOG-20123)


### Miscellaneous

* Merge pull request #475 from answerbook/sopria/LOG-20123 [f7b0d23](https://github.com/answerbook/vector/commit/f7b0d23a4eef7d9ea7cdd0723038ebfbd33e3c06) - GitHub [LOG-20123](https://logdna.atlassian.net/browse/LOG-20123)

## [3.22.2](https://github.com/answerbook/vector/compare/v3.22.1...v3.22.2) (2024-06-18)


### Bug Fixes

* **kafka source**: Reorder message consume loop to avoid memory growth [664f7d8](https://github.com/answerbook/vector/commit/664f7d82bb3cfb3d808c6141fa3654460251be90) - Dan Hable [LOG-20069](https://logdna.atlassian.net/browse/LOG-20069)

## [3.22.1](https://github.com/answerbook/vector/compare/v3.22.0...v3.22.1) (2024-06-12)


### Chores

* **otlp**: get rid of the otlp healthcheck [be8f06f](https://github.com/answerbook/vector/commit/be8f06f552425d9b4cc1bf966e0441be7d13a2d2) - Sergey Opria [LOG-20060](https://logdna.atlassian.net/browse/LOG-20060)


### Miscellaneous

* Merge pull request #472 from answerbook/sopria/LOG-20060 [45f5e3f](https://github.com/answerbook/vector/commit/45f5e3fa3bcad860865014cd93c89f1de7bf24f2) - GitHub [LOG-20060](https://logdna.atlassian.net/browse/LOG-20060)

# [3.22.0](https://github.com/answerbook/vector/compare/v3.21.4...v3.22.0) (2024-06-12)


### Chores

* remove vector hpas [02bf63f](https://github.com/answerbook/vector/commit/02bf63f234275d0ef58cd855815786a1aed15e9a) - Adam Holmberg [LOG-19700](https://logdna.atlassian.net/browse/LOG-19700)


### Features

* make CPU limits optional for vector sts [dda4b6e](https://github.com/answerbook/vector/commit/dda4b6ef648de93f8de7f8e07ee976d47620793d) - Adam Holmberg [LOG-19700](https://logdna.atlassian.net/browse/LOG-19700)


### Miscellaneous

* Merge pull request #471 from answerbook/feature/LOG-19700 [ef0137d](https://github.com/answerbook/vector/commit/ef0137d1b498f067a3a0e57afeb0ce9063aa8336) - GitHub [LOG-19700](https://logdna.atlassian.net/browse/LOG-19700)

## [3.21.4](https://github.com/answerbook/vector/compare/v3.21.3...v3.21.4) (2024-06-06)


### Bug Fixes

* **otlp sink**: make otlp sink healthcheck configurable [146a30d](https://github.com/answerbook/vector/commit/146a30d27afa8654774935933544bc766e65525a) - Sergey Opria [LOG-20060](https://logdna.atlassian.net/browse/LOG-20060)


### Miscellaneous

* Merge pull request #470 from answerbook/sopria/LOG-20060 [b8cdba4](https://github.com/answerbook/vector/commit/b8cdba4b84f952c1956834212e8781f8c9b1d85d) - GitHub [LOG-20060](https://logdna.atlassian.net/browse/LOG-20060)

## [3.21.3](https://github.com/answerbook/vector/compare/v3.21.2...v3.21.3) (2024-06-04)


### Bug Fixes

* **aggregate-v2**: fix cardinality limit check [db927a2](https://github.com/answerbook/vector/commit/db927a25ef852d6a91105f15787be8d5947099d1) - Dan Hable [LOG-20037](https://logdna.atlassian.net/browse/LOG-20037)

## [3.21.2](https://github.com/answerbook/vector/compare/v3.21.1...v3.21.2) (2024-05-30)


### Bug Fixes

* **consolidation**: bad connection string failures [a7a643e](https://github.com/answerbook/vector/commit/a7a643e48c5661dd41aa207f833fb5cb330d8812) - dominic-mcallister-logdna [LOG-19993](https://logdna.atlassian.net/browse/LOG-19993)


### Miscellaneous

* Merge pull request #468 from answerbook/dominic/LOG-19993 [13927de](https://github.com/answerbook/vector/commit/13927ded7e1b59f93d35641e259c606407dff354) - GitHub [LOG-19993](https://logdna.atlassian.net/browse/LOG-19993)

## [3.21.1](https://github.com/answerbook/vector/compare/v3.21.0...v3.21.1) (2024-05-30)


### Bug Fixes

* **postgresql**: Nullify bad JSON serialization [a603ae4](https://github.com/answerbook/vector/commit/a603ae4ea895c71399f41ab908cb4b6050d5a2fa) - Darin Spivey [LOG-19992](https://logdna.atlassian.net/browse/LOG-19992)

# [3.21.0](https://github.com/answerbook/vector/compare/v3.20.13...v3.21.0) (2024-05-28)


### Features

* **aggregate**: Increase default cardinality limit [2726250](https://github.com/answerbook/vector/commit/27262506dc617cab2eddef719b9198920776ec6b) - Dan Hable [LOG-19908](https://logdna.atlassian.net/browse/LOG-19908)

## [3.20.13](https://github.com/answerbook/vector/compare/v3.20.12...v3.20.13) (2024-05-22)


### Bug Fixes

* **postgresql**: Remove unicode nulls from values [dce512c](https://github.com/answerbook/vector/commit/dce512ce943693b30f3536a130e3ef91bfe68242) - Darin Spivey [LOG-19926](https://logdna.atlassian.net/browse/LOG-19926)

## [3.20.12](https://github.com/answerbook/vector/compare/v3.20.11...v3.20.12) (2024-05-22)


### Bug Fixes

* **kafka source**: loop on fatal error, creating new client [4441930](https://github.com/answerbook/vector/commit/4441930bc9cca6080127be5ff0dacc4c682a0ecc) - Adam Holmberg [LOG-19805](https://logdna.atlassian.net/browse/LOG-19805)


### Miscellaneous

* Merge pull request #464 from answerbook/holmberg/LOG-19805 [6cb65fd](https://github.com/answerbook/vector/commit/6cb65fd522243bac7e139f4012db328456790bd0) - GitHub [LOG-19805](https://logdna.atlassian.net/browse/LOG-19805)

## [3.20.11](https://github.com/answerbook/vector/compare/v3.20.10...v3.20.11) (2024-05-21)


### Bug Fixes

* **sink**: otlp metrics tags and name normalize [458f099](https://github.com/answerbook/vector/commit/458f0994dbe97d5cfac8e1e39084ece947011445) - Sergey Opria [LOG-19601](https://logdna.atlassian.net/browse/LOG-19601)


### Miscellaneous

* Merge pull request #462 from answerbook/sopria/LOG-19601 [71cdf9e](https://github.com/answerbook/vector/commit/71cdf9e47c6395b3b996e6d803a911f8689f55ed) - GitHub [LOG-19601](https://logdna.atlassian.net/browse/LOG-19601)

## [3.20.10](https://github.com/answerbook/vector/compare/v3.20.9...v3.20.10) (2024-05-15)


### Bug Fixes

* **ci**: Turn on commitlint [a03e9c6](https://github.com/answerbook/vector/commit/a03e9c6c43d34ea2d6763a747384ed77891b1a2d) - Darin Spivey [LOG-19886](https://logdna.atlassian.net/browse/LOG-19886)

## [3.20.9](https://github.com/answerbook/vector/compare/v3.20.8...v3.20.9) (2024-05-10)


### Bug Fixes

* **sink**: OTLP sink panics when event is invalid or cannot be detected [2d8647f](https://github.com/answerbook/vector/commit/2d8647f0fa8e2434ba255283878556b2d0f29a55) - Sergey Opria [LOG-19721](https://logdna.atlassian.net/browse/LOG-19721)


### Miscellaneous

* Merge pull request #461 from answerbook/sopria/LOG-19721 [d70590a](https://github.com/answerbook/vector/commit/d70590ab04d19421043c9b488c32b9559dce97f5) - GitHub [LOG-19721](https://logdna.atlassian.net/browse/LOG-19721)

## [3.20.8](https://github.com/answerbook/vector/compare/v3.20.7...v3.20.8) (2024-05-08)


### Bug Fixes

* **s3-source**: expose errors in user logs [acf5400](https://github.com/answerbook/vector/commit/acf54002233399171d918e2d8305bdd0834a0771) - Mike Del Tito [LOG-19534](https://logdna.atlassian.net/browse/LOG-19534)


### Miscellaneous

* Merge pull request #460 from answerbook/mdeltito/LOG-19534 [00c1add](https://github.com/answerbook/vector/commit/00c1add454c43408ff3cf036b3855543c0e30e65) - GitHub [LOG-19534](https://logdna.atlassian.net/browse/LOG-19534)

## [3.20.7](https://github.com/answerbook/vector/compare/v3.20.6...v3.20.7) (2024-05-06)


### Bug Fixes

* **s3 consolidation**: memory usage [b7379ba](https://github.com/answerbook/vector/commit/b7379ba7d9c51c348eb408c1a7c48ad0e3749dca) - dominic-mcallister-logdna [LOG-19824](https://logdna.atlassian.net/browse/LOG-19824)


### Miscellaneous

* Merge pull request #459 from answerbook/dominic/LOG-19824 [140e475](https://github.com/answerbook/vector/commit/140e47503513605bc441d0cf436654a99d40659a) - GitHub [LOG-19824](https://logdna.atlassian.net/browse/LOG-19824)

## [3.20.6](https://github.com/answerbook/vector/compare/v3.20.5...v3.20.6) (2024-05-06)


### Bug Fixes

* **source**: Convert OTLP metric into Mezmo metric format. [c639196](https://github.com/answerbook/vector/commit/c6391965d347a5452307285feae8b447c467ce82) - Sergey Opria [LOG-19601](https://logdna.atlassian.net/browse/LOG-19601)


### Miscellaneous

* Merge pull request #458 from answerbook/sopria/LOG-19601 [f73d4d0](https://github.com/answerbook/vector/commit/f73d4d01212c9f16871d2c9fd161537f2e5f1453) - GitHub [LOG-19601](https://logdna.atlassian.net/browse/LOG-19601)

## [3.20.5](https://github.com/answerbook/vector/compare/v3.20.4...v3.20.5) (2024-05-01)


### Bug Fixes

* **user_trace**: Move the `captured_data` wrapper to the log fn [27e1f9a](https://github.com/answerbook/vector/commit/27e1f9ae137979795849cf0d58c10108f0f9fcb5) - Darin Spivey [LOG-19802](https://logdna.atlassian.net/browse/LOG-19802)

## [3.20.4](https://github.com/answerbook/vector/compare/v3.20.3...v3.20.4) (2024-05-01)


### Bug Fixes

* **s3**: multipart file consolidation newlines [14f17f6](https://github.com/answerbook/vector/commit/14f17f6f0ed49e8b75e0b7c71e82078fd81ea857) - dominic-mcallister-logdna [LOG-19797](https://logdna.atlassian.net/browse/LOG-19797)


### Miscellaneous

* Merge pull request #455 from answerbook/dominic/LOG-19797 [406dc17](https://github.com/answerbook/vector/commit/406dc174fba5f135eb5ea2477de902cc3fe87df5) - GitHub [LOG-19797](https://logdna.atlassian.net/browse/LOG-19797)

## [3.20.3](https://github.com/answerbook/vector/compare/v3.20.2...v3.20.3) (2024-05-01)


### Bug Fixes

* **http**: handle and report partial failures with 207 status [744386a](https://github.com/answerbook/vector/commit/744386a8ac8bfb149470fab88bd888256ad8dc4e) - Mike Del Tito [LOG-19799](https://logdna.atlassian.net/browse/LOG-19799)


### Miscellaneous

* Merge pull request #456 from answerbook/mdeltito/LOG-19799 [0d680df](https://github.com/answerbook/vector/commit/0d680df777b29525103bd1b7f109aa676cc97af4) - GitHub [LOG-19799](https://logdna.atlassian.net/browse/LOG-19799)

## [3.20.2](https://github.com/answerbook/vector/compare/v3.20.1...v3.20.2) (2024-05-01)


### Bug Fixes

* **user_trace**: captured_data shape incorrect [0300ddc](https://github.com/answerbook/vector/commit/0300ddc5824234dd064151086aab11b1ad47b390) - Mike Del Tito [LOG-19789](https://logdna.atlassian.net/browse/LOG-19789)


### Miscellaneous

* Merge pull request #454 from answerbook/mdeltito/LOG-19789-fix [c5fffba](https://github.com/answerbook/vector/commit/c5fffbaa9b2f5bb69361bfe2592f60f8b3df4c7e) - GitHub [LOG-19789](https://logdna.atlassian.net/browse/LOG-19789)

## [3.20.1](https://github.com/answerbook/vector/compare/v3.20.0...v3.20.1) (2024-04-30)


### Bug Fixes

* **user_trace**: log http error responses as captured_data [6870759](https://github.com/answerbook/vector/commit/6870759b55d1dc75b0958761141badbf66ce950c) - Mike Del Tito [LOG-19789](https://logdna.atlassian.net/browse/LOG-19789)


### Miscellaneous

* Merge pull request #453 from answerbook/mdeltito/LOG-19789 [ee3fac9](https://github.com/answerbook/vector/commit/ee3fac91454b692f4c4791b6a1d37a208720a068) - GitHub [LOG-19789](https://logdna.atlassian.net/browse/LOG-19789)

# [3.20.0](https://github.com/answerbook/vector/compare/v3.19.0...v3.20.0) (2024-04-29)


### Features

* **edge**: add support for tap filtering in edge [1797324](https://github.com/answerbook/vector/commit/179732441c7eed7442d6e4ac33c624bf7e941641) - Mike Del Tito [LOG-19757](https://logdna.atlassian.net/browse/LOG-19757)


### Miscellaneous

* Merge pull request #452 from answerbook/mdeltito/LOG-19757 [248a767](https://github.com/answerbook/vector/commit/248a767c8a174fe9935d3c789bc7e62859dd65c6) - GitHub [LOG-19757](https://logdna.atlassian.net/browse/LOG-19757)

# [3.19.0](https://github.com/answerbook/vector/compare/v3.18.0...v3.19.0) (2024-04-26)


### Features

* **tap**: support VRL for tap event filtering [540f698](https://github.com/answerbook/vector/commit/540f698818ac4056f0f4b14d55296afe0edbd748) - Mike Del Tito [LOG-19753](https://logdna.atlassian.net/browse/LOG-19753)


### Miscellaneous

* Merge pull request #451 from answerbook/mdeltito/LOG-19753 [f1cc535](https://github.com/answerbook/vector/commit/f1cc53541ae7a03090b5b5056007050dd24e3302) - GitHub [LOG-19753](https://logdna.atlassian.net/browse/LOG-19753)

# [3.18.0](https://github.com/answerbook/vector/compare/v3.17.1...v3.18.0) (2024-04-24)


### Bug Fixes

* **opentelemetry**: ensure timestamps are handled consistently [baea6f6](https://github.com/answerbook/vector/commit/baea6f6d4abc75fbace0dfd0f279e2629fcaa36b) - Mike Del Tito [LOG-19371](https://logdna.atlassian.net/browse/LOG-19371)
* **opentelemetry**: include resource schema_url in decoder [c9e17ca](https://github.com/answerbook/vector/commit/c9e17ca5633ea7283fd9c3f53a0638c60d6d0fdd) - Mike Del Tito [LOG-19371](https://logdna.atlassian.net/browse/LOG-19371)
* **opentelemetry**: remove hashmap from traces store [8f02423](https://github.com/answerbook/vector/commit/8f024237e03151e37f43067c898a55dfae584107) - Mike Del Tito [log-19371](https://logdna.atlassian.net/browse/log-19371)
* **opentelemetry**: rename fields for consistency [86b98dc](https://github.com/answerbook/vector/commit/86b98dce311f30d77787ebc8725edcc51fd78c92) - Mike Del Tito [LOG-19371](https://logdna.atlassian.net/browse/LOG-19371)


### Code Refactoring

* consolidate handling of otel type conversion from events [722893a](https://github.com/answerbook/vector/commit/722893ae028cb520292bd7a61b81d779e3a97c39) - Mike Del Tito [LOG-19371](https://logdna.atlassian.net/browse/LOG-19371)


### Features

* **opentelemetry**: add traces support for the otel sink [0b12104](https://github.com/answerbook/vector/commit/0b12104951f20ebb09e2969400b566a2b9ed68b0) - Mike Del Tito [LOG-19371](https://logdna.atlassian.net/browse/LOG-19371)


### Miscellaneous

* Merge pull request #450 from answerbook/mdeltito/LOG-19371 [ebe48c2](https://github.com/answerbook/vector/commit/ebe48c2398aae214d5ef3314a489e6feba4bc126) - GitHub [LOG-19371](https://logdna.atlassian.net/browse/LOG-19371)

## [3.17.1](https://github.com/answerbook/vector/compare/v3.17.0...v3.17.1) (2024-04-15)


### Bug Fixes

* **kafka source**: Fix vector log with incorrect pipeline ID (#449) [a3212f4](https://github.com/answerbook/vector/commit/a3212f44cfa59ca54a21ba5b99dbf8dcedba661b) - GitHub [LOG-19224](https://logdna.atlassian.net/browse/LOG-19224)

# [3.17.0](https://github.com/answerbook/vector/compare/v3.16.2...v3.17.0) (2024-04-15)


### Features

* Increase the rockdb persistence ttl [09cd6e0](https://github.com/answerbook/vector/commit/09cd6e020c51d55e6c4d5c4f0ea77381f5b30b2a) - Dan Hable [LOG-18683](https://logdna.atlassian.net/browse/LOG-18683)

## [3.16.2](https://github.com/answerbook/vector/compare/v3.16.1...v3.16.2) (2024-04-15)


### Bug Fixes

* **ci**: Separate release commit and image publishing [4217922](https://github.com/answerbook/vector/commit/42179221076e6214fdc2418f3b3b96b658e2b8f8) - Darin Spivey [LOG-19686](https://logdna.atlassian.net/browse/LOG-19686)

## [3.16.1](https://github.com/answerbook/vector/compare/v3.16.0...v3.16.1) (2024-04-11)


### Bug Fixes

* **ci**: Do no run tests on a release commit [8ac93a9](https://github.com/answerbook/vector/commit/8ac93a9c8bb5f928be5310b9a1c2d9720454fb7a) - Darin Spivey [LOG-19679](https://logdna.atlassian.net/browse/LOG-19679)

# [3.16.0](https://github.com/answerbook/vector/compare/v3.15.1...v3.16.0) (2024-04-10)


### Features

* **sources**: Update `exec` source to execute command or VRL [38622aa](https://github.com/answerbook/vector/commit/38622aa1fb795fa66f955d08fe0b2685c61b6516) - Darin Spivey [LOG-19574](https://logdna.atlassian.net/browse/LOG-19574)

## [3.15.1](https://github.com/answerbook/vector/compare/v3.15.0...v3.15.1) (2024-04-09)


### Bug Fixes

* **classification**: Use the correct SYSLOG pattern order [f5d6b34](https://github.com/answerbook/vector/commit/f5d6b347bb3aa397c29d1773f21ee92a21a9c9dc) - Darin Spivey [LOG-19650](https://logdna.atlassian.net/browse/LOG-19650)

# [3.15.0](https://github.com/answerbook/vector/compare/v3.14.4...v3.15.0) (2024-04-08)


### Features

* **sink**: OTLP sink - Log implementation [df954e0](https://github.com/answerbook/vector/commit/df954e01b89298876f9bb03a76878fe74baba51c) - Sergey Opria [LOG-19370](https://logdna.atlassian.net/browse/LOG-19370)


### Miscellaneous

* Merge pull request #436 from answerbook/sopria/LOG-19370 [0f6c03d](https://github.com/answerbook/vector/commit/0f6c03dcf63dbe20b9e6f9244dcaa19e979a5f0c) - GitHub [LOG-19370](https://logdna.atlassian.net/browse/LOG-19370)

## [3.14.4](https://github.com/answerbook/vector/compare/v3.14.3...v3.14.4) (2024-04-08)


### Bug Fixes

* **classifier**: avoid reallocating grok pattern names/strings [379d51f](https://github.com/answerbook/vector/commit/379d51fe4c04ef7d6b52f7a21c0934606b57695f) - Mike Del Tito [LOG-19646](https://logdna.atlassian.net/browse/LOG-19646)


### Miscellaneous

* Merge pull request #444 from answerbook/feature/LOG-19646 [caef31c](https://github.com/answerbook/vector/commit/caef31cbdac83693e8e11d31b75f3fa399b19b3b) - GitHub [LOG-19646](https://logdna.atlassian.net/browse/LOG-19646)

## [3.14.3](https://github.com/answerbook/vector/compare/v3.14.2...v3.14.3) (2024-04-08)


### Bug Fixes

* **ci**: Make all vector dev volumnes unique [ee0749c](https://github.com/answerbook/vector/commit/ee0749c296fdeef2220b889b6bce8ec74771f040) - Darin Spivey [LOG-19643](https://logdna.atlassian.net/browse/LOG-19643)

## [3.14.2](https://github.com/answerbook/vector/compare/v3.14.1...v3.14.2) (2024-04-05)


### Chores

* reenable `profiling` feature for jemalloc [6e3e3d5](https://github.com/answerbook/vector/commit/6e3e3d57f9c67e4a4683cba22cf838e0f33588d5) - Mike Del Tito [LOG-19647](https://logdna.atlassian.net/browse/LOG-19647)


### Miscellaneous

* Merge pull request #442 from answerbook/mdeltito/LOG-19647 [10e26f6](https://github.com/answerbook/vector/commit/10e26f6e4cb0d7fd311f4c3ab0e432f208903414) - GitHub [LOG-19647](https://logdna.atlassian.net/browse/LOG-19647)

## [3.14.1](https://github.com/answerbook/vector/compare/v3.14.0...v3.14.1) (2024-04-03)


### Bug Fixes

* **ci**: Clean disk space after ci runs [ci skip] [7963582](https://github.com/answerbook/vector/commit/796358215aaa5eecd597addbd39a6e5fad998a13) - Darin Spivey [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590)


### Miscellaneous

* Merge pull request #439 from answerbook/darinspivey/LOG-19590 [b658874](https://github.com/answerbook/vector/commit/b658874d925621326410d26a187ef3087efa24a1) - GitHub [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590)

# [3.14.0](https://github.com/answerbook/vector/compare/v3.13.3...v3.14.0) (2024-04-02)


### Bug Fixes

* **throttle**: use ms for window config [f898c1f](https://github.com/answerbook/vector/commit/f898c1ff8b873a63937e061d722a9499cd8da9b4) - Mike Del Tito [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)


### Features

* **throttle**: add state persistence [ad882f2](https://github.com/answerbook/vector/commit/ad882f26c0f88b660b75b6be9f9c443fac554620) - Mike Del Tito [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)
* **throttle**: reimplement throttle transform with basic rate limiting [066cac3](https://github.com/answerbook/vector/commit/066cac3957882804c6174f11924e1b2ea4a0b91f) - Mike Del Tito [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)


### Miscellaneous

* Merge pull request #438 from answerbook/mdeltito/LOG-19565 [f8854d6](https://github.com/answerbook/vector/commit/f8854d6c07b4cc173531acb70ced6a2aea424b17) - GitHub [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)

## [3.13.3](https://github.com/answerbook/vector/compare/v3.13.2...v3.13.3) (2024-03-27)


### Chores

* **build**: Use open source vrl fork [347e20f](https://github.com/answerbook/vector/commit/347e20f28b0480c707868f3cc8713b7e7d534ade) - Dan Hable [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155)


### Miscellaneous

* Merge pull request #6 from mezmo/darinspivey/answerbook_sync [bd66904](https://github.com/answerbook/vector/commit/bd66904d93efc261909215b5c3141cf21540fcf4) - GitHub
* Merge remote-tracking branch 'answerbook/master' into darinspivey/answerbook_sync [98280d6](https://github.com/answerbook/vector/commit/98280d69bfc94fd4e1261841e7eeaca13df1a2e2) - Darin Spivey
* Merge pull request #5 from mezmo/holmberg/LOG-19506 [35fed6f](https://github.com/answerbook/vector/commit/35fed6fd25638e558f15404fa75b1a818caa6247) - GitHub [LOG-19506](https://logdna.atlassian.net/browse/LOG-19506)
* Merge pull request #3 from mezmo/edge1 [a049b0c](https://github.com/answerbook/vector/commit/a049b0c62ba1705b958fb56d45abc3c1b970d0a3) - GitHub
* Merge branch 'master' into edge1 [596ba22](https://github.com/answerbook/vector/commit/596ba22be5bf186e88ea8e1798d1b6587dec8ebe) - GitHub
* Merge pull request #2 from mezmo/dhable/LOG-19155 [7ddc8c2](https://github.com/answerbook/vector/commit/7ddc8c2f5fb8c29a10610522d533b36ac27e7087) - GitHub [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155)

## [3.13.2](https://github.com/answerbook/vector/compare/v3.13.1...v3.13.2) (2024-03-26)


### Chores

* **deps**: Use public dependencies for everything [894ef6a](https://github.com/answerbook/vector/commit/894ef6ad50af8c3811e7fdefe28b9aeb567aef7d) - Darin Spivey [LOG-19548](https://logdna.atlassian.net/browse/LOG-19548)


### Miscellaneous

* Merge pull request #435 from answerbook/darinspivey/LOG-19548 [f358607](https://github.com/answerbook/vector/commit/f358607651d7ed77c4549f1dfca6faf405385177) - GitHub [LOG-19548](https://logdna.atlassian.net/browse/LOG-19548)

## [3.13.1](https://github.com/answerbook/vector/compare/v3.13.0...v3.13.1) (2024-03-26)


### Bug Fixes

* **classification**: include metadata in event size calculation [5d65d5f](https://github.com/answerbook/vector/commit/5d65d5fbdb85d1f378d15b6a56dbca8fcf138196) - Mike Del Tito [LOG-19550](https://logdna.atlassian.net/browse/LOG-19550)


### Miscellaneous

* Merge pull request #434 from answerbook/mdeltito/LOG-19550 [37dc960](https://github.com/answerbook/vector/commit/37dc960670f927f777711b885a0e2f65c0b6ec7d) - GitHub [LOG-19550](https://logdna.atlassian.net/browse/LOG-19550)

# [3.13.0](https://github.com/answerbook/vector/compare/v3.12.1...v3.13.0) (2024-03-25)


### Bug Fixes

* **postgresql**: Account for storage of `Value::Object` [6d1f32a](https://github.com/answerbook/vector/commit/6d1f32a111eb17cf814133e9dd049d7cff55c501) - Darin Spivey [LOG-19497](https://logdna.atlassian.net/browse/LOG-19497)


### Features

* **mezmo**: `user_log` acceps optional `captured_data` parameter [e623edd](https://github.com/answerbook/vector/commit/e623edd26d5b1a24210b32644835bf3da603f1bc) - Darin Spivey [LOG-19497](https://logdna.atlassian.net/browse/LOG-19497)


### Miscellaneous

* Merge pull request #433 from answerbook/feature/LOG-19497 [f122d43](https://github.com/answerbook/vector/commit/f122d4371f5bb6a37d680c8d29e38a8b0bd094ee) - GitHub [LOG-19497](https://logdna.atlassian.net/browse/LOG-19497)

## [3.12.1](https://github.com/answerbook/vector/compare/v3.12.0...v3.12.1) (2024-03-19)


### Bug Fixes

* **profiling**: only track profiling annotations from classification [c3106a9](https://github.com/answerbook/vector/commit/c3106a903c40ac61a957c97dae67ae092473b51a) - Mike Del Tito [LOG-19502](https://logdna.atlassian.net/browse/LOG-19502)


### Miscellaneous

* Merge pull request #431 from answerbook/mdeltito/LOG-19502 [c267715](https://github.com/answerbook/vector/commit/c26771560d8e56d2c6472023a7804f4c87d9befc) - GitHub [LOG-19502](https://logdna.atlassian.net/browse/LOG-19502)

# [3.12.0](https://github.com/answerbook/vector/compare/v3.11.1...v3.12.0) (2024-03-13)


### Features

* **s3 sink**: add recursive directory consolidation [301ac96](https://github.com/answerbook/vector/commit/301ac96b00170af05aab9f3bfd4db1a0b4a896c8) - dominic-mcallister-logdna [LOG-19448](https://logdna.atlassian.net/browse/LOG-19448)


### Miscellaneous

* Merge pull request #428 from answerbook/dominic/LOG-19448 [f43e028](https://github.com/answerbook/vector/commit/f43e028b0e500a84997afb60ff0026dc38ba0fb1) - GitHub [LOG-19448](https://logdna.atlassian.net/browse/LOG-19448)

## [3.11.1](https://github.com/answerbook/vector/compare/v3.11.0...v3.11.1) (2024-03-12)


### Bug Fixes

* **classification**: `SYSLOGLINE` is having false positive matches [e908658](https://github.com/answerbook/vector/commit/e9086582b383f9a25178a6215333521f14ae66f2) - Darin Spivey [LOG-19416](https://logdna.atlassian.net/browse/LOG-19416)


### Miscellaneous

* Merge pull request #429 from answerbook/darinspivey/LOG-19416 [c129f60](https://github.com/answerbook/vector/commit/c129f605b54128ae46eb40ec682ac4920bee5cb5) - GitHub [LOG-19416](https://logdna.atlassian.net/browse/LOG-19416)

# [3.11.0](https://github.com/answerbook/vector/compare/v3.10.0...v3.11.0) (2024-03-01)


### Chores

* disable integration tests on pull requests [23251f2](https://github.com/answerbook/vector/commit/23251f290ac02e3ec8c34fe93082e6fe1a74041a) - Mike Del Tito [LOG-19433](https://logdna.atlassian.net/browse/LOG-19433)


### Features

* **classification**: identify and annotate json string messages [a837636](https://github.com/answerbook/vector/commit/a837636de79d9339b03d16c4e95e906f4cdca587) - Mike Del Tito [LOG-19433](https://logdna.atlassian.net/browse/LOG-19433)


### Miscellaneous

* Merge pull request #427 from answerbook/mdeltito/LOG-19433 [6ef9311](https://github.com/answerbook/vector/commit/6ef931137b5ae2600240c39d0fa111664f047d4a) - GitHub [LOG-19433](https://logdna.atlassian.net/browse/LOG-19433)

# [3.10.0](https://github.com/answerbook/vector/compare/v3.9.0...v3.10.0) (2024-02-29)


### Features

* **azure blob**: add tagging and file consolidation [d933d28](https://github.com/answerbook/vector/commit/d933d2800edfde28e2eb19d6c27a84b5431238e4) - dominic-mcallister-logdna [LOG-19336](https://logdna.atlassian.net/browse/LOG-19336) [LOG-19337](https://logdna.atlassian.net/browse/LOG-19337)


### Miscellaneous

* Merge pull request #426 from answerbook/dominic/LOG-19232 [a5acdec](https://github.com/answerbook/vector/commit/a5acdec594f76de949900f6f31dcd1f300f6f2dd) - GitHub [LOG-19232](https://logdna.atlassian.net/browse/LOG-19232)

# [3.9.0](https://github.com/answerbook/vector/compare/v3.8.1...v3.9.0) (2024-02-26)


### Features

* **aggregate-v2**: Track and expose metadata on event [e88c421](https://github.com/answerbook/vector/commit/e88c421ba9818b1ab7678be413064dfdc2d36915) - Dan Hable [LOG-19291](https://logdna.atlassian.net/browse/LOG-19291)

## [3.8.1](https://github.com/answerbook/vector/compare/v3.8.0...v3.8.1) (2024-02-23)


### Bug Fixes

* **sinks**: Enable user error logging for azure blob (#424) [fc3cd8a](https://github.com/answerbook/vector/commit/fc3cd8a84fe2893fe01e203ee4ece48fb5305047) - GitHub [LOG-19360](https://logdna.atlassian.net/browse/LOG-19360)


### Code Refactoring

* **sinks**: Restore healthcheck and response user error logging (#422) [4326471](https://github.com/answerbook/vector/commit/4326471d8ee7a406fe52f8c450c1d2ba8eaaa61b) - GitHub [LOG-19146](https://logdna.atlassian.net/browse/LOG-19146)

# [3.8.0](https://github.com/answerbook/vector/compare/v3.7.4...v3.8.0) (2024-02-20)


### Features

* spread vector nodes using topo spread constraints [df7f3f6](https://github.com/answerbook/vector/commit/df7f3f6cf9481766f53ab7cb4325693ce84825d4) - Adam Holmberg [LOG-18832](https://logdna.atlassian.net/browse/LOG-18832)


### Miscellaneous

* Merge pull request #421 from answerbook/holmberg/LOG-18832 [6aa2489](https://github.com/answerbook/vector/commit/6aa2489a43843266e54881fa869456fe4f46b0d2) - GitHub [LOG-18832](https://logdna.atlassian.net/browse/LOG-18832)

## [3.7.4](https://github.com/answerbook/vector/compare/v3.7.3...v3.7.4) (2024-02-14)


### Bug Fixes

* **mezmo-sink**: remove ARC, replace sleep timer with manual check [8b74d7f](https://github.com/answerbook/vector/commit/8b74d7f2272ddf561f20d4b93656d27ddc4ece4b) - Mike Del Tito [LOG-19184](https://logdna.atlassian.net/browse/LOG-19184)


### Miscellaneous

* Merge pull request #420 from answerbook/feature/LOG-19184 [273f6a5](https://github.com/answerbook/vector/commit/273f6a5f19585db8c1dfde9fe6684b133f2ad917) - GitHub [LOG-19184](https://logdna.atlassian.net/browse/LOG-19184)

## [3.7.3](https://github.com/answerbook/vector/compare/v3.7.2...v3.7.3) (2024-02-08)


### Bug Fixes

* **graphql**: Include `metadata` in remote task execution [2e13c69](https://github.com/answerbook/vector/commit/2e13c69d20ed4c25dcb786b1562a90e5a65f9517) - Darin Spivey [LOG-19261](https://logdna.atlassian.net/browse/LOG-19261)


### Miscellaneous

* Merge pull request #419 from answerbook/darinspivey/LOG-19261 [4301694](https://github.com/answerbook/vector/commit/43016946c91a3a57f8b6a84e14e106712a347291) - GitHub [LOG-19261](https://logdna.atlassian.net/browse/LOG-19261)

## [3.7.2](https://github.com/answerbook/vector/compare/v3.7.1...v3.7.2) (2024-02-08)


### Bug Fixes

* **ci**: Speed improvements [99a767f](https://github.com/answerbook/vector/commit/99a767f1b642b3d26091e2c46530d3732e244e19) - Darin Spivey [LOG-19242](https://logdna.atlassian.net/browse/LOG-19242)
* **http_server**: Store request metadata in `PathPrefix::Metadata` [9875ad9](https://github.com/answerbook/vector/commit/9875ad9f438216b570f0b29514fa91ff52d4d153) - Darin Spivey [LOG-19242](https://logdna.atlassian.net/browse/LOG-19242)


### Miscellaneous

* Merge pull request #415 from answerbook/darinspivey/LOG-19242 [1b99c75](https://github.com/answerbook/vector/commit/1b99c75355d2044c82aacf35995c0e6af6857fc1) - GitHub [LOG-19242](https://logdna.atlassian.net/browse/LOG-19242)

## [3.7.1](https://github.com/answerbook/vector/compare/v3.7.0...v3.7.1) (2024-02-07)


### Tests

* **aggregate-v2**: Supporting tumbling window config [0d12d6f](https://github.com/answerbook/vector/commit/0d12d6f936810282146f8d8a8e9d397b6ec927e5) - Dan Hable [LOG-18963](https://logdna.atlassian.net/browse/LOG-18963)

# [3.7.0](https://github.com/answerbook/vector/compare/v3.6.0...v3.7.0) (2024-02-07)


### Chores

* refactoring MezmoAggregateV2 [9cb5dbc](https://github.com/answerbook/vector/commit/9cb5dbc9ce8c98f1762bec8d2848aec3c75300fa) - Dan Hable [LOG-19116](https://logdna.atlassian.net/browse/LOG-19116)


### Features

* Expose prior aggregate value in flush condition [aff984c](https://github.com/answerbook/vector/commit/aff984cf044e614f3f074c97356977f3fcde8e1b) - Dan Hable [LOG-19116](https://logdna.atlassian.net/browse/LOG-19116)

# [3.6.0](https://github.com/answerbook/vector/compare/v3.5.2...v3.6.0) (2024-02-06)


### Features

* **mezmo-sink**: include _originating_user_agent [d897521](https://github.com/answerbook/vector/commit/d89752131f870c04974c17a58bcd360eb674157e) - dominic-mcallister-logdna [LOG-19196](https://logdna.atlassian.net/browse/LOG-19196)


### Miscellaneous

* Merge pull request #413 from answerbook/dominic/LOG-19196 [78c0c8d](https://github.com/answerbook/vector/commit/78c0c8d5483fbf5ad0d3de158d90ec4ed40e519a) - GitHub [LOG-19196](https://logdna.atlassian.net/browse/LOG-19196)

## [3.5.2](https://github.com/answerbook/vector/compare/v3.5.1...v3.5.2) (2024-02-05)


### Chores

* **deployment**: added reduce threshold limits into the deployment config [eb244bc](https://github.com/answerbook/vector/commit/eb244bc6e0c80a595bc8dd6f125500831d8f738c) - Sergey Opria [LOG-19194](https://logdna.atlassian.net/browse/LOG-19194)


### Miscellaneous

* Merge pull request #414 from answerbook/sopria/LOG-19194 [bcd3d4b](https://github.com/answerbook/vector/commit/bcd3d4bdb66ee24eace89cd5dd3420d77dc27e26) - GitHub [LOG-19194](https://logdna.atlassian.net/browse/LOG-19194)

## [3.5.1](https://github.com/answerbook/vector/compare/v3.5.0...v3.5.1) (2024-02-01)


### Chores

* **version**: bump vrl@0.14.0 [21ee8f0](https://github.com/answerbook/vector/commit/21ee8f0613d89e6e31ba8730f038240788cdcede) - dominic-mcallister-logdna [LOG-18911](https://logdna.atlassian.net/browse/LOG-18911)


### Miscellaneous

* Merge pull request #411 from answerbook/dominic/LOG-18911 [7e4e670](https://github.com/answerbook/vector/commit/7e4e6705a8bab8242d06f293c51eb424deb3f7b2) - GitHub [LOG-18911](https://logdna.atlassian.net/browse/LOG-18911)

# [3.5.0](https://github.com/answerbook/vector/compare/v3.4.5...v3.5.0) (2024-01-31)


### Code Refactoring

* rename `sliding_aggregate` to `mezmo_aggregate_v2` [0485391](https://github.com/answerbook/vector/commit/0485391488b0869e2930e09f342b6f03481007b9) - Mike Del Tito [LOG-19134](https://logdna.atlassian.net/browse/LOG-19134)


### Features

* **persistence**: add jitter to state flush [db3cae3](https://github.com/answerbook/vector/commit/db3cae3556d74ff05e1f10266ef208505e59ad48) - Mike Del Tito [LOG-19171](https://logdna.atlassian.net/browse/LOG-19171)


### Miscellaneous

* Merge pull request #410 from answerbook/mdeltito/LOG-19134 [0c71dd0](https://github.com/answerbook/vector/commit/0c71dd068c710ecf722d180b0c6d1c0be6bdd86c) - GitHub [LOG-19134](https://logdna.atlassian.net/browse/LOG-19134)

## [3.4.5](https://github.com/answerbook/vector/compare/v3.4.4...v3.4.5) (2024-01-30)


### Chores

* **tests**: Re-enable splunk tests in Github Actions [00a852a](https://github.com/answerbook/vector/commit/00a852a78f7b40e39047ff592bce16afca9e7708) - Darin Spivey [LOG-18162](https://logdna.atlassian.net/browse/LOG-18162)


### Miscellaneous

* Merge pull request #409 from answerbook/darinspivey/LOG-18162 [0ae8b1f](https://github.com/answerbook/vector/commit/0ae8b1ff78fa3b51ad3364a7d41a323e9988042b) - GitHub [LOG-18162](https://logdna.atlassian.net/browse/LOG-18162)

## [3.4.4](https://github.com/answerbook/vector/compare/v3.4.3...v3.4.4) (2024-01-29)


### Chores

* ignore paths from integration suite run [0b96fa8](https://github.com/answerbook/vector/commit/0b96fa8a15df0aed53600697cd7d069428bf10c3) - Mike Del Tito


### Miscellaneous

* Merge pull request #406 from answerbook/mdeltito/ignore-deployment-changes-actions [8fa9653](https://github.com/answerbook/vector/commit/8fa965370be29429ce424b10834a306e3e351dd7) - GitHub

## [3.4.3](https://github.com/answerbook/vector/compare/v3.4.2...v3.4.3) (2024-01-29)


### Chores

* **build**: Use open source vrl fork [8c68704](https://github.com/answerbook/vector/commit/8c68704487ee339c816757a500c03c50bb280f43) - Dan Hable [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155)

## [3.4.2](https://github.com/answerbook/vector/compare/v3.4.1...v3.4.2) (2024-01-29)


### Bug Fixes

* **persistence**: define pvc per-partition [f4f2b09](https://github.com/answerbook/vector/commit/f4f2b0981824e917ada2059279e75c27ce965e20) - Mike Del Tito [LOG-19044](https://logdna.atlassian.net/browse/LOG-19044)


### Miscellaneous

* Merge pull request #407 from answerbook/mdeltito/LOG-19044-pvc-by-partition [21dc87a](https://github.com/answerbook/vector/commit/21dc87ae483685e77fe98bf75e74bdacbb9fd38d) - GitHub [LOG-19044](https://logdna.atlassian.net/browse/LOG-19044)

## [3.4.1](https://github.com/answerbook/vector/compare/v3.4.0...v3.4.1) (2024-01-26)


### Bug Fixes

* **deployment**: adjust resource vars and add namespace [8f8aff6](https://github.com/answerbook/vector/commit/8f8aff6afdef52a7111fb52ac290ecfc72e12290) - Mike Del Tito [LOG-19044](https://logdna.atlassian.net/browse/LOG-19044)


### Miscellaneous

* Merge pull request #405 from answerbook/mdeltito/LOG-19044-fix [c5b3474](https://github.com/answerbook/vector/commit/c5b3474fbdb4fead2b90487e77fc02943bb73c53) - GitHub [LOG-19044](https://logdna.atlassian.net/browse/LOG-19044)

# [3.4.0](https://github.com/answerbook/vector/compare/v3.3.0...v3.4.0) (2024-01-26)


### Bug Fixes

* **persistence**: include pod name in db directory [a23107a](https://github.com/answerbook/vector/commit/a23107a7c1908ef84e8df12ebdf786d9ebcea011) - Mike Del Tito [LOG-19044](https://logdna.atlassian.net/browse/LOG-19044)


### Features

* **persistence**: add pvc/mount for component state storage [ff5929f](https://github.com/answerbook/vector/commit/ff5929f28cd34a7a7755e9678f2fd0ffd6a86ed7) - Mike Del Tito [LOG-19044](https://logdna.atlassian.net/browse/LOG-19044)


### Miscellaneous

* Merge pull request #404 from answerbook/mdeltito/LOG-19044 [fdeb534](https://github.com/answerbook/vector/commit/fdeb534bc0a0e86a9ccc40b311f74c4739ac5f6e) - GitHub [LOG-19044](https://logdna.atlassian.net/browse/LOG-19044)

# [3.3.0](https://github.com/answerbook/vector/compare/v3.2.0...v3.3.0) (2024-01-26)


### Bug Fixes

* **build**: add libclang-dev dependency to the environment [cfa0321](https://github.com/answerbook/vector/commit/cfa032177b612fbff6e739975c7489d8cc1ce5ac) - Mike Del Tito [LOG-18959](https://logdna.atlassian.net/browse/LOG-18959)


### Features

* **persistence**: define persistence trait/impl and wire into aggregate [85e5d1a](https://github.com/answerbook/vector/commit/85e5d1a23fa90825f0ce941ac197032a8d31b84b) - Mike Del Tito [LOG-18959](https://logdna.atlassian.net/browse/LOG-18959)


### Miscellaneous

* Merge pull request #402 from answerbook/mdeltito/LOG-18959 [a1ec760](https://github.com/answerbook/vector/commit/a1ec760304e73c022df565fae1be096041b738bf) - GitHub [LOG-18959](https://logdna.atlassian.net/browse/LOG-18959)


### Tests

* allow conditional compiling of tests under ci [67d6f7d](https://github.com/answerbook/vector/commit/67d6f7d04f22f4fdd616b52e2ebb53b0306a1557) - Mike Del Tito [LOG-18959](https://logdna.atlassian.net/browse/LOG-18959)

# [3.2.0](https://github.com/answerbook/vector/compare/v3.1.3...v3.2.0) (2024-01-24)


### Bug Fixes

* **sources**: `http_server` is not saving query params as key/val [733f3a9](https://github.com/answerbook/vector/commit/733f3a9929cf17ffece9d19aa3200afc45f3d0de) - Darin Spivey [LOG-19104](https://logdna.atlassian.net/browse/LOG-19104)


### Features

* **http_server source**: add all headers to the namespace metadata (#18922) [3772b19](https://github.com/answerbook/vector/commit/3772b19deca84d7375cd33e75189309bd1d267ed) - Darin Spivey [LOG-19103](https://logdna.atlassian.net/browse/LOG-19103)
* **sources**: `http_server` accepts query parameter wildcards [6627a95](https://github.com/answerbook/vector/commit/6627a95fb27365dc37fc6fc3d3c4fd127f10c219) - Darin Spivey [LOG-19105](https://logdna.atlassian.net/browse/LOG-19105)


### Miscellaneous

* Merge pull request #403 from answerbook/darinspivey/LOG-19103 [896eaed](https://github.com/answerbook/vector/commit/896eaed03cb80c233121e13cd8b541eb8aee7eaf) - GitHub [LOG-19103](https://logdna.atlassian.net/browse/LOG-19103)

## [3.1.3](https://github.com/answerbook/vector/compare/v3.1.2...v3.1.3) (2024-01-23)


### Code Refactoring

* Add memory and window limits to sliding aggregate (#400) [aa050e2](https://github.com/answerbook/vector/commit/aa050e2c94dcb62125710925c8c05becb35c4f00) - GitHub [LOG-18818](https://logdna.atlassian.net/browse/LOG-18818)

## [3.1.2](https://github.com/answerbook/vector/compare/v3.1.1...v3.1.2) (2024-01-23)


### Chores

* bump vrl depey to rev=v0.12.0 [6d08598](https://github.com/answerbook/vector/commit/6d085987e7a5aa0d7e5dc9d1c3cfe1e547fc36b4) - dominic-mcallister-logdna [LOG-18993](https://logdna.atlassian.net/browse/LOG-18993) [LOG-18994](https://logdna.atlassian.net/browse/LOG-18994) [LOG-18999](https://logdna.atlassian.net/browse/LOG-18999)


### Miscellaneous

* Merge pull request #401 from answerbook/dominic/LOGs-18993_18994_18999-add_groks [edc9090](https://github.com/answerbook/vector/commit/edc909055cbb5067f979085d783204925ff08e8f) - GitHub

## [3.1.1](https://github.com/answerbook/vector/compare/v3.1.0...v3.1.1) (2024-01-22)


### Bug Fixes

* **otlp**: replace or remove fields with an empty string [db87922](https://github.com/answerbook/vector/commit/db879220fbba21935ed05d2496f49979624193da) - Sergey Opria [LOG-18908](https://logdna.atlassian.net/browse/LOG-18908)


### Miscellaneous

* Merge pull request #392 from answerbook/sopria/LOG-18908 [a599fdd](https://github.com/answerbook/vector/commit/a599fddc5a8fd8e0189f229a93c0a8b2aaf34faf) - GitHub [LOG-18908](https://logdna.atlassian.net/browse/LOG-18908)

# [3.1.0](https://github.com/answerbook/vector/compare/v3.0.1...v3.1.0) (2024-01-17)


### Features

* **vrl**: Implement Mezmo VRL functions in CLI [53743bf](https://github.com/answerbook/vector/commit/53743bf631767ec85e0bc1ef392b490432b4d3ce) - Dan Hable [LOG-19051](https://logdna.atlassian.net/browse/LOG-19051)

## [3.0.1](https://github.com/answerbook/vector/compare/v3.0.0...v3.0.1) (2024-01-17)


### Bug Fixes

* **s3-consolidation**: file copy failure [753bbc1](https://github.com/answerbook/vector/commit/753bbc1808da14800d0446e7c76261314cc748ed) - dominic-mcallister-logdna [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)


### Miscellaneous

* Merge pull request #398 from answerbook/dominic/LOG-18535_removecopypart [72ebd31](https://github.com/answerbook/vector/commit/72ebd31362ece5ad56b244b95a349ffa2467d549) - GitHub [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)

# [3.0.0](https://github.com/answerbook/vector/compare/v2.1.2...v3.0.0) (2024-01-17)


### Bug Fixes

* **amqp sink**: remove duplicate events (#18932) [a916605](https://github.com/answerbook/vector/commit/a9166056653cc3ed2d7598f40281a70aed78d074) - GitHub
* **amqp sink**: remove unnecessary unwrap & emit event dropped errors (#18923) [26f430c](https://github.com/answerbook/vector/commit/26f430c77138ef2373e86182966d5d5085b68514) - GitHub
* **amqp sink**: remove unused feature flag (#18948) [cb53588](https://github.com/answerbook/vector/commit/cb53588f95a832f9cba60c5cf20b5cbf8375e56c) - GitHub
* **buffers**: apply stricter file permissions to buffer data files when possible (#18895) [cf7298f](https://github.com/answerbook/vector/commit/cf7298f80d09ddf5aff9ff0aec9a6b1ca7f12918) - GitHub
* **clickhouse sink**: fix healthcheck uri (#19067) [7a7b53b](https://github.com/answerbook/vector/commit/7a7b53b1c6958f346648042de84d53f6c7357064) - GitHub
* **codecs**: fix 'ProtobufSerializerConfig' input type (#19264) [d2fea65](https://github.com/answerbook/vector/commit/d2fea6580aaa9a0936f11b9f70fc053676872837) - Jesse Szwedko
* **codecs**: native JSON serialization/deserialization for special f64 values (#18650) [39b9298](https://github.com/answerbook/vector/commit/39b9298a92f6c801b5f3be0d77f1b12bd240d6be) - GitHub
* **config**: Only try default paths if the path is not specified (#18681) [37fc9db](https://github.com/answerbook/vector/commit/37fc9dbb6ae4a19159391ff2d03dd9e9b1fbdd13) - GitHub
* **datadog_agent source, datadog_metrics sink**: handle interval for non-rate series metrics (#18889) [dc9d966](https://github.com/answerbook/vector/commit/dc9d966120f3f518ac1f1a07f4716020b69772eb) - GitHub
* **datadog_agent source**: return 200 on empty object payload (#19093) [ab7983a](https://github.com/answerbook/vector/commit/ab7983a201c7e001317d993f7a291a769af06b38) - Jesse Szwedko
* **datadog_metrics sink**: evaluate series v1 env var at runtime (#19148) [7b292ce](https://github.com/answerbook/vector/commit/7b292cea468bd7894f6f48aedaad11f46ff2a622) - Jesse Szwedko
* **datadog_metrics sink**: improve aggregation performance (#18759) [6a40169](https://github.com/answerbook/vector/commit/6a4016963c17e78d94014482af02118d818e241b) - GitHub
* **datadog_metrics sink**: Revert to using v1 endpoint by default (#19138) [3158f46](https://github.com/answerbook/vector/commit/3158f46d66297ac9a4a406553038d4641ebf7590) - Jesse Szwedko
* **datadog_metrics sink**: the integration tests weren't actually validating anything (#18754) [afc166f](https://github.com/answerbook/vector/commit/afc166f37fd2704e474cec33f31d4f5bc224c94d) - GitHub
* **datadog_traces sink**: improve request size limiting (#18903) [a477d72](https://github.com/answerbook/vector/commit/a477d720e00217603cbd2e81f427f28187641fa0) - GitHub
* **debian platform**: Re-add `conf-files` directive for `cargo-deb` (#18726) [e445721](https://github.com/answerbook/vector/commit/e44572170222dd07803998e1760fce5d7ae4e7f1) - GitHub
* **dev**: Gate config conversion tests (#18698) [c35ae64](https://github.com/answerbook/vector/commit/c35ae64725b2f184761909d003a1c0d56f29b3ed) - GitHub
* **dev**: update environment for website development (#18657) [4327776](https://github.com/answerbook/vector/commit/432777631445eea8c17551763808c7d767472258) - GitHub
* **dnstap source**: support DNSSEC RRSIG record data (#18878) [ed97f0d](https://github.com/answerbook/vector/commit/ed97f0d862d2b8c9e606d587e03fd1467159c608) - GitHub
* **docker source**: do not emit component error for out of order logs (#18649) [93d7af4](https://github.com/answerbook/vector/commit/93d7af46f465693a315b0ee69df5f91329742926) - GitHub
* **http_server source**: panic when http server receives metric events (#18781) [b107ff7](https://github.com/answerbook/vector/commit/b107ff706535a2a374cfb4d418f2c86d98628b3a) - GitHub
* **kafka sink**: Make KafkaService return `Poll::Pending` when producer queue is full (#18770) [a1863e6](https://github.com/answerbook/vector/commit/a1863e65cc22cd83d12c7806ec13baa6f65f8491) - GitHub
* **kafka source**: fix acknowledgement handling during shutdown and rebalance events (#17497) [f2efb1a](https://github.com/answerbook/vector/commit/f2efb1ac53f45621fff3f2ea7628cc1082040b5e) - GitHub
* **loki sink**: update to use the global list of compression algorithms (#19099) [218963a](https://github.com/answerbook/vector/commit/218963a3f8a460cfe8e9c3dd3b3ccabb775745ef) - Jesse Szwedko
* **releasing**: Update cargo-deb (#19009) [88194e7](https://github.com/answerbook/vector/commit/88194e76df9e063236777f1e2166624cbd348d1b) - GitHub
* **releasing**: Update example YAML config data_dir (#18896) [0b27019](https://github.com/answerbook/vector/commit/0b270195ebb1f5f01ea0df644fbfd76a30ca5e3e) - GitHub
* remove gh token call (#19047) [1c864aa](https://github.com/answerbook/vector/commit/1c864aa2753b497392f802dfce194dec5e41803a) - GitHub
* **sources**: emit `ComponentEventsDropped` when source send is cancelled (#18859) [a0e2769](https://github.com/answerbook/vector/commit/a0e2769983daa1cdc62fe1af9135b58f489a2681) - GitHub
* **tls**: for incoming connection alpn negotiation should be done using set_alpn_select_callback (#18843) [8a5b67e](https://github.com/answerbook/vector/commit/8a5b67e452772887d5353ff245d33bd4e2ed19ba) - GitHub


### Chores

* Add SHA256 checksums file to GH releases (#18701) [18f07a0](https://github.com/answerbook/vector/commit/18f07a00efc6843ccabb74153291d25573e39293) - GitHub
* **ci**: Add a summary if the regression workflow is skipped (#18724) [17bd2b1](https://github.com/answerbook/vector/commit/17bd2b1cb60861a9854a419636a74a08ab75bce2) - GitHub
* **ci**: Add a test to assert conf files aren't overwritten (#18728) [3ade682](https://github.com/answerbook/vector/commit/3ade6828c80700e4b9d7517e4d3abd1afba9c87a) - GitHub
* **ci**: add dependabot group for futures (#18954) [c23efce](https://github.com/answerbook/vector/commit/c23efce846ab8aec9ce12e638ba16d11281ec203) - GitHub
* **ci**: Bump aws-actions/amazon-ecr-login from 1 to 2 (#18752) [8e2032c](https://github.com/answerbook/vector/commit/8e2032c407985a2dd77b7550b62d5fcfd04399d1) - GitHub
* **ci**: Bump aws-actions/configure-aws-credentials from 4.0.0 to 4.0.1 (#18771) [c9804f0](https://github.com/answerbook/vector/commit/c9804f0c9e5a0931bbaaffe1270021d9c960fcb8) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.26.1 to 1.27.0 (#18783) [a784018](https://github.com/answerbook/vector/commit/a784018715d700710f76257c5d337a65f8e6e145) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.27.0 to 1.27.1 (#18866) [811b7f7](https://github.com/answerbook/vector/commit/811b7f7fb9874acacd3402f50a7b0b252d9cda99) - GitHub
* **ci**: Bump bufbuild/buf-setup-action from 1.27.1 to 1.27.2 (#18981) [72560b1](https://github.com/answerbook/vector/commit/72560b1291f64dc2b34b36c1507deac9f9a6e650) - GitHub
* **ci**: Bump check-spelling/check-spelling from 0.0.21 to 0.0.22 (#18723) [f98cd5d](https://github.com/answerbook/vector/commit/f98cd5d3a5558b459fce6c3b0afb0babb533b10c) - GitHub
* **ci**: Bump MacOS unit test runners to 13 (#18823) [5bf18df](https://github.com/answerbook/vector/commit/5bf18df5684fc1603e248e48541a74f5d619a4e1) - GitHub
* **ci**: Bump tspascoal/get-user-teams-membership from 2 to 3 (#18808) [4701bb9](https://github.com/answerbook/vector/commit/4701bb96d170d17cf3d611f97faf69c1853cbb71) - GitHub
* **ci**: filter team members from gardener issue comment workflow (#18915) [bf56ac5](https://github.com/answerbook/vector/commit/bf56ac5b98569902b5a58e94b8afecd846545d14) - GitHub
* **ci**: Fix cookie banner style issues (#18745) [7a55e54](https://github.com/answerbook/vector/commit/7a55e5490a3a11bfdc7194380653b385b2bddbde) - GitHub
* **ci**: Remove unusued Dockerfile (#18824) [9d1a676](https://github.com/answerbook/vector/commit/9d1a676626101208fd673a6b413e48e52c6d8626) - GitHub
* **ci**: Revet bump check-spelling/check-spelling from 0.0.21 to 0.0.22 (#18742) [7dce292](https://github.com/answerbook/vector/commit/7dce29248f96e32737d8982ddbef74ecc068ac99) - GitHub
* **ci**: Run deny check nightly instead of on every PR (#18799) [ae117dc](https://github.com/answerbook/vector/commit/ae117dc727284c09f3a8f60a876b14fa05a150bd) - GitHub
* **ci**: temporarily peg greptimedb to `v0.4.0`  to unblock CI (#18838) [0776cc0](https://github.com/answerbook/vector/commit/0776cc0ee18cda5a30deaef51fd2e8191643ce74) - GitHub
* convert test config to yaml (#18856) [8b00214](https://github.com/answerbook/vector/commit/8b002145656b59484eab4db0d18f8d9343cb1a20) - GitHub
* **core**: Add a CLI flag to allow for empty configs (#19021) [df4921b](https://github.com/answerbook/vector/commit/df4921b904a0310d8f9d48bdc456ab513594ebb0) - GitHub
* **core**: add more event metadata to proto (#18816) [2deeba1](https://github.com/answerbook/vector/commit/2deeba11a8312a90059f9120a48d6609aa2bf5c2) - GitHub
* **core**: Refactor `vector-core::stream` into its own package (#18900) [96f4d73](https://github.com/answerbook/vector/commit/96f4d73d3a8614d721bdb27845b7721e8d266bb8) - GitHub
* **core**: Set up internal topology API (#18919) [c9c184e](https://github.com/answerbook/vector/commit/c9c184ea3b6785823c723a818eb2b804b429cc3e) - GitHub
* **datadog_metrics sink**: Set partial Origin Metrics in edge cases (#18677) [3dab239](https://github.com/answerbook/vector/commit/3dab23984a16fc7080ff95cfcbb3de3f4c45cc55) - GitHub
* **datadog_metrics sink**: support and migrate to the `v2` series API endpoint (#18761) [3485f2c](https://github.com/answerbook/vector/commit/3485f2c53617270403317f4c21bb076a8f53eeee) - GitHub
* **datadog**: remove deprecated config options (#18940) [f42751d](https://github.com/answerbook/vector/commit/f42751d84a086a104bb8135f4fc419476070455c) - GitHub
* **deps**: Add more `dependabot` groups (#18719) [d0e605e](https://github.com/answerbook/vector/commit/d0e605ec4192d379c796ec771d3f12c6f8bda0c9) - GitHub
* **deps**: Bump @babel/traverse from 7.17.0 to 7.23.2 in /website (#18852) [85d2f17](https://github.com/answerbook/vector/commit/85d2f17dc0a6dc592220e2d93ea1758bb39afd99) - GitHub
* **deps**: Bump apache-avro from 0.15.0 to 0.16.0 (#18685) [b37ce3c](https://github.com/answerbook/vector/commit/b37ce3cc9314333159f60a8e562a3c56ae32e1a2) - GitHub
* **deps**: Bump async-compression from 0.4.3 to 0.4.4 (#18848) [1a8a8cc](https://github.com/answerbook/vector/commit/1a8a8ccfa91f9fa0eebdb1d40fef4bc0967ffbcc) - GitHub
* **deps**: Bump async-graphql from 5.0.10 to 6.0.9 (#18988) [4a4eb61](https://github.com/answerbook/vector/commit/4a4eb61c345b22ef471729cb79a8f322ef4e0b77) - GitHub
* **deps**: Bump async-nats from 0.32.0 to 0.32.1 (#18735) [047c772](https://github.com/answerbook/vector/commit/047c7729f65bf741c7180ffea6f0af2d6723cb8e) - GitHub
* **deps**: Bump async-trait from 0.1.73 to 0.1.74 (#18849) [8d82257](https://github.com/answerbook/vector/commit/8d82257e433da08daca71dc51c7b835b733e1bff) - GitHub
* **deps**: Bump base64 from 0.21.4 to 0.21.5 (#18907) [e754dee](https://github.com/answerbook/vector/commit/e754dee7a61975636c2e87a608376d954a4878a1) - GitHub
* **deps**: Bump bitmask-enum from 2.2.2 to 2.2.3 (#19057) [7a16ee2](https://github.com/answerbook/vector/commit/7a16ee2c78fb9d5beb89734f9d7b9835fb0df1c7) - GitHub
* **deps**: Bump bstr from 1.6.2 to 1.7.0 (#18810) [91221c6](https://github.com/answerbook/vector/commit/91221c6605526d42a6eeab1746e6938399a9b4a5) - GitHub
* **deps**: Bump cached from 0.45.1 to 0.46.0 (#18660) [331c5a0](https://github.com/answerbook/vector/commit/331c5a09c3d82a9fcf96a703d7a36b51639ccaa1) - GitHub
* **deps**: Bump cached from 0.46.0 to 0.46.1 (#19058) [51c6b57](https://github.com/answerbook/vector/commit/51c6b579059494f667081612eb31cb041dac7a75) - GitHub
* **deps**: Bump cargo_toml from 0.16.3 to 0.17.0 (#18978) [b55e436](https://github.com/answerbook/vector/commit/b55e436205bb08009c8e077206ce08d7fd11eaa8) - GitHub
* **deps**: Bump chrono from 0.4.30 to 0.4.31 (#18583) [052ed98](https://github.com/answerbook/vector/commit/052ed98ad08813f4fdc41c2c86362bb6e5bc86d3) - GitHub
* **deps**: Bump chrono-tz from 0.8.3 to 0.8.4 (#18979) [7f44b4c](https://github.com/answerbook/vector/commit/7f44b4c846638b85ff09c44bce32ac8b0a1066e4) - GitHub
* **deps**: Bump clap from 4.4.5 to 4.4.6 (#18715) [4d98fdf](https://github.com/answerbook/vector/commit/4d98fdfa2131a216283bf73976514efb4de6241a) - GitHub
* **deps**: Bump clap_complete from 4.4.2 to 4.4.3 (#18716) [27b2c93](https://github.com/answerbook/vector/commit/27b2c93523afc25113128db1561684290baa594f) - GitHub
* **deps**: Bump console-subscriber from 0.1.10 to 0.2.0 (#18732) [eda0378](https://github.com/answerbook/vector/commit/eda0378d4b8d65fe2b674e3f96429ffcf325aa04) - GitHub
* **deps**: Bump csv from 1.2.2 to 1.3.0 (#18768) [7cb8b52](https://github.com/answerbook/vector/commit/7cb8b52d80e6067482c85f23603fc75817e4b9df) - GitHub
* **deps**: Bump dd-rust-license-tool to 1.0.2 (#18711) [570bd52](https://github.com/answerbook/vector/commit/570bd52dd6a1ea34b2128ad6bc53e66314db1ccb) - GitHub
* **deps**: Bump dyn-clone from 1.0.14 to 1.0.16 (#19040) [8ba28e0](https://github.com/answerbook/vector/commit/8ba28e007381c22783d0a82abe4e6a312682fdc8) - GitHub
* **deps**: Bump fakedata_generator from 0.2.4 to 0.4.0 (#18910) [7debc60](https://github.com/answerbook/vector/commit/7debc602f7c63e64cfde67f566acca2b1567c4c3) - GitHub
* **deps**: Bump flate2 from 1.0.27 to 1.0.28 (#18850) [b3889bc](https://github.com/answerbook/vector/commit/b3889bcea835a00395928e0743a3c14582dc426d) - GitHub
* **deps**: Bump futures-util from 0.3.28 to 0.3.29 (#18951) [741aec3](https://github.com/answerbook/vector/commit/741aec36702efb80ebb3fa1a58c21a61ff181642) - GitHub
* **deps**: Bump goauth from 0.13.1 to 0.14.0 (#18872) [1913ee5](https://github.com/answerbook/vector/commit/1913ee5cd5260d1246013fe9a99e1a86218b9d48) - GitHub
* **deps**: Bump hashbrown from 0.14.0 to 0.14.1 (#18731) [31d92c2](https://github.com/answerbook/vector/commit/31d92c2746cd2797d8463bef64cfa035ff294481) - GitHub
* **deps**: Bump hashbrown from 0.14.1 to 0.14.2 (#18893) [aebe8db](https://github.com/answerbook/vector/commit/aebe8db2090e56954998de9222154b5fdc43a365) - GitHub
* **deps**: Bump indexmap from 2.0.0 to 2.0.1 (#18705) [65643da](https://github.com/answerbook/vector/commit/65643daf9f2ce3d6cc315283fd15065518de792e) - GitHub
* **deps**: Bump indexmap from 2.0.1 to 2.0.2 (#18737) [f47df40](https://github.com/answerbook/vector/commit/f47df40104aaa60c637bce6ec06a6313de268199) - GitHub
* **deps**: Bump inventory from 0.3.12 to 0.3.13 (#19024) [72eacf5](https://github.com/answerbook/vector/commit/72eacf510c4d4efbc07672b584aeb5af677aa483) - GitHub
* **deps**: Bump lading to 0.19.1 (#18869) [2d7c1bb](https://github.com/answerbook/vector/commit/2d7c1bbea68dea90e552e71d8ba240db35e6115f) - GitHub
* **deps**: Bump libc from 0.2.148 to 0.2.149 (#18800) [4002ef0](https://github.com/answerbook/vector/commit/4002ef0458ac2f28a25d7321409732c702c52bac) - GitHub
* **deps**: Bump libc from 0.2.149 to 0.2.150 (#19059) [611a652](https://github.com/answerbook/vector/commit/611a652cfd5b056cde09bc2100674b4a758ae1f3) - GitHub
* **deps**: Bump lru from 0.11.1 to 0.12.0 (#18767) [c531a3b](https://github.com/answerbook/vector/commit/c531a3b3c7b1bff0f2b44a6a4dcc079ab26d5074) - GitHub
* **deps**: Bump memchr from 2.6.3 to 2.6.4 (#18736) [c7482d0](https://github.com/answerbook/vector/commit/c7482d059e6c590fa719ee23bc55849fd68d605d) - GitHub
* **deps**: Bump memmap2 from 0.7.1 to 0.8.0 (#18659) [ca9e5b4](https://github.com/answerbook/vector/commit/ca9e5b4b5f09d872f0f6738e774b2e39fd847cab) - GitHub
* **deps**: Bump memmap2 from 0.8.0 to 0.9.0 (#18765) [dcbbb9b](https://github.com/answerbook/vector/commit/dcbbb9b13ba97aee074f311f21a4bc42743db0de) - GitHub
* **deps**: Bump mongodb from 2.6.1 to 2.7.0 (#18703) [539a40f](https://github.com/answerbook/vector/commit/539a40f2190ec4e0fe701691b33ade93a1baca15) - GitHub
* **deps**: Bump mongodb from 2.7.0 to 2.7.1 (#19023) [602f630](https://github.com/answerbook/vector/commit/602f630ded590380993fd109fc23b5e7c5cb7e64) - GitHub
* **deps**: Bump num_enum from 0.7.0 to 0.7.1 (#18975) [c9b6d45](https://github.com/answerbook/vector/commit/c9b6d45a6194cbcbf4d33147ce0440f1ca7bc3c8) - GitHub
* **deps**: Bump num-traits from 0.2.16 to 0.2.17 (#18802) [c4fbc25](https://github.com/answerbook/vector/commit/c4fbc2579790dc77b3a7d19915cc5eb186456a70) - GitHub
* **deps**: Bump OpenDAL to v0.41 (#19039) [5655f76](https://github.com/answerbook/vector/commit/5655f7674b27b09e24782d291a9613ff1216c58e) - GitHub
* **deps**: Bump openssl from 0.10.57 to 0.10.58 (#19025) [c4f2d0e](https://github.com/answerbook/vector/commit/c4f2d0e41054729a9427b0861532f7984eb32be4) - GitHub
* **deps**: Bump openssl from 0.10.58 to 0.10.59 (#19054) [ee232f8](https://github.com/answerbook/vector/commit/ee232f8cccc3a7b6835dde6a97cd7c7a4991f1ab) - GitHub
* **deps**: Bump openssl-src from 300.1.5+3.1.3 to 300.1.6+3.1.4 (#18936) [a75a043](https://github.com/answerbook/vector/commit/a75a043523cb3839c4c186719565aeeeceba01cf) - GitHub
* **deps**: Bump ordered-float from 4.1.0 to 4.1.1 (#18818) [99643ca](https://github.com/answerbook/vector/commit/99643ca32faa254a8cb863eb1ff71b3b5b3baf42) - GitHub
* **deps**: Bump postcss from 8.4.6 to 8.4.31 in /website (#18750) [92d2be9](https://github.com/answerbook/vector/commit/92d2be969e5bbc1af7f37abc417e003b03914348) - GitHub
* **deps**: Bump proc-macro2 from 1.0.67 to 1.0.69 (#18803) [bc3b3a2](https://github.com/answerbook/vector/commit/bc3b3a2bd1efded61dd3b2d72ed2b466f52f21bb) - GitHub
* **deps**: Bump proptest from 1.2.0 to 1.3.1 (#18738) [2b15c63](https://github.com/answerbook/vector/commit/2b15c63b5f6a73f918f336b9f6acfaf4a6fd8f52) - GitHub
* **deps**: Bump pulsar from 6.0.1 to 6.1.0 (#19004) [43f5913](https://github.com/answerbook/vector/commit/43f5913153dd0129c67fabac7850cea0bd5ba8e9) - GitHub
* **deps**: Bump quanta from 0.11.1 to 0.12.0 (#18774) [e9d2dae](https://github.com/answerbook/vector/commit/e9d2dae544dacb6e1b3835e6d84dd262bf3b916c) - GitHub
* **deps**: Bump quanta from 0.12.0 to 0.12.1 (#19005) [051de5a](https://github.com/answerbook/vector/commit/051de5afbd29ded9bf9cb321fa21d80d8ed39700) - GitHub
* **deps**: Bump ratatui from 0.23.0 to 0.24.0 (#18908) [8b56a93](https://github.com/answerbook/vector/commit/8b56a933b87b55222bc3dc43c153ffe7c55b6517) - GitHub
* **deps**: Bump regex from 1.10.0 to 1.10.2 (#18858) [434849d](https://github.com/answerbook/vector/commit/434849d54cab5312237c3831d1e9a98e42e5fcce) - GitHub
* **deps**: Bump regex from 1.9.5 to 1.9.6 (#18739) [8e98321](https://github.com/answerbook/vector/commit/8e983219ea91831a7642c5112902940372aded7e) - GitHub
* **deps**: Bump regex from 1.9.6 to 1.10.0 (#18812) [4d02abf](https://github.com/answerbook/vector/commit/4d02abf0656e85ae7910b5eae8fbed356d9a5804) - GitHub
* **deps**: Bump reqwest from 0.11.20 to 0.11.22 (#18760) [bac60ad](https://github.com/answerbook/vector/commit/bac60add0d6f314bc77e1483b4554fcdb754768c) - GitHub
* **deps**: Bump rustix from 0.37.19 to 0.37.25 (#18879) [3ca32b8](https://github.com/answerbook/vector/commit/3ca32b865a49b7d17fe6252f52519cd0765bbc9d) - GitHub
* **deps**: Bump semver from 1.0.18 to 1.0.19 (#18662) [aca7753](https://github.com/answerbook/vector/commit/aca7753a229116fff56305a93f036bc122d75f30) - GitHub
* **deps**: Bump semver from 1.0.19 to 1.0.20 (#18811) [d9aca80](https://github.com/answerbook/vector/commit/d9aca80873f1b5dc7863b483ab3426a5767a723b) - GitHub
* **deps**: Bump serde from 1.0.188 to 1.0.189 (#18834) [0d09898](https://github.com/answerbook/vector/commit/0d09898867bcb489244aaba4e9a257eed9e97437) - GitHub
* **deps**: Bump serde from 1.0.189 to 1.0.190 (#18945) [73afddb](https://github.com/answerbook/vector/commit/73afddb4292b113f659e9dc82d3d32d8ff3bf98d) - GitHub
* **deps**: Bump serde_with from 3.3.0 to 3.4.0 (#18874) [33243ac](https://github.com/answerbook/vector/commit/33243accf958588a0c1494c1a14da11180d5fc5b) - GitHub
* **deps**: Bump serde_yaml from 0.9.25 to 0.9.27 (#18956) [2ee96b1](https://github.com/answerbook/vector/commit/2ee96b17a743c28a0998e66f618d411feaceadf0) - GitHub
* **deps**: Bump serde-toml-merge from 0.3.2 to 0.3.3 (#18804) [abb9101](https://github.com/answerbook/vector/commit/abb9101cc33200bd10ae3c7d1872d72501e93d27) - GitHub
* **deps**: Bump serde-wasm-bindgen from 0.6.0 to 0.6.1 (#18935) [ffed6f7](https://github.com/answerbook/vector/commit/ffed6f70603a1f6702e85a96c45096a98924564a) - GitHub
* **deps**: Bump sha2 from 0.10.7 to 0.10.8 (#18684) [87af0bd](https://github.com/answerbook/vector/commit/87af0bd9c175131cc63a7952fe4ea455554a310f) - GitHub
* **deps**: Bump socket2 from 0.5.4 to 0.5.5 (#18902) [691fdca](https://github.com/answerbook/vector/commit/691fdca246c845942ca264fb5ed2d9d58e7284c7) - GitHub
* **deps**: Bump syn from 2.0.37 to 2.0.38 (#18789) [ec5238e](https://github.com/answerbook/vector/commit/ec5238ee9d868e01864fa9d68c895de8dcceb093) - GitHub
* **deps**: Bump syn from 2.0.38 to 2.0.39 (#19056) [0f0a0b4](https://github.com/answerbook/vector/commit/0f0a0b4e8b67a0ed692992acdeef504f9331b024) - GitHub
* **deps**: Bump tempfile from 3.6.0 to 3.8.0 (#18686) [c0d24b9](https://github.com/answerbook/vector/commit/c0d24b9d46ff09359a8a24a7d49d898bae8f4706) - GitHub
* **deps**: Bump tempfile from 3.8.0 to 3.8.1 (#18957) [40961ed](https://github.com/answerbook/vector/commit/40961edf9ae84f8a42294ca2f9870331a062d2a2) - GitHub
* **deps**: Bump the azure group with 4 updates (#18773) [0e61acc](https://github.com/answerbook/vector/commit/0e61accd512743fc5323ed70947428fd56098640) - GitHub
* **deps**: Bump the azure group with 4 updates (#19052) [53ffbc3](https://github.com/answerbook/vector/commit/53ffbc3a1e3acf2df02918739de6b5d6cef1e900) - GitHub
* **deps**: Bump the clap group with 1 update (#18906) [cddb835](https://github.com/answerbook/vector/commit/cddb83570d8accf5b528254f53502b00385ee268) - GitHub
* **deps**: Bump the clap group with 2 updates (#18925) [78934c2](https://github.com/answerbook/vector/commit/78934c211d085dabe3be4c183b804b05d49303c4) - GitHub
* **deps**: Bump the futures group with 1 update (#18961) [0a5e3db](https://github.com/answerbook/vector/commit/0a5e3dbd9f2c81036266785be2075970dae4142d) - GitHub
* **deps**: Bump the tonic group with 2 updates (#18714) [c95df7c](https://github.com/answerbook/vector/commit/c95df7cf5ec4178ed2e6bdee32c6445d7e193ce4) - GitHub
* **deps**: Bump the zstd group with 1 update (#18826) [96ef9ee](https://github.com/answerbook/vector/commit/96ef9eeed036f8723b4265f0c782e6423cbf6341) - GitHub
* **deps**: Bump thiserror from 1.0.48 to 1.0.49 (#18683) [6b92a83](https://github.com/answerbook/vector/commit/6b92a83b05872b8343df9bdb0aefd6b4ca68169b) - GitHub
* **deps**: Bump thiserror from 1.0.49 to 1.0.50 (#18892) [16df7ea](https://github.com/answerbook/vector/commit/16df7ea298e90e36d947a7cd2546ab15d404653e) - GitHub
* **deps**: Bump tokio from 1.32.0 to 1.33.0 (#18809) [76971bd](https://github.com/answerbook/vector/commit/76971bd9696f3498faa5bae4e586c0beb6e1f5e9) - GitHub
* **deps**: Bump tokio-tungstenite from 0.20.0 to 0.20.1 (#18661) [8abed12](https://github.com/answerbook/vector/commit/8abed1230159a58f0d4462e105c593ee58ce1950) - GitHub
* **deps**: Bump toml from 0.8.0 to 0.8.1 (#18687) [efbe673](https://github.com/answerbook/vector/commit/efbe673780f00d5540b316bf57b25add14f8c449) - GitHub
* **deps**: Bump toml from 0.8.1 to 0.8.2 (#18747) [f9e51e1](https://github.com/answerbook/vector/commit/f9e51e1c895389bae95064a4ad7811196839d8c0) - GitHub
* **deps**: Bump toml from 0.8.2 to 0.8.3 (#18909) [22402ca](https://github.com/answerbook/vector/commit/22402ca6ae4352e7afb32633543f660c20bd4ad4) - GitHub
* **deps**: Bump toml from 0.8.3 to 0.8.4 (#18913) [249330a](https://github.com/answerbook/vector/commit/249330a3a01b132feea17cd3fbd4b0ed22429524) - GitHub
* **deps**: Bump toml from 0.8.4 to 0.8.5 (#18950) [4a525a8](https://github.com/answerbook/vector/commit/4a525a833432ce340e00e174b769ec8ab9a38abb) - GitHub
* **deps**: Bump toml from 0.8.5 to 0.8.6 (#18962) [5e7ae83](https://github.com/answerbook/vector/commit/5e7ae83e7f575ade97e57bf5710cc943041e63d5) - GitHub
* **deps**: Bump tracing-log from 0.1.3 to 0.1.4 (#18914) [e4fd78c](https://github.com/answerbook/vector/commit/e4fd78c78c16f0ef6c859b79615fce7338923ed4) - GitHub [log-0](https://logdna.atlassian.net/browse/log-0) [log-0](https://logdna.atlassian.net/browse/log-0)
* **deps**: Bump tracing-log from 0.1.4 to 0.2.0 (#18941) [30a1e26](https://github.com/answerbook/vector/commit/30a1e2613f63eaeed3e8768ee6423dba568fca4d) - GitHub [log-0](https://logdna.atlassian.net/browse/log-0) [log-0](https://logdna.atlassian.net/browse/log-0)
* **deps**: Bump trust-dns-proto from 0.23.0 to 0.23.1 (#18846) [0568d7a](https://github.com/answerbook/vector/commit/0568d7a50e0a0e8b899edf83333d88a2cd752b04) - GitHub
* **deps**: Bump trust-dns-proto from 0.23.1 to 0.23.2 (#18911) [1eaf8b1](https://github.com/answerbook/vector/commit/1eaf8b1ec759f77461cee072e793ef2457737c0d) - GitHub
* **deps**: Bump uuid from 1.4.1 to 1.5.0 (#18880) [a025caa](https://github.com/answerbook/vector/commit/a025caab1119116b7f0b8c387c696caa71b682bf) - GitHub
* **deps**: Bump warp from 0.3.5 to 0.3.6 (#18704) [decaaeb](https://github.com/answerbook/vector/commit/decaaeb9f5d4e52c483365e526b63b5f7042738d) - GitHub
* **deps**: Bump wasm-bindgen from 0.2.87 to 0.2.88 (#19026) [63bb9e4](https://github.com/answerbook/vector/commit/63bb9e497f4ca9cefdc1221371fa883d0bd1d529) - GitHub
* **deps**: Bump webpki from 0.22.1 to 0.22.2 (#18744) [4295985](https://github.com/answerbook/vector/commit/429598552d2e245b0ae0d9b83aa025c229f0bcbc) - GitHub
* **deps**: Bump wiremock from 0.5.19 to 0.5.21 (#19055) [4622ef6](https://github.com/answerbook/vector/commit/4622ef6a6129d40cb6b69ffec14e31b8d569fc58) - GitHub
* **deps**: Bump zerocopy from 0.7.21 to 0.7.31 (#19394) [74d6cb1](https://github.com/answerbook/vector/commit/74d6cb1effcba4b8f7a7be951907a78f95d39996) - Jesse Szwedko
* **deps**: clean up VRL crate features (#18740) [1452d54](https://github.com/answerbook/vector/commit/1452d54ac21ab69f8a3e0611913f348052be8f1b) - GitHub
* **deps**: Group csv crate updates (#18797) [98aa157](https://github.com/answerbook/vector/commit/98aa157a141a62a7827d29eef91758b39ff3b07e) - GitHub
* **deps**: Remove usages of atty (#18985) [371580c](https://github.com/answerbook/vector/commit/371580c902822ada9f7bcb501c40ec6ddc6bb51b) - GitHub
* **deps**: Update dependencies (#18971) [3ead10f](https://github.com/answerbook/vector/commit/3ead10f8518914fbc9c8877cee5a181ea85c6f3c) - GitHub
* **deps**: Update lading to 0.19.0 (#18861) [dc729f5](https://github.com/answerbook/vector/commit/dc729f580164a96712cef0dc7414ae8daf4ea5d2) - GitHub
* **deps**: Update license-tool.toml webpki version (#18986) [0051ec0](https://github.com/answerbook/vector/commit/0051ec0e72ca1636410b714cd4326770fe0dc929) - GitHub
* **deps**: Update VRL to 0.8.1 (#19011) [f5ea285](https://github.com/answerbook/vector/commit/f5ea28500ebc255630318895e2926029165afb1e) - GitHub
* **dev**: Add `vector-lib` wrapper for three more libs (#18992) [1eb418b](https://github.com/answerbook/vector/commit/1eb418bedd34a2ac8efe9f2980e2263e42e84abe) - GitHub
* **dev**: Add wrapper for `codecs` to `vector-lib` (#18959) [5f30f74](https://github.com/answerbook/vector/commit/5f30f74bbb15edeea13051703ddd16945479c6c8) - GitHub
* **dev**: Add wrapper for `enrichment` to `vector-lib` (#18977) [e61f308](https://github.com/answerbook/vector/commit/e61f308aab380b1af9c6b1e99a57d66181270ee8) - GitHub
* **dev**: Add wrapper for `file-source` in `vector-lib` (#18984) [f44da16](https://github.com/answerbook/vector/commit/f44da167b24d926d47168167e599b79515919a44) - GitHub
* **dev**: Add wrapper for `lookup` in `vector-lib` (#18995) [164f1e9](https://github.com/answerbook/vector/commit/164f1e9b716743569c3ede5ec5936a28a6882142) - GitHub
* **dev**: Add wrapper for `vector-buffers` to `vector-lib` (#18964) [2cef62c](https://github.com/answerbook/vector/commit/2cef62c0ab8f1bfa39c12687cb6b8b36b20ef856) - GitHub
* **dev**: Detail the format of DEPRECATIONS.md file (#19016) [223dd7b](https://github.com/answerbook/vector/commit/223dd7b22967369669734e7c6f476559c7de6533) - GitHub
* **dev**: Move some macros into `vector-core` (#19002) [4f613ce](https://github.com/answerbook/vector/commit/4f613ce3c1c498fd61c8c60cb282a81e83174521) - GitHub
* **dev**: Remove deprecation action item for armv7 RPMs (#19018) [ff7b95f](https://github.com/answerbook/vector/commit/ff7b95fb7192a7f56cb4cc53a020e39d32de2d72) - GitHub
* **dev**: Set up `vector-lib` wrapper crate with `vector-common` (#18927) [00c40d7](https://github.com/answerbook/vector/commit/00c40d7cbe90287a0ee22ed2707576948a89cfff) - GitHub
* **dev**: Wrap `vector-config` in `vector-lib` as `configurable` (#18944) [42beb3f](https://github.com/answerbook/vector/commit/42beb3f099f34c20b890b2d5f9f9ee07dc5697de) - GitHub
* **dev**: Wrap `vector-core` in `vector-lib` (#18934) [270fdfd](https://github.com/answerbook/vector/commit/270fdfd6002f58275610995e232aae241ada4822) - GitHub
* **dev**: Wrap `vector-stream` in `vector-lib` (#18953) [8a02b16](https://github.com/answerbook/vector/commit/8a02b168d4904d0837028b1ef9fc9743d9dee345) - GitHub
* **docs**: Add alpha to traces and beta to metrics in descriptions (#19139) [1b9fb9b](https://github.com/answerbook/vector/commit/1b9fb9b5ac3c99eef2fbe160660401c3797f4254) - Jesse Szwedko
* **docs**: add highlight post for secrets in disk buffers (#18994) [0cc9389](https://github.com/answerbook/vector/commit/0cc9389822bdbf677dd41fb59fc0c074d788f40d) - GitHub
* **docs**: Add spec for `listen` option (#18080) [29e5e22](https://github.com/answerbook/vector/commit/29e5e22aeb3e1f835940904813f54570c66ad085) - GitHub
* **docs**: Replace setup.vector.dev references (#19080) [def235e](https://github.com/answerbook/vector/commit/def235e8e7d67f9461898bd72b55809f6ee09a3a) - Jesse Szwedko
* **docs**: update a few more examples to YAML (#19103) [a59329a](https://github.com/answerbook/vector/commit/a59329aaca2c5d4ae98517fc06fec11728957375) - Jesse Szwedko
* **docs**: update quickstart.md to use YAML (#18796) [0e76fe0](https://github.com/answerbook/vector/commit/0e76fe06ac8c0c0a63d2cf319a4fb3a807b1dec3) - GitHub
* **external docs**: First batch of editorial edits for the Functions doc (#18780) [1da9005](https://github.com/answerbook/vector/commit/1da9005b00c04686ae768d173504b907cc6cb409) - GitHub
* **external docs**: Remove or replace mentions of vector in functions doc (#18679) [7ad4112](https://github.com/answerbook/vector/commit/7ad41129c765c49aff4c24a93e3c8f8a2468102a) - GitHub
* Follow redirects for `sh.vector.dev` (#19000) [9893b86](https://github.com/answerbook/vector/commit/9893b8697e2e3ab30edf85d90fbb0b45244c8f6e) - GitHub
* **gcp_stackdriver_metrics sink**: rewrite to stream based sink (#18749) [92268e4](https://github.com/answerbook/vector/commit/92268e47692c9ab8a45cf44c05df658c4c74c953) - GitHub
* **kubernetes**: Regenerate manifests from 0.27.0 chart (#19001) [5e9dd1d](https://github.com/answerbook/vector/commit/5e9dd1d00aa26e27b82c376a775ac58b2e5b8a50) - GitHub
* **metrics**: improve creation of Origin metadata structures (#18788) [f0adce7](https://github.com/answerbook/vector/commit/f0adce73efe1db4d52235911a3093f661e7f93bc) - GitHub
* Note the version to remove the v1 metrics support from the Datadog Metrics sink (#19017) [be9f229](https://github.com/answerbook/vector/commit/be9f229dd56b483b5223c3da57ac9f690ddc0a13) - GitHub
* **observability, blackhole sink**: Don't report by default (#18963) [3b85b48](https://github.com/answerbook/vector/commit/3b85b48165c58013b4767e5db4620d3b9331b950) - GitHub
* **observability**: deprecate obsolete http metrics (#18972) [f33dce2](https://github.com/answerbook/vector/commit/f33dce27a3743cb37a6e6f7c07889003427b5b54) - GitHub
* **observability**: fix tokio unstable (#18776) [67c4beb](https://github.com/answerbook/vector/commit/67c4beb8fbff4cdb0f988da16c10ff720fb36e05) - GitHub
* **observability**: remove `peer_addr` internal metric tag (#18982) [b9447f6](https://github.com/answerbook/vector/commit/b9447f613b724e52db57fa4eed25aee04f167967) - GitHub
* **observability**: remove deprecated `component_name` metric tag (#18942) [c6f5d2b](https://github.com/answerbook/vector/commit/c6f5d2b62520cb6b9b923e029298c1e794011a3f) - GitHub
* **observability**: remove metrics replaced by component_errors_total (#18965) [17f4ed2](https://github.com/answerbook/vector/commit/17f4ed2eaf1347c2fbb725400a7f706f9f8f464a) - GitHub
* **prometheus_remote_write sink**: remote write sink rewrite (#18676) [5c1707f](https://github.com/answerbook/vector/commit/5c1707f5972ff37d6bcd5782a157afac89efaa3d) - GitHub
* **regression**: Unmark regression tests as erratic now (#19020) [2cdf654](https://github.com/answerbook/vector/commit/2cdf6547b0fa0d6ac867f95e8470b26168e1f7e8) - GitHub
* **releasing, kubernetes**: Update manifests to v0.26.0 of the chart (#18694) [5f4c3ba](https://github.com/answerbook/vector/commit/5f4c3baa0c7656bc162b0be2a336ed6845fd77b9) - GitHub
* **releasing**: Add deprecation note about respositories.timber.io deprecation (#19078) [09df599](https://github.com/answerbook/vector/commit/09df599a655a116b7eb6016a28705165519fa3f9) - Jesse Szwedko
* **releasing**: Add known issue for 0.33.0 debian packaging regression (#18727) [ff745ab](https://github.com/answerbook/vector/commit/ff745abb2a0c46f05f5aa894e42b679f45247087) - GitHub
* **releasing**: Add known issue for Datadog Metrics sink in v0.34.0 (#19122) [cee9d07](https://github.com/answerbook/vector/commit/cee9d071165a6c9b5d7ba59721e2a838f52fa88b) - Jesse Szwedko
* **releasing**: Add known issue for protobuf encoder in v0.34.0 (#19244) [f7c3824](https://github.com/answerbook/vector/commit/f7c3824c1f6830119ac98b8f4791322fb7e24e50) - Jesse Szwedko
* **releasing**: Add upgrade note about TOML breaking change to v0.34.0 (#19120) [dba0ba1](https://github.com/answerbook/vector/commit/dba0ba17a5c7888ddb20fe808422532939c57619) - Jesse Szwedko
* **releasing**: Bump Vector version to v0.34.0 (#18693) [1c2f970](https://github.com/answerbook/vector/commit/1c2f9704e4c4c799e75500b9250a976890e79329) - GitHub
* **releasing**: Fix changelog not for kafka fix in 0.33.1 (#19032) [2501049](https://github.com/answerbook/vector/commit/250104965db6cccbe922d6bc0cd1f73a6177f8c6) - GitHub
* **releasing**: Fix formatting for v0.34.0 release note (#19085) [3569271](https://github.com/answerbook/vector/commit/356927197e86f280a2762cc0a2a4ee610650df8b) - Jesse Szwedko
* **releasing**: Prepare v0.33.0 release [682f0e0](https://github.com/answerbook/vector/commit/682f0e080a85f644d19773f2f15a8cf5cdad1828) - Jesse Szwedko
* **releasing**: Prepare v0.33.1 release [409b69d](https://github.com/answerbook/vector/commit/409b69dd6eb6316e388dfddb445b7271bc7cb841) - Jesse Szwedko
* **releasing**: Prepare v0.34.0 release [c909b66](https://github.com/answerbook/vector/commit/c909b660ca3cf3cc1c6d9cd7880cef8b61b2b426) - Jesse Szwedko
* **releasing**: Prepare v0.34.1 release [86f1c22](https://github.com/answerbook/vector/commit/86f1c22d7f00d7d80210a2704ea9f5061f74ee55) - Jesse Szwedko
* **releasing**: Prepare v0.34.2 release [d685a16](https://github.com/answerbook/vector/commit/d685a16e5e2acf136b91030718c57dc37572d0ff) - Jesse Szwedko
* **releasing**: Typo in v0.33.1 release docs (#18987) [36974a0](https://github.com/answerbook/vector/commit/36974a0d847758a10914ff060c59ea455f67c67d) - GitHub
* Remove @spencergilbert from CODEOWNERS (#18778) [f300c85](https://github.com/answerbook/vector/commit/f300c85817f22bdd0187d11ae81bd50ebde170f5) - GitHub
* Remove armv7 RPM package (#18837) [eca8c76](https://github.com/answerbook/vector/commit/eca8c761308dc28e52d09d1f7f6bf569f76b0eb7) - GitHub
* remove config/vector.toml (#18833) [efb0d1a](https://github.com/answerbook/vector/commit/efb0d1a59074ea6e097918a230d552204161c42a) - GitHub
* **security**: Ignore RUSTSEC-2023-0071 for now (#19263) [e27b7bd](https://github.com/answerbook/vector/commit/e27b7bdd997879e6fcc99b60b6165e2e533adf6e) - Jesse Szwedko
* **security**: Remove legacy OpenSSL provider flags (#19015) [2bba40a](https://github.com/answerbook/vector/commit/2bba40a0baad268335bf725a327fcf20e9f6ec9b) - GitHub
* **sinks**: Update `PartitionBatcher` to use `BatchConfig` (#18792) [4a7d0c3](https://github.com/answerbook/vector/commit/4a7d0c33fa5ffef2b510f78714e687c6e03a6cf1) - GitHub
* Update *-release.md issue templates for vector.dev package release (#18814) [ab8f8d2](https://github.com/answerbook/vector/commit/ab8f8d28e2535340dcd6b146bc6072e6ae124fd7) - GitHub
* **vrl**: Revive old remap tests (#18678) [53cad38](https://github.com/answerbook/vector/commit/53cad38db12ceb11e0394b4d5906f7de541ec7dc) - GitHub
* **website**: Set download page dropdown to latest version (#18758) [23745f2](https://github.com/answerbook/vector/commit/23745f213f18667548c204133bfd09d55b8ff8c5) - GitHub
* **websites**: Setup preview site workflows (#18924) [9d006c7](https://github.com/answerbook/vector/commit/9d006c7e345645051a06affa9130305d85003cbf) - GitHub
* **websites**: Workflow updates (#19036) [b4ca866](https://github.com/answerbook/vector/commit/b4ca866bd6ae30dae4a27c83a2838d50486001ac) - GitHub
* **website**: WEB-4247 | Update references from s3 to setup.vector.dev (#19149) [9e1ad37](https://github.com/answerbook/vector/commit/9e1ad37179e88de5aed5f78176771133b1c8bde7) - Jesse Szwedko
* **website**: WEB-4275 | Update Navigation (#19186) [73a668c](https://github.com/answerbook/vector/commit/73a668c7988e196a74e8a5d4a171dd2e5eddbed3) - Doug Smith
* **website**: Workflow fixes (#19046) [fb63f8e](https://github.com/answerbook/vector/commit/fb63f8e0545332faa3bf4ff00de894c8f851deda) - GitHub


### Continuous Integration

* mark otlp_http_to_blackhole experiment erratic (#18786) [54b54a5](https://github.com/answerbook/vector/commit/54b54a5bd8c8a6ac0f13ee2cb607e2debd0befeb) - GitHub


### Features

* add PR comment trigger for the workload checks workflow (#18839) [11bc5d9](https://github.com/answerbook/vector/commit/11bc5d98b8de9def6f56541285fd08e065c8b09f) - GitHub
* **amqp**: added integration test for TLS (#18813) [3c4ae86](https://github.com/answerbook/vector/commit/3c4ae86ec14fe835947d8f84d8cf977dffb2fa29) - GitHub [LOG-16435](https://logdna.atlassian.net/browse/LOG-16435)
* **ci**: Add Vector workload checks (#18569) [e2b7de0](https://github.com/answerbook/vector/commit/e2b7de07795b1a649ceb6d0e9555034a042d490b) - GitHub
* **codecs**: add support for protobuf encoding (#18598) [737f5c3](https://github.com/answerbook/vector/commit/737f5c36545fa6fc9c71e0ca3c779e96902bf0af) - GitHub
* **docs**: add fallibility examples (#18931) [08b45a5](https://github.com/answerbook/vector/commit/08b45a576bedd302d8dd6e4914f43c873052a998) - GitHub
* **journald source**: Add emit_cursor option (#18882) [74051dc](https://github.com/answerbook/vector/commit/74051dc85388ca3683e5b932653c0e6a4511a702) - GitHub
* **regression**: convert all regression cases configs to YAML (#18825) [1fb0f0d](https://github.com/answerbook/vector/commit/1fb0f0d9404c600baa40c2d3abd05001ca08d1d4) - GitHub
* **timestamp encoding**: add unixtime formats (#18817) [53039e7](https://github.com/answerbook/vector/commit/53039e72d36844e8188a3de2c25344005ac41c2a) - GitHub


### Miscellaneous

* Merge pull request #396 from answerbook/feature/LOG-18978 [2b046bf](https://github.com/answerbook/vector/commit/2b046bf6b2dd28b00d7ff81d61c2f57d1745c002) - GitHub [LOG-18978](https://logdna.atlassian.net/browse/LOG-18978)
* Merge tag 'v0.34.2' into feature/LOG-18978 [a88d5ed](https://github.com/answerbook/vector/commit/a88d5ed0789cc6350b17b1bebe7a2bf17f45aaef) - Darin Spivey [LOG-18978](https://logdna.atlassian.net/browse/LOG-18978) [LOG-18978](https://logdna.atlassian.net/browse/LOG-18978)
* Update RUM domain (#19367) [4acb9de](https://github.com/answerbook/vector/commit/4acb9de4efec3887746eb2189ec39e74491237d3) - Jesse Szwedko
* chore(docs):Add Obs Pipelines to docs (#19201) [8f3f160](https://github.com/answerbook/vector/commit/8f3f160ee1b81cdf99c85671089ac9a8120ee5d9) - Jesse Szwedko
* updating the doc, 2 urls were 404 (#18949) [bf58b06](https://github.com/answerbook/vector/commit/bf58b06baf18fdaaaa059d76bc84d159f6ca24e0) - GitHub
* chore!(config, docs): delete deprecated vector.toml code (#18795) [c8557d0](https://github.com/answerbook/vector/commit/c8557d080dc522f572b8bd3b384ec22941ac5ea8) - GitHub
* [WEB-3464] Adds TrustArc cookie consent banner (#18741) [43428d3](https://github.com/answerbook/vector/commit/43428d3556d7ddf2ce76aab536087907cd5eb8c0) - GitHub
* Add announcement for new repository URLs (#18798) [239cf94](https://github.com/answerbook/vector/commit/239cf942c165f067b4b21b43912ff0dac579db50) - GitHub
* **examples**: Convert config/examples from TOML to YAML (#18832) [6ffb072](https://github.com/answerbook/vector/commit/6ffb072f548fdeaec444de7064d76ebff2fe2f67) - GitHub
* **external docs**: Fix metrics test example  (#18725) [96def01](https://github.com/answerbook/vector/commit/96def01c488bb122acb680618bf152254b6a6ae6) - GitHub
* fix truncate arguments (#19068) [9a5cb51](https://github.com/answerbook/vector/commit/9a5cb519b21e727a7aedec2ebc9d39367e4c7859) - Jesse Szwedko
* **gcp_pubsub source**: Add required fields to documentation examples (#18998) [21f741d](https://github.com/answerbook/vector/commit/21f741dbba034b382881ba1c5efeef265e2fa5c8) - GitHub
* **nats source**: add subscriber_capacity option (#18899) [e7b563d](https://github.com/answerbook/vector/commit/e7b563d1955a0acc350364aba90e5dcd8c7b3c6e) - GitHub
* **sources, sinks**: add telemetry to http and grpc servers (#18887) [e779019](https://github.com/answerbook/vector/commit/e77901970b4d56168e24f9255fa42ed1f0e4ec86) - GitHub
* **tls**: add new dedicated page for TLS configuration (#18844) [625e4bd](https://github.com/answerbook/vector/commit/625e4bd785cc3678fa5685044e85cfc4bddce815) - GitHub
* **vrl**: add an example of parsing upstreaminfo with parse_nginx_log (#18815) [774094e](https://github.com/answerbook/vector/commit/774094ec1f8972c01e26d3f8a35429bea2091e01) - GitHub


### **BREAKING CHANGES**

* **security:** Remove legacy OpenSSL provider flags (#19015)
* **observability:** remove metrics replaced by component_errors_total (#18965)
* **observability:** remove `peer_addr` internal metric tag (#18982)
* **observability, blackhole sink:** Don't report by default (#18963)
* **observability:** remove deprecated `component_name` metric tag (#18942)
* **datadog:** remove deprecated config options (#18940)

## [2.1.2](https://github.com/answerbook/vector/compare/v2.1.1...v2.1.2) (2024-01-12)


### Bug Fixes

* **data_profiling**: Revert only track components in the analysis phase [4cec56c](https://github.com/answerbook/vector/commit/4cec56cc8024c3d833d3088c7b197446e3adfb52) - Jorge Bay [LOG-18984](https://logdna.atlassian.net/browse/LOG-18984)


### Miscellaneous

* Merge pull request #397 from answerbook/LOG-18984-revert [80f3cd4](https://github.com/answerbook/vector/commit/80f3cd4dfe5dde6d5b708de02791d8efb9fa0dbd) - GitHub [LOG-18984](https://logdna.atlassian.net/browse/LOG-18984)

## [2.1.1](https://github.com/answerbook/vector/compare/v2.1.0...v2.1.1) (2024-01-11)


### Chores

* pipeline_id is no longer a thing in remote task results [cd867d5](https://github.com/answerbook/vector/commit/cd867d56b79b54945e2a811a7557494ac2546a87) - Adam Holmberg [LOG-18815](https://logdna.atlassian.net/browse/LOG-18815)


### Miscellaneous

* Merge pull request #395 from answerbook/holmberg/LOG-18815 [d43a698](https://github.com/answerbook/vector/commit/d43a69811aa5b809dfe18bea38ff3a26086b3e95) - GitHub [LOG-18815](https://logdna.atlassian.net/browse/LOG-18815)

# [2.1.0](https://github.com/answerbook/vector/compare/v2.0.2...v2.1.0) (2024-01-11)


### Features

* **mezmo-sink**: add config for the LA route [a5df87f](https://github.com/answerbook/vector/commit/a5df87fc75487dc784fe11c6f7fcf4a0f4339b8d) - Mike Del Tito [LOG-19010](https://logdna.atlassian.net/browse/LOG-19010)


### Miscellaneous

* Merge pull request #394 from answerbook/mdeltito/LOG-19010 [00dcbad](https://github.com/answerbook/vector/commit/00dcbadf42724eaf0d392d9ca10d9962d185e8b8) - GitHub [LOG-19010](https://logdna.atlassian.net/browse/LOG-19010)

## [2.0.2](https://github.com/answerbook/vector/compare/v2.0.1...v2.0.2) (2024-01-11)


### Bug Fixes

* **data_profiling**: Only track components in the analysis phase [f18ac9d](https://github.com/answerbook/vector/commit/f18ac9ddc6c71dc4e24565ad32b14368c21680ec) - Jorge Bay [LOG-18984](https://logdna.atlassian.net/browse/LOG-18984)

## [2.0.1](https://github.com/answerbook/vector/compare/v2.0.0...v2.0.1) (2024-01-09)


### Bug Fixes

* **s3 consolidation**: large file copy source [8df07c0](https://github.com/answerbook/vector/commit/8df07c03171ad37b309772e4fe99daf135e3ed31) - dominic-mcallister-logdna [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)


### Miscellaneous

* Merge pull request #389 from answerbook/dominic/LOG-18535 [dec3be9](https://github.com/answerbook/vector/commit/dec3be9a0bc5a69cf4b2e3a84ba2ef78e6bc21cd) - GitHub [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)

# [2.0.0](https://github.com/answerbook/vector/compare/v1.36.2...v2.0.0) (2024-01-09)


### Bug Fixes

* **aws provider**: Don't unwap external_id (#18452) [fd0ccd5](https://github.com/answerbook/vector/commit/fd0ccd558715fc1e964755d2ebf7c9ca9fa1d7ee) - GitHub
* **aws_s3 source**: Allow region to be optional (#18258) [23a1a2d](https://github.com/answerbook/vector/commit/23a1a2df6bdf170054757ad048edaf18821989ae) - GitHub
* **aws_s3 source**: Use the decoder to calculate type defs (#18274) [40f525c](https://github.com/answerbook/vector/commit/40f525cae3eb7c6867afeeed4b2bd82cf85f5a65) - GitHub
* **ci**: add comment trigger filter for regression workflow concurrency group (#18408) [ff6e888](https://github.com/answerbook/vector/commit/ff6e8884941cbbced58b781936e9af1cf69dd7c5) - GitHub
* **ci**: don't continue on errors in unit-mac test (#18496) [712a210](https://github.com/answerbook/vector/commit/712a2101ceb980805dfeddccfdef8ee1b25fab8f) - GitHub
* **ci**: Drop docker-compose from bootstrap install (#18407) [41c5567](https://github.com/answerbook/vector/commit/41c55677aede24d217b9482ce4a6959cc6f43b10) - GitHub
* **ci**: Unlink python before `brew install` (#18402) [e39d9b3](https://github.com/answerbook/vector/commit/e39d9b3547124206135853688f653862e8c47a13) - GitHub
* **config**: fix concurrency default & docs (#18651) [b35527d](https://github.com/answerbook/vector/commit/b35527d142465d2274af6a582da681b049a3b8d3) - GitHub
* **config**: Only try default paths if the path is not specified (#18681) [b050a65](https://github.com/answerbook/vector/commit/b050a65286ec30ee6734472f8d295f2a924dd98b) - Jesse Szwedko
* **core**: don't show warning about max allocation groups if tracing not enabled (#18589) [3afda3c](https://github.com/answerbook/vector/commit/3afda3c5e8fca52dc689acb4d283988dd304c2fd) - GitHub
* **datadog_metrics sink**: improve aggregation performance (#18759) [4f97d48](https://github.com/answerbook/vector/commit/4f97d481a68c78eaece5c1e8c1fcbf30108d1197) - Jesse Szwedko
* **debian platform**: Re-add `conf-files` directive for `cargo-deb` (#18726) [aa39f5e](https://github.com/answerbook/vector/commit/aa39f5e724e2924a34c4ea8238aee2fa041a14a1) - Jesse Szwedko
* **debian platform**: Remove `conf-files` directive for `cargo-deb` (#18455) [40ef7c4](https://github.com/answerbook/vector/commit/40ef7c4d1f6505353aa7a17b289023243118f7e9) - GitHub
* default to nullbyte delimiter for GELF #18008 (#18419) [a112704](https://github.com/answerbook/vector/commit/a1127044f2c53439282be64a66f1ddf692f94ee7) - GitHub
* **deps, security**: temporarily ignore `ed25519-dalek` security vulnerability (#18245) [1b90398](https://github.com/answerbook/vector/commit/1b90398cbd0442c1cd639e736d6785c9cd790d49) - GitHub
* **deps**: fix [dev-dependencies] for some libs (#18328) [8086e19](https://github.com/answerbook/vector/commit/8086e19dcce467d29c07ef3e56ebe79bca75c57a) - GitHub
* **deps**: load default and legacy openssl providers (#18276) [fc17fba](https://github.com/answerbook/vector/commit/fc17fba992e584887690c9ade9397067427650d8) - GitHub
* **distribution**: Use yaml instead of toml file (#18606) [24701cf](https://github.com/answerbook/vector/commit/24701cfe6a643df61f0d6074f3ae92140de234e8) - GitHub
* **dnstap source**: support DNSSEC RRSIG record data (#18878) [3eaad37](https://github.com/answerbook/vector/commit/3eaad37e7d70518885523873f704da39d3999d39) - Jesse Szwedko
* **docs**: add the 'http_client_requests_sent_total' (#18299) [69e8383](https://github.com/answerbook/vector/commit/69e83838f4c07ad8449edab9d8edf95f94a2190a) - GitHub
* **docs**: remove '---\n' prefix from toYaml config example generator (#18502) [8d07e18](https://github.com/answerbook/vector/commit/8d07e184afa239ff9e111bdd8c0f4c7620f4d959) - GitHub
* **elasticsearch**: Ignore `pipeline` argument if it is an empty string (#18248) [d9dbed8](https://github.com/answerbook/vector/commit/d9dbed8896793016deb48808e59cb200c4e641a0) - GitHub
* **external docs**: Document intentional label on component_discarded_events_total (#18622) [fd58af9](https://github.com/answerbook/vector/commit/fd58af921b4b019ef11018a7200ceb50dc2ccac0) - GitHub
* **gcp service**: retry on unauthorized (#18586) [75d03b3](https://github.com/answerbook/vector/commit/75d03b370fbcf87872baebcb5d29ee2066ae88d2) - GitHub
* **json codec**: Fix deserializing non-object values with the `Vector` namespace (#18379) [f15144b](https://github.com/answerbook/vector/commit/f15144bb16a5d7a7389b20e2625c162101907f02) - GitHub
* **kafka sink**: Make KafkaService return `Poll::Pending` when producer queue is full (#18770) [fa09de3](https://github.com/answerbook/vector/commit/fa09de37c735bec57a67d78641b9db13c17097d8) - Jesse Szwedko
* **kafka sink**: performance improvements and fix memory leak (#18634) [3c662f3](https://github.com/answerbook/vector/commit/3c662f3ff0042826c38f8452b03d80b1a9db73ba) - GitHub
* **kubernetes_logs source**: Fix events being empty when log namespacing is enabled (#18244) [8918c66](https://github.com/answerbook/vector/commit/8918c66af5bca25603be2ef491c07afff350a4f5) - GitHub
* **new_relic sink**: Multiple fixes related to metrics (#18151) [953e305](https://github.com/answerbook/vector/commit/953e305470b474ce2ba368ab587db8abbc4693fa) - GitHub
* **observability**: add all events that are being encoded (#18289) [93baba2](https://github.com/answerbook/vector/commit/93baba2a9863a0f5ce2b0734a86fe927141c6446) - GitHub
* **observability**: don't increment component_errors_total for `HttpClient` warning (#18505) [a544e6e](https://github.com/answerbook/vector/commit/a544e6e8061244771ea89029e9489318773cf441) - GitHub
* **opentelemetry source**: Remove the 4MB default for gRPC request decoding (#18306) [a6262cd](https://github.com/answerbook/vector/commit/a6262cdea7c6ec4ab74752a627e01a11a74ff8be) - GitHub
* **prometheus_remote_write source, prometheus_scrape source**: Fix feature check (#18440) [9c1abd6](https://github.com/answerbook/vector/commit/9c1abd665fec92e03f8a19be1c13e89b359d5f07) - GitHub
* **releasing**: Update example YAML config data_dir (#18896) [ac86a8a](https://github.com/answerbook/vector/commit/ac86a8a1c578481b33a0b64634b02055e0fef429) - Jesse Szwedko
* **remap transform**: log namespace should be used when splitting events from arrays (#18372) [15a63b4](https://github.com/answerbook/vector/commit/15a63b404262ef297929b3693d2387bfdd445f90) - GitHub
* **sample transform**: Use metadata when log namespacing is enabled (#18259) [69b4c1c](https://github.com/answerbook/vector/commit/69b4c1c48a6f07a8dccf295a49962ef28050683f) - GitHub
* **sinks**: resolve memory leak by always setting a request builder concurrency limit (#18637) [6ced6ca](https://github.com/answerbook/vector/commit/6ced6ca22546d1033c66029dbe3b920868824134) - GitHub
* skip encoding empty sketches (#18530) [ae0aa11](https://github.com/answerbook/vector/commit/ae0aa11c409a5bb6b74c35adf22a06dcd1b42895) - GitHub
* socket tcp port typedef (#18180) [b0c89ab](https://github.com/answerbook/vector/commit/b0c89ab111ace1387acc85e2b2ca0e97217d9325) - GitHub
* **syslog source, docs**: Fix docs for `host` field for syslog source (#18453) [0ef902d](https://github.com/answerbook/vector/commit/0ef902d9181859fd29e1ff7b1a75ba4dc5d1da9a) - GitHub
* **tests**: fix tests for the generate command (#18383) [2a4235c](https://github.com/answerbook/vector/commit/2a4235ca2c32c0ca6314fb7262d69bdfceba0b15) - GitHub
* use `rstest` in `generate` command tests (vs wrong usage of `proptest`) (#18365) [4359c9a](https://github.com/answerbook/vector/commit/4359c9a9fae8a09096739152efe2e3936843f57e) - GitHub
* **website**: Fix installer list for MacOS (#18364) [13eec06](https://github.com/answerbook/vector/commit/13eec06d9addb4b558ae13465b2b1e875d5469da) - GitHub


### Chores

* Add SHA256 checksums file to GH releases (#18701) [3411642](https://github.com/answerbook/vector/commit/3411642407f2ab869065ff3767f6015ed30d5f50) - Jesse Szwedko
* **appsignal sink**: Refactor to use StreamSink (#18209) [c495939](https://github.com/answerbook/vector/commit/c49593964799ba05d587bd4d3c6d02ac34df190d) - GitHub
* **aws provider, external_docs**: Update the AWS authentication documentation (#18492) [cd8c5fe](https://github.com/answerbook/vector/commit/cd8c5fef1beb3690c944c029cff75ad935492d22) - GitHub
* **ci**: Add a test to assert conf files aren't overwritten (#18728) [ab272f6](https://github.com/answerbook/vector/commit/ab272f694ebe568fc8a8ec0f5361805ecc3fbd67) - Jesse Szwedko
* **ci**: Bump actions/checkout from 3 to 4 (#18476) [59dfd67](https://github.com/answerbook/vector/commit/59dfd6789c213396ddadff94460971056419fb9a) - GitHub
* **ci**: Bump aws-actions/configure-aws-credentials from 2.2.0 to 3.0.1 (#18386) [f33aff1](https://github.com/answerbook/vector/commit/f33aff1a81b64ec444d204cd5d9709d427139767) - GitHub
* **ci**: Bump aws-actions/configure-aws-credentials from 3.0.1 to 3.0.2 (#18511) [737d2f1](https://github.com/answerbook/vector/commit/737d2f1bb17506832b68bb69205db90b09768008) - GitHub
* **ci**: Bump aws-actions/configure-aws-credentials from 3.0.2 to 4.0.0 (#18544) [3018864](https://github.com/answerbook/vector/commit/30188644651731a088617a945dbcc16aee604871) - GitHub
* **ci**: Bump docker/build-push-action from 4.1.1 to 4.2.1 (#18512) [f19d166](https://github.com/answerbook/vector/commit/f19d166a33ac4887250f79c298124111af2b6422) - GitHub
* **ci**: Bump docker/build-push-action from 4.2.1 to 5.0.0 (#18546) [c30a4b2](https://github.com/answerbook/vector/commit/c30a4b2402eed5b0e9530b896d753572f447db01) - GitHub
* **ci**: Bump docker/login-action from 2 to 3 (#18556) [d976c6e](https://github.com/answerbook/vector/commit/d976c6e1d21854455f2e84b228ee3780c5eef776) - GitHub
* **ci**: Bump docker/metadata-action from 4.6.0 to 5.0.0 (#18543) [8aae235](https://github.com/answerbook/vector/commit/8aae235a3a9aa5ce07fe9131b8cb1bcc60c2e1bc) - GitHub
* **ci**: Bump docker/setup-buildx-action from 2.10.0 to 3.0.0 (#18545) [899e3c0](https://github.com/answerbook/vector/commit/899e3c0586428bc38fe4060afc1f99db7a11e41d) - GitHub
* **ci**: Bump docker/setup-buildx-action from 2.9.1 to 2.10.0 (#18406) [dd8a0ef](https://github.com/answerbook/vector/commit/dd8a0ef20e27eb315ef60733b88c8a11fadaa6ad) - GitHub
* **ci**: Bump docker/setup-qemu-action from 2.2.0 to 3.0.0 (#18547) [dcda1de](https://github.com/answerbook/vector/commit/dcda1deb26c55acdc0015d535f6dabf0196be3b8) - GitHub
* **ci**: Bump MacOS unit test runners to 13 (#18823) [c499427](https://github.com/answerbook/vector/commit/c499427ca1e3ebe9af89b19b7fe4204dd1ff5952) - Jesse Szwedko
* **ci**: Bump myrotvorets/set-commit-status-action from 1.1.7 to 2.0.0 (#18510) [8d5003a](https://github.com/answerbook/vector/commit/8d5003a04a929cac1df5c68e099ab5edbc2233d4) - GitHub
* **ci**: Bump tibdex/github-app-token from 1.8.0 to 1.8.2 (#18454) [4e4ece6](https://github.com/answerbook/vector/commit/4e4ece637681cafb949d09b0f6680069a4daa281) - GitHub
* **ci**: Bump tibdex/github-app-token from 1.8.2 to 2.0.0 (#18528) [7295f22](https://github.com/answerbook/vector/commit/7295f223576b8530a167bcef63ac1a1857db8ef1) - GitHub
* **ci**: Bump tibdex/github-app-token from 2.0.0 to 2.1.0 (#18608) [addc46e](https://github.com/answerbook/vector/commit/addc46e28dcd1c73f653bb507e55f01b2c52c759) - GitHub
* **ci**: Fix cookie banner style issues (#18745) [792a1b5](https://github.com/answerbook/vector/commit/792a1b541aaa1b34bef605bb9be4f0787b35afab) - Jesse Szwedko
* **ci**: group azure and prost crates for dependabot (#18525) [7b6ad62](https://github.com/answerbook/vector/commit/7b6ad621a07dc16b59e7922bc73ad12f4cb16bf5) - GitHub
* **ci**: Re-add docker-compose installation (#18415) [c1ed017](https://github.com/answerbook/vector/commit/c1ed01755788f15a96c2c83b2213b2c492155c85) - GitHub
* **ci**: remove kinetic as it's no longer supported (#18540) [c74a469](https://github.com/answerbook/vector/commit/c74a46959e4ebdd8a32d914112bdad8a2f8b1755) - GitHub
* **ci**: revert bump actions/checkout from 3 to 4 (#18490) [3fd6486](https://github.com/answerbook/vector/commit/3fd648603d2f46bbf6a01a3e7e4c4af7c70304ca) - GitHub
* **config, docs**: Replace 'vector.toml' with 'vector.yaml' (#18388) [9d29563](https://github.com/answerbook/vector/commit/9d295634538b9e97c0dbee1e1c72672c090ff83f) - GitHub
* **datadog_logs sink**: Use `endpoint` config setting consistent with the other datadog_ sinks. (#18497) [1ac19dd](https://github.com/answerbook/vector/commit/1ac19dded48f0ea3f91290c36f5462551277bbba) - GitHub
* **deps**: Bump `nkeys` to 0.3.2 (#18264) [fc533a1](https://github.com/answerbook/vector/commit/fc533a14fa1dfe013bd5435d94d108d1dcbca348) - GitHub
* **deps**: Bump anyhow from 1.0.72 to 1.0.74 (#18255) [a77b652](https://github.com/answerbook/vector/commit/a77b6521956448481e7aa8023f0e02506da7176c) - GitHub
* **deps**: Bump anyhow from 1.0.74 to 1.0.75 (#18284) [fb5f099](https://github.com/answerbook/vector/commit/fb5f099f1c2c403f6888c9f03c35121db50e3d0f) - GitHub
* **deps**: Bump async-compression from 0.4.1 to 0.4.2 (#18417) [8639655](https://github.com/answerbook/vector/commit/8639655ca0a5247eeb6bb44668fe3b3480051210) - GitHub
* **deps**: Bump async-compression from 0.4.2 to 0.4.3 (#18539) [3e3c7ad](https://github.com/answerbook/vector/commit/3e3c7ada10e7406aed31ac36e9fcfe0aa9bd65a8) - GitHub
* **deps**: Bump async-nats from 0.31.0 to 0.32.0 (#18640) [7ecb06a](https://github.com/answerbook/vector/commit/7ecb06ad45c151636259a555c58303c0a519a11c) - GitHub
* **deps**: Bump async-recursion from 1.0.4 to 1.0.5 (#18466) [8b017b6](https://github.com/answerbook/vector/commit/8b017b6231cb82752a8b112441648b57eca57d1f) - GitHub
* **deps**: Bump azure_core from 0.13.0 to 0.14.0 (#18361) [0bbb152](https://github.com/answerbook/vector/commit/0bbb152a067e070388c231d5dc4ec2edc8cd35d0) - GitHub
* **deps**: Bump base64 from 0.21.2 to 0.21.3 (#18398) [5dca377](https://github.com/answerbook/vector/commit/5dca377b8a45acaaeafdf0b9bdab04f5877a3880) - GitHub
* **deps**: Bump base64 from 0.21.3 to 0.21.4 (#18522) [44a87dc](https://github.com/answerbook/vector/commit/44a87dcf3face3c1cec9e20ab6de50be0dd3d131) - GitHub
* **deps**: Bump bollard from 0.14.0 to 0.15.0 (#18581) [996372f](https://github.com/answerbook/vector/commit/996372f935af0cc6a03d987ff92a5f98c5c89813) - GitHub
* **deps**: Bump bstr from 1.6.0 to 1.6.1 (#18422) [fa6f2cd](https://github.com/answerbook/vector/commit/fa6f2cdc880e4bdd0dc2e99ba0fb8ef2679182ba) - GitHub
* **deps**: Bump bstr from 1.6.1 to 1.6.2 (#18433) [8b6a307](https://github.com/answerbook/vector/commit/8b6a307a3f6960132f51a7fd793475e0fb7c7751) - GitHub
* **deps**: Bump bytes from 1.4.0 to 1.5.0 (#18508) [bfdb5b0](https://github.com/answerbook/vector/commit/bfdb5b046985ee9d6b4876b4ab4f1bb095c048b6) - GitHub
* **deps**: Bump bytesize from 1.2.0 to 1.3.0 (#18367) [836a31e](https://github.com/answerbook/vector/commit/836a31e34d9ed72c7ad8410d89d8610d72d8dd98) - GitHub
* **deps**: Bump cached from 0.44.0 to 0.45.0 (#18478) [325fbea](https://github.com/answerbook/vector/commit/325fbea85b08c569f203d5f32a0aeccc906d40ad) - GitHub
* **deps**: Bump cached from 0.45.0 to 0.45.1 (#18485) [f8981e1](https://github.com/answerbook/vector/commit/f8981e182ca1a4cb3c2366868294d9143c983f41) - GitHub
* **deps**: Bump cargo_toml from 0.15.3 to 0.16.0 (#18571) [0e60001](https://github.com/answerbook/vector/commit/0e600013a22bd1e91a956a9372ca15e5c28518fe) - GitHub
* **deps**: Bump cargo_toml from 0.16.0 to 0.16.1 (#18605) [c47e65f](https://github.com/answerbook/vector/commit/c47e65fc6521834c494d4b3afd15c37ada8b20c7) - GitHub
* **deps**: Bump cargo_toml from 0.16.1 to 0.16.2 (#18619) [fa526a8](https://github.com/answerbook/vector/commit/fa526a817a470feec273398d1c0127ca86f5a4f5) - GitHub
* **deps**: Bump cargo_toml from 0.16.2 to 0.16.3 (#18674) [a4cd8b7](https://github.com/answerbook/vector/commit/a4cd8b74872e118bfb4dc06d010b765195033070) - GitHub
* **deps**: Bump chrono to 0.4.30 (#18527) [a6305de](https://github.com/answerbook/vector/commit/a6305deb1638f440a928599223e4fe5cd7184bcd) - GitHub
* **deps**: Bump cidr-utils from 0.5.10 to 0.5.11 (#18516) [f5cd27a](https://github.com/answerbook/vector/commit/f5cd27afd0bd6aa93ac1b18f508f0ecedaa87489) - GitHub
* **deps**: Bump clap from 4.3.21 to 4.3.22 (#18293) [47f25d3](https://github.com/answerbook/vector/commit/47f25d31b0124c0c75d4679efff16ebf4f02dc0f) - GitHub
* **deps**: Bump clap from 4.3.22 to 4.3.23 (#18311) [05765d8](https://github.com/answerbook/vector/commit/05765d8a4e773974fbfe8cda2d35130be521fc7e) - GitHub
* **deps**: Bump clap from 4.3.23 to 4.3.24 (#18362) [5f4a6d8](https://github.com/answerbook/vector/commit/5f4a6d8354c62a474ccdfa8954a60f0b9afcc2b7) - GitHub
* **deps**: Bump clap from 4.3.24 to 4.4.1 (#18411) [d3b9eda](https://github.com/answerbook/vector/commit/d3b9eda8ba4e46eaa87b17983577a00eb7642dd0) - GitHub
* **deps**: Bump clap from 4.4.1 to 4.4.2 (#18447) [6998518](https://github.com/answerbook/vector/commit/699851891d23eab8364f740bd06657a2e90c85cb) - GitHub
* **deps**: Bump clap from 4.4.2 to 4.4.3 (#18550) [9e8407e](https://github.com/answerbook/vector/commit/9e8407e3ff5dc7d063b886b2eef43673c7bc7c39) - GitHub
* **deps**: Bump clap_complete from 4.3.2 to 4.4.0 (#18374) [76ffdab](https://github.com/answerbook/vector/commit/76ffdabb89cb15df857959f930c9bcb5255e2e33) - GitHub
* **deps**: Bump clap_complete from 4.4.0 to 4.4.1 (#18509) [af444ea](https://github.com/answerbook/vector/commit/af444ea9dcb9ec5c3b32904b5a9f1267b1bb634d) - GitHub
* **deps**: Bump clap_complete from 4.4.1 to 4.4.2 (#18673) [3edafef](https://github.com/answerbook/vector/commit/3edafefa359f2bcf8a61bd15267814d53166a914) - GitHub
* **deps**: Bump crossterm from 0.26.1 to 0.27.0 (#18168) [d9c75bd](https://github.com/answerbook/vector/commit/d9c75bd3c2675c192d907a7d0436320e915b7916) - GitHub
* **deps**: Bump dashmap from 5.5.0 to 5.5.1 (#18338) [754bee0](https://github.com/answerbook/vector/commit/754bee00333ef80dc236fd2f11bfe3cf42335da8) - GitHub
* **deps**: Bump dashmap from 5.5.1 to 5.5.3 (#18427) [adf134b](https://github.com/answerbook/vector/commit/adf134b77fdd9f998bca8cb4a6af808baa2cf321) - GitHub
* **deps**: Bump dyn-clone from 1.0.12 to 1.0.13 (#18285) [1fbbdcb](https://github.com/answerbook/vector/commit/1fbbdcb235519bbc407b6291a035d5e6e87d0955) - GitHub
* **deps**: Bump dyn-clone from 1.0.13 to 1.0.14 (#18607) [d5f4caa](https://github.com/answerbook/vector/commit/d5f4caa3d69cd2f9ff71350925e6c36d9bb9b611) - GitHub
* **deps**: Bump encoding_rs from 0.8.32 to 0.8.33 (#18360) [2493288](https://github.com/answerbook/vector/commit/2493288204781314d7b9a08d439bc50b4a0ed5b4) - GitHub
* **deps**: Bump enumflags2 from 0.7.7 to 0.7.8 (#18560) [0f90c39](https://github.com/answerbook/vector/commit/0f90c39f8dd515a6fbff5484f780903c74ab02b7) - GitHub
* **deps**: Bump flate2 from 1.0.26 to 1.0.27 (#18254) [3a6af99](https://github.com/answerbook/vector/commit/3a6af99c3975859f16c0aac5c30de77a7932fad6) - GitHub
* **deps**: Bump h2 from 0.3.20 to 0.3.21 (#18330) [d41f9f6](https://github.com/answerbook/vector/commit/d41f9f6082d64f1ea9585ae4642cd9646d8621b4) - GitHub
* **deps**: Bump headers from 0.3.8 to 0.3.9 (#18448) [749594c](https://github.com/answerbook/vector/commit/749594cf357bd0b1932de8e83b287b2cfe51c54c) - GitHub
* **deps**: Bump http-serde from 1.1.2 to 1.1.3 (#18310) [31ec4b3](https://github.com/answerbook/vector/commit/31ec4b387b9ef66c803606ac257f3e8580ddc5e0) - GitHub
* **deps**: Bump indicatif from 0.17.6 to 0.17.7 (#18647) [6bab5be](https://github.com/answerbook/vector/commit/6bab5be898dbc14f57bba6c46fb5cc0ac340e89e) - GitHub
* **deps**: Bump indoc from 2.0.3 to 2.0.4 (#18582) [b3f76b5](https://github.com/answerbook/vector/commit/b3f76b56d7d3d2a07ab33c6cdbb9cf4ac9f87fb0) - GitHub
* **deps**: Bump inventory from 0.3.11 to 0.3.12 (#18429) [9f95026](https://github.com/answerbook/vector/commit/9f9502662352093fd14ddc99758f0164bec36352) - GitHub
* **deps**: Bump libc from 0.2.147 to 0.2.148 (#18563) [59e22fc](https://github.com/answerbook/vector/commit/59e22fcb2ca115d168fded1ed52def2743f52281) - GitHub
* **deps**: Bump lru from 0.11.0 to 0.11.1 (#18484) [7ec5b97](https://github.com/answerbook/vector/commit/7ec5b9703c4ea08ecb486ba370482b031892a846) - GitHub
* **deps**: Bump md-5 from 0.10.5 to 0.10.6 (#18648) [e8d946f](https://github.com/answerbook/vector/commit/e8d946fba2170eca1220d8caa527201767f4298f) - GitHub
* **deps**: Bump memchr from 2.5.0 to 2.6.0 (#18410) [7bc1942](https://github.com/answerbook/vector/commit/7bc19427eab2b14992ee64fb1032baf30414d5cc) - GitHub
* **deps**: Bump memchr from 2.6.0 to 2.6.1 (#18421) [3f4603c](https://github.com/answerbook/vector/commit/3f4603cc3d6e77b5163a06671f7d1d72e1eea1de) - GitHub
* **deps**: Bump memchr from 2.6.1 to 2.6.2 (#18434) [5e3eaa9](https://github.com/answerbook/vector/commit/5e3eaa984fbafd8956bd77db072de429a2d5c4b2) - GitHub
* **deps**: Bump memchr from 2.6.2 to 2.6.3 (#18470) [69a1ca0](https://github.com/answerbook/vector/commit/69a1ca011ab86237fc1351fb4f5dad4c7cdb3dce) - GitHub
* **deps**: Bump mlua from 0.8.9 to 0.8.10 (#18292) [03fe2fe](https://github.com/answerbook/vector/commit/03fe2fea176066729b9ef27c27acbfa46fcabd96) - GitHub
* **deps**: Bump mongodb from 2.6.0 to 2.6.1 (#18268) [28de959](https://github.com/answerbook/vector/commit/28de9594def66b3a6222c550683f91ef93fe6739) - GitHub
* **deps**: Bump MSRV to 1.70.0 (#18394) [fd21b19](https://github.com/answerbook/vector/commit/fd21b19a5b57d173723ecf84e8b9216dfad359cd) - GitHub
* **deps**: Bump no-proxy from 0.3.3 to 0.3.4 (#18277) [7a1c49c](https://github.com/answerbook/vector/commit/7a1c49c3bc65743fc2c0e688233357af5b3ad4cd) - GitHub
* **deps**: Bump notify from 6.0.1 to 6.1.0 (#18317) [e963766](https://github.com/answerbook/vector/commit/e9637665b04a0dfe2a785673607f7592ee8fecac) - GitHub
* **deps**: Bump notify from 6.1.0 to 6.1.1 (#18332) [ae5de9c](https://github.com/answerbook/vector/commit/ae5de9cd2d112879a56d3e81600b0c932588aa63) - GitHub
* **deps**: Bump num_enum from 0.6.1 to 0.7.0 (#18238) [771f476](https://github.com/answerbook/vector/commit/771f47685241bf849315c9a0c1bd3f0cf74ddcd4) - GitHub
* **deps**: Bump openssl from 0.10.56 to 0.10.57 (#18400) [15792f6](https://github.com/answerbook/vector/commit/15792f62477a57fdb79397c3d209128d8493bf55) - GitHub
* **deps**: Bump ordered-float from 3.7.0 to 3.8.0 (#18302) [833ac19](https://github.com/answerbook/vector/commit/833ac19092bd7c7b88514d98c2c7176e5ff002b1) - GitHub
* **deps**: Bump ordered-float from 3.8.0 to 3.9.0 (#18307) [ca30b6d](https://github.com/answerbook/vector/commit/ca30b6d9558e0457a6b07133441f9c79959e63ac) - GitHub
* **deps**: Bump ordered-float from 3.9.0 to 3.9.1 (#18350) [ab41edc](https://github.com/answerbook/vector/commit/ab41edc475783a4285b9cc7b2b95384471c71fd3) - GitHub
* **deps**: Bump proc-macro2 from 1.0.66 to 1.0.67 (#18561) [6f091e1](https://github.com/answerbook/vector/commit/6f091e1353485164ec7fc8871f69d6d40f703578) - GitHub
* **deps**: Bump prost from 0.11.9 to 0.12.0 (#18460) [4fba377](https://github.com/answerbook/vector/commit/4fba377b66a564a6753579f27fcbd8f6f32644b6) - GitHub
* **deps**: Bump prost-reflect from 0.11.4 to 0.11.5 (#18426) [221e0a1](https://github.com/answerbook/vector/commit/221e0a1379aa36f99ed85be124644801ddd8b862) - GitHub
* **deps**: Bump quote from 1.0.32 to 1.0.33 (#18283) [f2a6887](https://github.com/answerbook/vector/commit/f2a68871ddff4bcfe79af3d7af7351f8b75fba38) - GitHub
* **deps**: Bump ratatui from 0.22.0 to 0.23.0 (#18412) [82be883](https://github.com/answerbook/vector/commit/82be883cfc4c0cbfb39e071d285a45a19a6693d9) - GitHub
* **deps**: Bump rdkafka from 0.33.2 to 0.34.0 (#18393) [016890e](https://github.com/answerbook/vector/commit/016890e566008960419c28e72f371cd217cc7885) - GitHub
* **deps**: Bump redis from 0.23.2 to 0.23.3 (#18464) [2e2692e](https://github.com/answerbook/vector/commit/2e2692ed94192fb75bd447e132acadcff9927857) - GitHub
* **deps**: Bump regex from 1.9.3 to 1.9.4 (#18399) [21f7679](https://github.com/answerbook/vector/commit/21f7679e9cf152d6d0f6cd6c8abeae72cbe3b365) - GitHub
* **deps**: Bump regex from 1.9.4 to 1.9.5 (#18472) [1aeb42e](https://github.com/answerbook/vector/commit/1aeb42e98c2d36d2e167327ce1a9f60bbc90432b) - GitHub
* **deps**: Bump reqwest from 0.11.18 to 0.11.19 (#18329) [9b4625c](https://github.com/answerbook/vector/commit/9b4625ce54c8c31a2b627732b3e13648a28fc0b9) - GitHub
* **deps**: Bump reqwest from 0.11.19 to 0.11.20 (#18366) [2aaea89](https://github.com/answerbook/vector/commit/2aaea89e8a63ae23eddfa9ac72dbcf52cc58c4c0) - GitHub
* **deps**: Bump rmpv from 1.0.0 to 1.0.1 (#18049) [0cda906](https://github.com/answerbook/vector/commit/0cda90678c22351d6ae08e9733751c3432d3e2d2) - GitHub
* **deps**: Bump Rust to 1.72.1 (#18638) [466ef84](https://github.com/answerbook/vector/commit/466ef846bdd7c58f1c341dc557fe01eac47bcc1d) - GitHub
* **deps**: Bump serde from 1.0.183 to 1.0.185 (#18319) [ea2d576](https://github.com/answerbook/vector/commit/ea2d57667ebe5bbea2124403b94febc702fcf759) - GitHub
* **deps**: Bump serde from 1.0.185 to 1.0.186 (#18370) [83ec3cf](https://github.com/answerbook/vector/commit/83ec3cf555ea801b7d1b9a754d3b7695970d6270) - GitHub
* **deps**: Bump serde from 1.0.186 to 1.0.188 (#18395) [4a2805d](https://github.com/answerbook/vector/commit/4a2805d4984a46755b79ef347b723f9e301cb96c) - GitHub
* **deps**: Bump serde_derive_internals from 0.28.0 to 0.29.0 (#18499) [f7c3c96](https://github.com/answerbook/vector/commit/f7c3c965ed45ce05d0a3c5d8f10d12068c9e7f0c) - GitHub
* **deps**: Bump serde_json from 1.0.104 to 1.0.105 (#18267) [d05bc3e](https://github.com/answerbook/vector/commit/d05bc3e636082d7adb27080e9dcff855c3928336) - GitHub
* **deps**: Bump serde_json from 1.0.105 to 1.0.106 (#18523) [2687ed1](https://github.com/answerbook/vector/commit/2687ed1f9f9a88bc4d39813907a4d239dc6d50a8) - GitHub
* **deps**: Bump serde_json from 1.0.106 to 1.0.107 (#18562) [3c3e251](https://github.com/answerbook/vector/commit/3c3e25194623f966fc32dc637e5adc22dbb84168) - GitHub
* **deps**: Bump serde_with from 3.2.0 to 3.3.0 (#18315) [54d48d7](https://github.com/answerbook/vector/commit/54d48d79b5ed569a8e213c697a79dd0d91177d2e) - GitHub
* **deps**: Bump serde-wasm-bindgen from 0.5.0 to 0.6.0 (#18565) [95297b2](https://github.com/answerbook/vector/commit/95297b2014277ca990a98283098aa6ed4d388fc4) - GitHub
* **deps**: Bump similar-asserts from 1.4.2 to 1.5.0 (#18318) [b3a1d53](https://github.com/answerbook/vector/commit/b3a1d5307453e5c4ba70c3b2bde45f879f5b9973) - GitHub
* **deps**: Bump smallvec from 1.11.0 to 1.11.1 (#18620) [feca4c8](https://github.com/answerbook/vector/commit/feca4c8e89f7166df29e321f78985395e118ebf0) - GitHub
* **deps**: Bump socket2 from 0.5.3 to 0.5.4 (#18531) [8eb5256](https://github.com/answerbook/vector/commit/8eb52562f034930b83eb4416a667bea1381dd9c8) - GitHub
* **deps**: Bump syn from 2.0.28 to 2.0.29 (#18282) [d10e373](https://github.com/answerbook/vector/commit/d10e37379a11b430f527895bcaa207533a71817b) - GitHub
* **deps**: Bump syn from 2.0.29 to 2.0.31 (#18471) [f2b46d6](https://github.com/answerbook/vector/commit/f2b46d6cc6eaaf099354d5235fd749e60e238d54) - GitHub
* **deps**: Bump syn from 2.0.31 to 2.0.32 (#18524) [f30537e](https://github.com/answerbook/vector/commit/f30537e2840cf51513d52dd5563f4233d98a48d8) - GitHub
* **deps**: Bump syn from 2.0.32 to 2.0.33 (#18559) [f750cf7](https://github.com/answerbook/vector/commit/f750cf7480881976b65ff2c4b4c4e6d62b51be14) - GitHub
* **deps**: Bump syn from 2.0.33 to 2.0.37 (#18580) [e80d7b7](https://github.com/answerbook/vector/commit/e80d7b7eb29e90ac4a2156675464f191c3e428ec) - GitHub
* **deps**: Bump the azure group with 4 updates (#18529) [3d7199e](https://github.com/answerbook/vector/commit/3d7199ea42368a09d5ac909b6fd89f66758889a3) - GitHub
* **deps**: Bump the prost group with 3 updates (#18579) [16edc22](https://github.com/answerbook/vector/commit/16edc225dcb3f58a79c2b703c35eec7dc650605e) - GitHub
* **deps**: Bump thiserror from 1.0.44 to 1.0.46 (#18253) [2a8d974](https://github.com/answerbook/vector/commit/2a8d9749eeb6d94dea583bb6f3cd03b8a340ec35) - GitHub
* **deps**: Bump thiserror from 1.0.46 to 1.0.47 (#18286) [378926d](https://github.com/answerbook/vector/commit/378926d863dfb82ff71380ec11db40d14f188e85) - GitHub
* **deps**: Bump thiserror from 1.0.47 to 1.0.48 (#18473) [7ad6313](https://github.com/answerbook/vector/commit/7ad631348760abe42bba42950c4a7571ba187f42) - GitHub
* **deps**: Bump to Rust 1.72.0 (#18389) [7849d80](https://github.com/answerbook/vector/commit/7849d804cf2bf73c2f3e77e03c8b7af73aaa1a06) - GitHub
* **deps**: Bump tokio from 1.30.0 to 1.32.0 (#18279) [47051a5](https://github.com/answerbook/vector/commit/47051a5b80bf970fdfe8c3348d88d4fb3c1542d7) - GitHub
* **deps**: Bump tokio-postgres from 0.7.7 to 0.7.9 (#18316) [a7800f7](https://github.com/answerbook/vector/commit/a7800f74b0cdd75415b6852e37e03dbd4a4f4e27) - GitHub
* **deps**: Bump tokio-postgres from 0.7.9 to 0.7.10 (#18391) [725b9bd](https://github.com/answerbook/vector/commit/725b9bd7feba15f7ab7c22d75f61934c0ef80c67) - GitHub
* **deps**: Bump tokio-test from 0.4.2 to 0.4.3 (#18357) [6716959](https://github.com/answerbook/vector/commit/671695929ccc4a17cfcd26fba757a8692eb4fbe3) - GitHub
* **deps**: Bump tokio-tungstenite from 0.20.0 to 0.20.1 (#18661) [42fea39](https://github.com/answerbook/vector/commit/42fea39d33093e232ca4afe95fee9dfc494ccf65) - Jesse Szwedko
* **deps**: Bump toml from 0.7.6 to 0.7.7 (#18507) [688e2d9](https://github.com/answerbook/vector/commit/688e2d90f3ecf2fc801aaa20785e43d6886876c1) - GitHub
* **deps**: Bump toml from 0.7.7 to 0.7.8 (#18520) [0476bb5](https://github.com/answerbook/vector/commit/0476bb5f55494e4ec70d298204ffe1fcb4524780) - GitHub
* **deps**: Bump toml from 0.7.8 to 0.8.0 (#18549) [0382bc5](https://github.com/answerbook/vector/commit/0382bc5bb89ccde211cd57955b56db091923a799) - GitHub
* **deps**: Bump tonic from 0.10.0 to 0.10.1 (#18639) [9523987](https://github.com/answerbook/vector/commit/9523987bf9ba7df2d9731dfa6dc3be0afcc24611) - GitHub
* **deps**: Bump tonic-build from 0.10.0 to 0.10.1 (#18641) [ec9efb5](https://github.com/answerbook/vector/commit/ec9efb5d092086bc43f037f4a342df2cbb398127) - GitHub
* **deps**: Bump tower-http from 0.4.3 to 0.4.4 (#18461) [d3d8a4c](https://github.com/answerbook/vector/commit/d3d8a4c703bbe4aea2ad1945fc93f446457dc5ca) - GitHub
* **deps**: Bump trust-dns-proto from 0.22.0 to 0.23.0 (#18349) [a252eda](https://github.com/answerbook/vector/commit/a252eda9ee70e0bc717196e4ace98d0ada87f270) - GitHub
* **deps**: Bump typetag from 0.2.12 to 0.2.13 (#18287) [704cbfe](https://github.com/answerbook/vector/commit/704cbfee475f6bed6b7e22fdf463e2fbbbad3a76) - GitHub
* **deps**: Bump url from 2.4.0 to 2.4.1 (#18414) [314a37b](https://github.com/answerbook/vector/commit/314a37b0fa852e96348b9f490bf83830f1dc9f1b) - GitHub
* **deps**: Bump warp from 0.3.5 to 0.3.6 (#18704) [0a5d29e](https://github.com/answerbook/vector/commit/0a5d29e84e067c6d39be2fe6f111c1d57aec91ae) - Jesse Szwedko
* **deps**: Bump webpki 0.22.1 -> 0.22.4 [d9c4f2e](https://github.com/answerbook/vector/commit/d9c4f2e18afbc0878f5fa631f19109beed45118f) - Jesse Szwedko
* **deps**: Bump webpki from 0.22.0 to 0.22.1 (#18494) [9859b9e](https://github.com/answerbook/vector/commit/9859b9ed93d20d61df7f24aa25dbeabc7bda2d27) - GitHub
* **deps**: Bump webpki from 0.22.1 to 0.22.2 (#18744) [2cbc16c](https://github.com/answerbook/vector/commit/2cbc16c8641c7291de152f02c000d489eb0fe76d) - Jesse Szwedko
* **deps**: Drop patch to use custom `chrono` repo (#18567) [6c34cd5](https://github.com/answerbook/vector/commit/6c34cd531d6c5ceebe17d68ee5044c610e081c59) - GitHub
* **deps**: Drop use of `once_cell::{sync,unsync}::OnceCell` (#17621) [f2cd59a](https://github.com/answerbook/vector/commit/f2cd59ad048c9b867e240cc29e92de1cbffc2d5b) - GitHub
* **deps**: Fix issue with `cargo` refetching refs on every run (#18331) [6397edb](https://github.com/answerbook/vector/commit/6397edbcca5a0672985b93c8720dbe9c04caec8f) - GitHub
* **deps**: group tonic crates in dependabot (#18645) [32be0d3](https://github.com/answerbook/vector/commit/32be0d3658919dc4d8fe38392a2f9a120144c40c) - GitHub
* **deps**: Remove openssl-sys patch (#18495) [8ec87eb](https://github.com/answerbook/vector/commit/8ec87eb43073b4242f489ea91cc56f684905e006) - GitHub
* **deps**: Update fork of rust-openssl (#18404) [f8d073e](https://github.com/answerbook/vector/commit/f8d073eb3fbc3569b3d47ddc9a755f17ced5114d) - GitHub
* **deps**: Update VRL to 0.7.0 (#18672) [70e8b5f](https://github.com/answerbook/vector/commit/70e8b5fe55dc9c4767f2fc992e83ac3bb932b740) - GitHub
* **dev**: Add CODEOWNERS for documentation updates (#18628) [918beac](https://github.com/answerbook/vector/commit/918beac97732d9517c61fee5dfaf0b97e03fe6ae) - GitHub
* **dev**: Add DEPRECATIONS.md file to track deprecations (#18613) [d8f36e4](https://github.com/answerbook/vector/commit/d8f36e45b7f443a97dc6367d79bc16620971e05d) - GitHub
* **distribution**: Fix PATH modification to allow for spaces (#18294) [aed8224](https://github.com/answerbook/vector/commit/aed8224d4d9aaaa2d94ccf9e2919593a32a0010e) - GitHub
* **docs**: Add highlight announcing YAML as the new default config … (#18435) [c150b14](https://github.com/answerbook/vector/commit/c150b144a062d96027424e982b54f74b7e37072d) - GitHub
* **docs**: add more comparison examples (#18333) [2f458f6](https://github.com/answerbook/vector/commit/2f458f61584b0aa046cabb7c1b4c82b87496cc98) - GitHub
* **docs**: Change the default configuration tab and add comments to… (#18420) [ed1dedf](https://github.com/answerbook/vector/commit/ed1dedf5855ca5eafbc27d5abc9789f00fafa342) - GitHub
* **docs**: Convert a few more configs to YAML (#18632) [f2d60cb](https://github.com/answerbook/vector/commit/f2d60cbb9d2a4bdf74e559113c76a8d53a243f3d) - GitHub
* **docs**: discuss disk throughput configurations in sizing guidance (#18566) [5a52b61](https://github.com/answerbook/vector/commit/5a52b61f5bfa1339e80553da89adba1a1db0d527) - GitHub
* **docs**: Emphasize the "may" bit of the backpressure docs (#18457) [1647964](https://github.com/answerbook/vector/commit/164796404fb01e3f7bd9207e00a42e3a8251142a) - GitHub
* **docs**: make YAML appear first in the example configurations (#18325) [85dd439](https://github.com/answerbook/vector/commit/85dd43921f9948148d088cb94d3e49127cd613c1) - GitHub
* **exec source**: remove obsolete codec file and import (#18257) [adbc06f](https://github.com/answerbook/vector/commit/adbc06fd45f9ab0bd34a1981ee836e7890b70b06) - GitHub
* **exec source**: split tests into own module file (#18301) [ac90069](https://github.com/answerbook/vector/commit/ac90069d448b094f17fc8fc4a212bd73f7b6ca36) - GitHub
* **external docs**: Remove or replace mentions of vector in functions doc (#18679) [3910677](https://github.com/answerbook/vector/commit/391067761a210341f68ad3a4db8fcd0cfa42e578) - Jesse Szwedko
* feature gate aws-core features (#18482) [e19243f](https://github.com/answerbook/vector/commit/e19243fb05f2f65705892258aae1a1becb4040fe) - GitHub
* **gcp_stackdriver_logs sink**: refactor to new style (#18335) [0d8ab26](https://github.com/answerbook/vector/commit/0d8ab26c33e7dfca8cff4e84ed4559f1b4553ca0) - GitHub
* **honeycomb sink**: refactor to new style (#18280) [e104dc4](https://github.com/answerbook/vector/commit/e104dc461130b47faeb91d27e486097202981361) - GitHub
* **http sink**: refactor to new style (#18200) [d3a6235](https://github.com/answerbook/vector/commit/d3a623540ec08e63dd3b075614159db663ac0dc2) - GitHub
* Improve checksum script (#18487) [8981cba](https://github.com/answerbook/vector/commit/8981cba07c768b0155d90b92d799eed464ee0b7a) - GitHub
* **internal docs**: Add workspace to new rust docs deployment (#18616) [180647b](https://github.com/answerbook/vector/commit/180647b895aafc4aeb53dad10f8dae48a656ea3e) - GitHub
* **kubernetes**: Regenerate manifests from new Helm chart version (#18629) [99de28b](https://github.com/answerbook/vector/commit/99de28b67799ed190e8cf00ee6f148ac1924ae37) - GitHub
* **log_to_metric transform**: Add Cargo feature for the transform (#18337) [d044084](https://github.com/answerbook/vector/commit/d0440848e70d6f8eaedfaa20696c74b60d631519) - GitHub
* **log_to_metric transform**: add missing Cargo feature (#18308) [69621bd](https://github.com/answerbook/vector/commit/69621bd79ad38ed6059443c739886eb5d611b5af) - GitHub
* **log_to_metric transform**: Revert missing Cargo feature `transforms-log_to_metric` (#18327) [e6ec664](https://github.com/answerbook/vector/commit/e6ec664538715026a3ab78b83f2e59ebd9b5a409) - GitHub
* make pin-project a workspace dependency (#18176) [b755a46](https://github.com/answerbook/vector/commit/b755a46901af404102c77df286187ffa451c6f49) - GitHub
* **nats sink**: Refactor to use StreamSink components (#18243) [294c1dd](https://github.com/answerbook/vector/commit/294c1ddfb3f0398105fb02b37c9b9a38e50a6a6c) - GitHub
* **prometheus_remote_write source, prometheus_scrape source**: Split compilation features (#18431) [c07c99d](https://github.com/answerbook/vector/commit/c07c99d95b72622a536701c104966d143a184848) - GitHub
* **redis sink**: Refactor to use StreamSink (#18220) [1e7e99c](https://github.com/answerbook/vector/commit/1e7e99c1840d57fbab9fa00b6d894f874ae0bd31) - GitHub
* **releasing**: Add 0.32.0 highlight for legacy OpenSSL provider deprecation (#18263) [62bd340](https://github.com/answerbook/vector/commit/62bd3406c624a62ba939defbb5c234f580698725) - GitHub
* **releasing**: Add known issue for 0.33.0 debian packaging regression (#18727) [eae7b82](https://github.com/answerbook/vector/commit/eae7b827fb885af5af12419b3451c841df06abdf) - Jesse Szwedko
* **releasing**: Add known issues for v0.32.0 (#18298) [d09b2af](https://github.com/answerbook/vector/commit/d09b2af3488e0f9734ce30404e304722628ffaa1) - GitHub
* **releasing**: Add note about protobuf codec addition for 0.32.0 release (#18275) [87850ee](https://github.com/answerbook/vector/commit/87850ee31c67e6b0d575644b74646010689d28c2) - GitHub
* **releasing**: Bump k8s manifests to v0.24.1 of the chart (#18334) [a1d05e4](https://github.com/answerbook/vector/commit/a1d05e42b0ff69bbb04cedb359b2614b09b1a400) - GitHub
* **releasing**: Bump Vector to 0.33.0 (#18250) [ef51e7a](https://github.com/answerbook/vector/commit/ef51e7a52e0fadea78b0f68c78d4bee78d1fb6bc) - GitHub
* **releasing**: Prepare v0.32.0 release [50685f9](https://github.com/answerbook/vector/commit/50685f9602ae12b0a3229994b3526ed7cdc7d11c) - Jesse Szwedko
* **releasing**: Prepare v0.32.1 release [bb42d52](https://github.com/answerbook/vector/commit/bb42d52b5482569711039a6401b47fa31a20bdaf) - Jesse Szwedko
* **releasing**: Prepare v0.32.2 release [d833296](https://github.com/answerbook/vector/commit/d83329607c01a8d37e8284fba4931b6409e6cf95) - Jesse Szwedko
* **releasing**: Prepare v0.33.0 release [89605fb](https://github.com/answerbook/vector/commit/89605fbba72b4a8572f0265248dba2174415c764) - Jesse Szwedko
* **releasing**: Prepare v0.33.1 release [3cc27b9](https://github.com/answerbook/vector/commit/3cc27b98bbab84e6e749cc405fe6a62797a0926d) - Jesse Szwedko
* **releasing**: Regenerate k8s manifests for v0.24.0 of the chart (#18251) [d155a23](https://github.com/answerbook/vector/commit/d155a237281c02960d44d03b4ca3737290de2504) - GitHub
* **releasing**: Use large pages for better OS compatibility (#18481) [e23941c](https://github.com/answerbook/vector/commit/e23941c59ba7b7f14cd318ce5b946f31d307106a) - GitHub
* **rust-doc**: Add new Make command for CI builds (#18444) [4127b2c](https://github.com/answerbook/vector/commit/4127b2c0a6a2f92320f54b742ab01c95896bddc9) - GitHub
* **rust-doc**: Update amplify build to include workspace flag (#18630) [f01354e](https://github.com/answerbook/vector/commit/f01354e484cd85e187e4ff4f35d4516a7833bd0e) - GitHub
* **security**: Make the warning for the deprecated OpenSSL provider more verbose (#18278) [7952bfb](https://github.com/answerbook/vector/commit/7952bfbdfd01a2c11b07c527d3926efcdd6c4664) - GitHub
* tidy `encode_input` function (#18300) [2a45722](https://github.com/answerbook/vector/commit/2a45722cc777c7b971754148928bc376a06d82b1) - GitHub
* update `rustls-webpki` due to security advisory (#18344) [b982a74](https://github.com/answerbook/vector/commit/b982a74c82ce54e994ca19c39e22d10563192f1f) - GitHub
* Use `Ipv#Addr` constants (#17627) [7a6365e](https://github.com/answerbook/vector/commit/7a6365e2dec20b374bd2e9e31760a46014dc8d21) - GitHub
* **websites**: Add amplify build spec files to appropriate directories (#18668) [7cbb758](https://github.com/answerbook/vector/commit/7cbb7585327b829e8d3f32b4b8047d0af22700c6) - GitHub
* **website**: Set download page dropdown to latest version (#18758) [4b72f7e](https://github.com/answerbook/vector/commit/4b72f7e13c7607705fe16227259bd7b1429fc1f7) - Jesse Szwedko
* **website**: Update chat.vector.dev redirect (#18635) [76efb4b](https://github.com/answerbook/vector/commit/76efb4b91792a8c59eb7c4b0376220364dd2d72f) - GitHub


### Features

* Add checksums for artifacts (#18483) [f11eeb3](https://github.com/answerbook/vector/commit/f11eeb3cd8b79aa5cb942434505438d0c6e48a0d) - GitHub
* add convert config command (#18378) [9cb0ca3](https://github.com/answerbook/vector/commit/9cb0ca3b4a8705e2959c62a081345531f74e8b14) - GitHub
* Add git sha to the VRL playground header (#18500) [567de50](https://github.com/answerbook/vector/commit/567de50d66aca3b6302063a55988adfc5fb19540) - GitHub
* add support for YAML and JSON to the generate command (#18345) [e15aec7](https://github.com/answerbook/vector/commit/e15aec7a3896a8b4091832fd1e42b7279483db52) - GitHub
* **appsignal sink**: Normalize metrics  (#18217) [61c0ae8](https://github.com/answerbook/vector/commit/61c0ae8a54ecd17a4457f6916987effa3d2f903b) - GitHub
* Begin publishing armv7hl rpm packages (#18387) [4c901ed](https://github.com/answerbook/vector/commit/4c901ed9e241ac6b7076a292efad3958c0d9ecde) - GitHub
* change dedupe config paths to `ConfigTargetPath` (#18241) [4ec6c11](https://github.com/answerbook/vector/commit/4ec6c11a72a1312c5a79135aa649c0e26cda5da9) - GitHub
* **dev**: add `ENVIRONMENT_AUTOPULL` override to Makefile (#18446) [bc6c421](https://github.com/answerbook/vector/commit/bc6c421da92e23d1d2c76853c5c36c9387a7979a) - GitHub
* **dev**: add environment networking overrides to Makefile (#18654) [89697d1](https://github.com/answerbook/vector/commit/89697d102793a4786c1ad61f64a90b32769c204a) - GitHub
* **dev**: add networking overrides to website Makefile (#18655) [75ebda0](https://github.com/answerbook/vector/commit/75ebda0826992c5e4b63a882e4425fb5ed1e0dcc) - GitHub
* **dev**: decouple syslog source and codec features (#18381) [aca3a29](https://github.com/answerbook/vector/commit/aca3a296174d38cbe6c4da61e30d8d3033a30060) - GitHub
* disable vrl 'string_path' feature (#18188) [5ce5ff1](https://github.com/answerbook/vector/commit/5ce5ff19365a42afbb857acbcd48636bb2d99194) - GitHub
* **exec source**: add support for customizing command environment (#18223) [e27684c](https://github.com/answerbook/vector/commit/e27684c6e05d3561a96c652fdd8662285c08dcf8) - GitHub
* **metrics**: support Datadog metric origin metadata (#18405) [587c2e7](https://github.com/answerbook/vector/commit/587c2e7d8cbcc833379eb28e6e1d77902e12bede) - GitHub
* **new sink**: add AWS Simple Notification Service `aws_sns` sink (#18141) [7b2bddc](https://github.com/answerbook/vector/commit/7b2bddc26d8ffa51c4f968a50a1b2983b98717f2) - GitHub
* **playground**: Create built.rs with versions and expose versions to the UI (#18424) [ce7da4e](https://github.com/answerbook/vector/commit/ce7da4e3249a9bf450d1b34ddc7a8c40aa9c1ea1) - GitHub
* **route transform**: Add option to enable/disable unmatched output (#18309) [71343bd](https://github.com/answerbook/vector/commit/71343bd91ee6e851b430c69ac27753ba0e41104c) - GitHub


### Miscellaneous

* Merge pull request #390 from answerbook/feature/LOG-18931 [0ce8b8d](https://github.com/answerbook/vector/commit/0ce8b8daf3f78c27abe7f06a7dc216795b872a08) - GitHub [LOG-18931](https://logdna.atlassian.net/browse/LOG-18931)
* Merge branch 'master' into feature/LOG-18931 [e8a7404](https://github.com/answerbook/vector/commit/e8a74044aa06ecd4043ad895f5f22d2529f81463) - Darin Spivey [LOG-18931](https://logdna.atlassian.net/browse/LOG-18931)
* Merge tag 'v0.33.1' into feature/LOG-18931 [c5efad4](https://github.com/answerbook/vector/commit/c5efad4f40ddca17687287d97c2a566209466426) - Darin Spivey [LOG-18931](https://logdna.atlassian.net/browse/LOG-18931) [LOG-18931](https://logdna.atlassian.net/browse/LOG-18931)
* updating the doc, 2 urls were 404 (#18949) [904cb67](https://github.com/answerbook/vector/commit/904cb67bb02a45090605f0015bd79d845e311e6c) - Jesse Szwedko
* [WEB-3464] Adds TrustArc cookie consent banner (#18741) [69b4623](https://github.com/answerbook/vector/commit/69b4623f00b3b19de47748a19415304353f1b046) - Jesse Szwedko
* enhancement (journald source): Add `extra_journalctl_args` option to specify arbitrary command line arguments to journalctl (#18568) [21aec0c](https://github.com/answerbook/vector/commit/21aec0c7b5ce9e1e620c7913818de4357b1da735) - GitHub
* Bump `chrono` to 0.4.27 (#18436) [e9feabd](https://github.com/answerbook/vector/commit/e9feabde6f458471688fdd2bc2402c899e25a400) - GitHub
* fix(codecs) csv encoding quoting bug (#18320) [93ad80d](https://github.com/answerbook/vector/commit/93ad80ded903b03bf65e116ee7028ce0769bc1b2) - GitHub
* 0.32.0.cue typo (#18270) [f124e70](https://github.com/answerbook/vector/commit/f124e70d1cfc29aba56803f3f0cb6ccc30551a8b) - GitHub
* Add notes about `ingress_upstreaminfo` log format for `parse_nginx_log()` function (#18477) [5cfb3e4](https://github.com/answerbook/vector/commit/5cfb3e41c0735d8c5650e17b75508c0e861fe647) - GitHub
* add PGO information (#18369) [eac3cd0](https://github.com/answerbook/vector/commit/eac3cd0eff454f2802cf33ef9b8a485ea44271a6) - GitHub
* **aws provider**: Use FIPS endpoints when configured to do so (#18390) [02c1b4c](https://github.com/answerbook/vector/commit/02c1b4c96aa632f1e7d8ff91acad09a581a38e9d) - GitHub
* **ci**: Add protobuf compatibility check (pt 1) (#18552) [ddb5195](https://github.com/answerbook/vector/commit/ddb519508f5dc929ca19da3ac6706f6949167f46) - GitHub
* **ci**: protobuf compatibility check (pt 2) (#18553) [1eb933d](https://github.com/answerbook/vector/commit/1eb933d60adb8353e4bd62842aca38b158c333b9) - GitHub
* **core**: Add CLI arg and env variable to control openssl probing (#18229) [a4d73ca](https://github.com/answerbook/vector/commit/a4d73ca2cca51f197cf48be7128ba875a8fb5be7) - GitHub
* **core**: default tokio worker threads to `std::thread::available_parallelism()` (#18541) [730bb15](https://github.com/answerbook/vector/commit/730bb151618d0af6d1a39f75e2830c984cffb8db) - GitHub
* **deps**: remove openssl legacy provider flag and update docs (#18609) [a6b1bed](https://github.com/answerbook/vector/commit/a6b1bed163fa52e1a3fc2189ac24f0f355d752cf) - GitHub
* Editorial edits for updated component descriptions (#18590) [d179c57](https://github.com/answerbook/vector/commit/d179c57d0ddd09fbf6547941883c39c12a327b3a) - GitHub
* **enterprise**: configurable app name (#18554) [2e7e7e7](https://github.com/answerbook/vector/commit/2e7e7e7e0a5f0957bdaf4af9bc65bfe32a5f0ac8) - GitHub
* **es sink**: separate aws support in es & prometheus sink (#18288) [e652ea4](https://github.com/answerbook/vector/commit/e652ea4023dd4c07d59489f5343c1cdfc8cbb083) - GitHub
* **file source**: fix some typos (#18401) [a3a1ef0](https://github.com/answerbook/vector/commit/a3a1ef0271cb599492dcf23205b3e9d9322ad435) - GitHub
* Fixed NixOS page (#18396) [89f0f08](https://github.com/answerbook/vector/commit/89f0f088f20876c44e256254410c293100db195c) - GitHub
* **http_server source**: Configurable http response code (#18208) [afdc66e](https://github.com/answerbook/vector/commit/afdc66e08fc46a990e7f0c65a8a1a540de8ef52b) - GitHub
* **kubernetes_logs source**: Expose `oldest_first` (#18376) [dc66566](https://github.com/answerbook/vector/commit/dc665666bcd4d3487ca3684fd2fe41d4415cea52) - GitHub
* **tls**: add new dedicated page for TLS configuration (#18844) [9ca6c7b](https://github.com/answerbook/vector/commit/9ca6c7b186e73605359844db0bb20946bfdc6390) - Jesse Szwedko
* **websocket sink**: Allow any data type depending on configured codec (#18295) [1c303e8](https://github.com/answerbook/vector/commit/1c303e83949e1f2a9feb30176faaf070fb9b55fc) - GitHub


### **BREAKING CHANGES**

* **config:** fix concurrency default & docs (#18651)
* **deps:** remove openssl legacy provider flag and update docs (#18609)
* **datadog_logs sink:** Use `endpoint` config setting consistent with the other datadog_ sinks. (#18497)

## [1.36.2](https://github.com/answerbook/vector/compare/v1.36.1...v1.36.2) (2024-01-05)


### Bug Fixes

* **prometheus**: surface errors in user logs [4336bd2](https://github.com/answerbook/vector/commit/4336bd2ef4f7feb51cce48df754c734fff570963) - Mike Del Tito [LOG-18946](https://logdna.atlassian.net/browse/LOG-18946)


### Miscellaneous

* Merge pull request #388 from answerbook/mdeltito/LOG-18946 [d6aeb96](https://github.com/answerbook/vector/commit/d6aeb96f5ac0ff56aff616ed06b6418cf33d0bac) - GitHub [LOG-18946](https://logdna.atlassian.net/browse/LOG-18946)

## [1.36.1](https://github.com/answerbook/vector/compare/v1.36.0...v1.36.1) (2024-01-05)


### Chores

* Bump vrl dependency [6311551](https://github.com/answerbook/vector/commit/6311551ddbcd3ff4d5bd656ee5e3e6289affc023) - Jorge Bay [LOG-17304](https://logdna.atlassian.net/browse/LOG-17304)

# [1.36.0](https://github.com/answerbook/vector/compare/v1.35.2...v1.36.0) (2024-01-05)


### Features

* make http config provider optionally post a heartbeat payload [73f7d95](https://github.com/answerbook/vector/commit/73f7d95a87e0fead63d7c6b1fd32ba511defb6f7) - Adam Holmberg [LOG-18815](https://logdna.atlassian.net/browse/LOG-18815)


### Miscellaneous

* Merge pull request #385 from answerbook/holmberg/LOG-18815 [090925e](https://github.com/answerbook/vector/commit/090925eaa4be1560b697c5ea2a68baa1da66d171) - GitHub [LOG-18815](https://logdna.atlassian.net/browse/LOG-18815)

## [1.35.2](https://github.com/answerbook/vector/compare/v1.35.1...v1.35.2) (2024-01-05)


### Bug Fixes

* **log_clustering**: Store annotations for changed templates [7dc0095](https://github.com/answerbook/vector/commit/7dc009523bb238e3d79f42bbe91805d416f49f42) - Jorge Bay [LOG-18941](https://logdna.atlassian.net/browse/LOG-18941)

## [1.35.1](https://github.com/answerbook/vector/compare/v1.35.0...v1.35.1) (2024-01-04)


### Bug Fixes

* **filename**: update s3 consolidated filename [1dc3a3e](https://github.com/answerbook/vector/commit/1dc3a3e431f97052f34ffa95b4623e2f589e8d5d) - dominic-mcallister-logdna [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)


### Miscellaneous

* Merge pull request #384 from answerbook/dominic/LOG-18535_filesize_consolidatedname [7e83412](https://github.com/answerbook/vector/commit/7e83412ff5efcd28f84d8d50665da3a1ba5d5544) - GitHub [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)

# [1.35.0](https://github.com/answerbook/vector/compare/v1.34.0...v1.35.0) (2024-01-03)


### Features

* **reduce**:  Add max_events support to the reduce processor [849eb53](https://github.com/answerbook/vector/commit/849eb53a2774932742edf28e3738beb2f592dad8) - Tom Alexander [LOG-18718](https://logdna.atlassian.net/browse/LOG-18718)


### Miscellaneous

* Merge pull request #382 from answerbook/talexander/LOG-18718 [0a5e9ad](https://github.com/answerbook/vector/commit/0a5e9ad08f685e26501db220690fc808006d473d) - GitHub [LOG-18718](https://logdna.atlassian.net/browse/LOG-18718)

# [1.34.0](https://github.com/answerbook/vector/compare/v1.33.0...v1.34.0) (2024-01-03)


### Code Refactoring

* minor optimization to clock enum [b942716](https://github.com/answerbook/vector/commit/b942716320361b93a790539931bb372b117fd5ce) - Dan Hable [LOG-18819](https://logdna.atlassian.net/browse/LOG-18819)


### Features

* **transform**: enhance sliding aggregate processor [8c9b793](https://github.com/answerbook/vector/commit/8c9b79370b3cac93fb7adf5074af1e80e74e233f) - Dan Hable [LOG-18819](https://logdna.atlassian.net/browse/LOG-18819)

# [1.33.0](https://github.com/answerbook/vector/compare/v1.32.1...v1.33.0) (2024-01-03)


### Bug Fixes

* allow empty message_key value in config (#18091) [8a2f8f6](https://github.com/answerbook/vector/commit/8a2f8f67cd23fde5c7a48c07c5f67c67b833c089) - GitHub
* **aws provider**: Don't unwap external_id (#18452) [77d12ee](https://github.com/answerbook/vector/commit/77d12ee88b17d4d71b3609299b356e050afe651a) - Jesse Szwedko
* **azure_blob sink**: Base Content-Type on encoder and not compression (#18184) [4a049d4](https://github.com/answerbook/vector/commit/4a049d4a90a6a994c530140236c7d67e516674e3) - GitHub
* **ci**: add missing env var (#17872) [7e6495c](https://github.com/answerbook/vector/commit/7e6495c7b95e30bd459385ddc760e86dbdbd3f40) - GitHub
* **ci**: address issues in integration test suite workflow (#17928) [8b2447a](https://github.com/answerbook/vector/commit/8b2447a5ade93b3314876f5bc80429d9b6086f80) - GitHub
* **ci**: Drop docker-compose from bootstrap install (#18407) [d9db2e0](https://github.com/answerbook/vector/commit/d9db2e0b6a5085c5b24d7c7458da749a3543fd72) - Jesse Szwedko
* **ci**: fix gardener move blocked to triage on comment (#18126) [93b1945](https://github.com/answerbook/vector/commit/93b19459010575a702a0a5eba7c2bb923bf5baa1) - GitHub
* **codecs**: Move protobuf codec options under a `protobuf` key (#18111) [36788d1](https://github.com/answerbook/vector/commit/36788d13bd9f87c480c47677d0ca5f2ba400d743) - GitHub
* **component validation**: make tests deterministic through absolute comparisons instead of bounds checks (#17956) [52a8036](https://github.com/answerbook/vector/commit/52a8036722ab5cd4ed92d8916b89d85d6447f8c0) - GitHub
* **config**: Fix TOML parsing of compression levels (#18173) [8fc574f](https://github.com/answerbook/vector/commit/8fc574f98baf0551de8439eaf9ade1a3dea6f37c) - GitHub
* **demo gcp_pubsub internal_metrics source throttle transform**: Fix `interval` fractional second parsing (#17917) [b44a431](https://github.com/answerbook/vector/commit/b44a431bd188ca191b5b9c89d8485010bb2cd747) - GitHub
* **deps**: load default and legacy openssl providers (#18276) [8868b07](https://github.com/answerbook/vector/commit/8868b078ac78f66e62657b034d9d03b551bbebef) - Jesse Szwedko
* **dev**: fix issues when using container tools and `cargo` is not installed locally (#18112) [36111b5](https://github.com/answerbook/vector/commit/36111b5e7f971b336244113762210a486fdd6d0f) - GitHub
* **dev**: fix Rust toolchain check in Makefile (#18218) [f77fd3d](https://github.com/answerbook/vector/commit/f77fd3d2735dbfeda3d9bdaf8f11605e4acd8a33) - GitHub
* **docs, syslog source**: Correct docs for `syslog_ip` (#18003) [a1d3c3a](https://github.com/answerbook/vector/commit/a1d3c3a8488e05dc66f3661ca5ee48a27ca7eb95) - GitHub
* **docs**: add the 'http_client_requests_sent_total' (#18299) [2dcaf30](https://github.com/answerbook/vector/commit/2dcaf302f52206c516422615f0a52ba45fedae8b) - Jesse Szwedko
* make LogEvent index operator test only (#18185) [0c1cf23](https://github.com/answerbook/vector/commit/0c1cf23f4563e0a0beb6e080915da8ef5f78e7e7) - GitHub
* **observability**: add all events that are being encoded (#18289) [c9ccee0](https://github.com/answerbook/vector/commit/c9ccee0fdcc516af3555e498f8366c3059f1c74d) - Jesse Szwedko
* **opentelemetry source**: Remove the 4MB default for gRPC request decoding (#18306) [56177eb](https://github.com/answerbook/vector/commit/56177ebce2797c0015c49775e6fdffd4153cc26f) - Jesse Szwedko
* propagate and display invalid JSON errors in VRL web playground (#17826) [8519cb1](https://github.com/answerbook/vector/commit/8519cb1f25a8d83dc014452db5cbdf6b08ee9c9e) - GitHub
* propagate config build error instead of panicking (#18124) [8022464](https://github.com/answerbook/vector/commit/8022464f8ae08b68b3ae571a90fdf50ca6822973) - GitHub
* **reload**: restart api server based on topology (#17958) [b00727e](https://github.com/answerbook/vector/commit/b00727ee13cc4eef6dde63bb8eaa8e0a570294ce) - GitHub
* **spelling**: add spell check exception (#17906) [c4827e4](https://github.com/answerbook/vector/commit/c4827e42a9bfe0f2ef2e0249593d39663ff2a490) - GitHub
* **splunk_hec source**: insert fields as event_path so names aren't parsed as a path (#17943) [1acf5b4](https://github.com/answerbook/vector/commit/1acf5b47802bc83b4ded4bf2daf0c91f5502fb1b) - GitHub
* **syslog source, docs**: Fix docs for `host` field for syslog source (#18453) [dd460a0](https://github.com/answerbook/vector/commit/dd460a0bf91d210e262b1953a6afcaf3aa8f3033) - Jesse Szwedko
* **vdev**: Add `--features` with default features for vdev test (#17977) [eb4383f](https://github.com/answerbook/vector/commit/eb4383fce9e539bd72eb711bd825d542afb20cec) - GitHub
* **vector sink**: Add DataLoss error code as non-retryable (#17904) [4ef0b17](https://github.com/answerbook/vector/commit/4ef0b1778923567c8aa755e28d9419c52b6bc97c) - GitHub
* **vector sink**: cert verification with proxy enabled (#17651) [45e24c7](https://github.com/answerbook/vector/commit/45e24c73e78d3daf609103635950245dcc715444) - GitHub
* **vector source**: Remove the 4MB default for requests (#18186) [4cc9cdf](https://github.com/answerbook/vector/commit/4cc9cdf04cbd2e25426ca3283b76c5b3eee93565) - GitHub
* **website**: Fix installer list for MacOS (#18364) [3b9144c](https://github.com/answerbook/vector/commit/3b9144cb411ea91446c445324db714908ccb814a) - Jesse Szwedko
* **websocket sink**: send encoded message as binary frame (#18060) [b85f4f9](https://github.com/answerbook/vector/commit/b85f4f9cda826e08767c69dcffde04ffad977932) - GitHub


### Chores

* Add licenses to packages (#18006) [db9e47f](https://github.com/answerbook/vector/commit/db9e47fef445ece5c86d786c3cf96049d8f6ee6b) - GitHub
* add more direct regression case for s3 sink (#18082) [c592cb1](https://github.com/answerbook/vector/commit/c592cb17dc4fd153804335c4b315f43d22f0bceb) - GitHub
* added sink review checklist (#17799) [7f45949](https://github.com/answerbook/vector/commit/7f459493d48165818ab8c0796ecea25742131703) - GitHub
* **api**: Refactor top and tap for library use (#18129) [600f819](https://github.com/answerbook/vector/commit/600f8191a8fe169eb38c429958dd59714349acb4) - GitHub
* **aws provider, external_docs**: Update the AWS authentication documentation (#18492) [9356c56](https://github.com/answerbook/vector/commit/9356c56b86817fdca931168986b3e9c88aea1be9) - Jesse Szwedko
* **azure_monitor_logs sink**: refactor to new sink style (#18172) [0aeb143](https://github.com/answerbook/vector/commit/0aeb143cd8012e17f125569e84b968228ec4b4a1) - GitHub
* **CI**: Add missing `--use-consignor` flag on `smp` call (#17966) [7cae000](https://github.com/answerbook/vector/commit/7cae0007f9a4fd1d570a751fa84f0e31e46ead4e) - GitHub
* **ci**: Bump docker/setup-buildx-action from 2.8.0 to 2.9.0 (#17907) [251c4c4](https://github.com/answerbook/vector/commit/251c4c4608a70fd6c112ecacd0517c301f21e33c) - GitHub
* **ci**: Bump docker/setup-buildx-action from 2.9.0 to 2.9.1 (#17955) [77ffce8](https://github.com/answerbook/vector/commit/77ffce8a47faeae64ca8d8eb6642c66f25f15c35) - GitHub
* **ci**: check for team membership on secret-requiring int tests (#17909) [9765809](https://github.com/answerbook/vector/commit/976580949148191ea6faabc7d77ddd60b3c33782) - GitHub
* **ci**: exclude protobuf files from spell checking (#18152) [34eaf43](https://github.com/answerbook/vector/commit/34eaf43d37b51703510045890bbb279d7e0bf78e) - GitHub
* **ci**: Feature branch should be checked against `CURRENT_BRANCH` [4742c2f](https://github.com/answerbook/vector/commit/4742c2f6912f695d0dbf848d5ec22c4f447f4399) - Darin Spivey [LOG-18882](https://logdna.atlassian.net/browse/LOG-18882)
* **ci**: fix gardener issues comment workflow (#17868) [e9f21a9](https://github.com/answerbook/vector/commit/e9f21a98b9f17035fb971f3f95476ec37d9bbe56) - GitHub
* **ci**: fix gardener issues comment workflow pt 2 (#17886) [57ea2b3](https://github.com/answerbook/vector/commit/57ea2b3936c294b1b8b5911fd5f3742231147ea7) - GitHub
* **ci**: fix gardener issues comment workflow pt 3 (#17903) [98ca627](https://github.com/answerbook/vector/commit/98ca6271cbc7c8c4fdabe309a2bf74f3eaca145a) - GitHub
* **ci**: Fix integration test filter generation (#17914) [528fac3](https://github.com/answerbook/vector/commit/528fac3d5155815e59563f01a10c6abcc6802006) - GitHub
* **ci**: fix k8s validate comment job logic (#17841) [99502bb](https://github.com/answerbook/vector/commit/99502bb3d7e9377b6e244d2eb248693c295c0386) - GitHub
* **ci**: remove kinetic as it's no longer supported (#18540) [beb74c1](https://github.com/answerbook/vector/commit/beb74c1c234a2b7d9751cf0bea8a77aff609c604) - Jesse Szwedko
* **ci**: Remove path filter that runs all integration tests (#17908) [70632b7](https://github.com/answerbook/vector/commit/70632b7d980a0721bec83124390eca3604baf2ee) - GitHub
* **ci**: save time int test workflow merge queue (#17869) [9581b35](https://github.com/answerbook/vector/commit/9581b35675ea89bc8fa016b451b948b18a9d19e1) - GitHub
* **ci**: Set HOMEBREW_NO_INSTALL_FROM_API in CI (#17867) [36174e2](https://github.com/answerbook/vector/commit/36174e240e10d42150df84b1d357ec559178e372) - GitHub
* **CI**: Single Machine Performance: turn off consignor (#17967) [1dfc3e1](https://github.com/answerbook/vector/commit/1dfc3e16ccfbce88ad14034e755144bfc7374544) - GitHub
* **CI**: Switch regression detector to new API and analysis service (#17912) [f808ea2](https://github.com/answerbook/vector/commit/f808ea25772f6e888774dc5ab7c77b753c477588) - GitHub
* **CI**: Update `smp` to version 0.9.1 (#17964) [98e47c1](https://github.com/answerbook/vector/commit/98e47c183dff1a23e7207cd77652524a9ca704d5) - GitHub
* **ci**: Use GitHub App token for team membership rather than user PAT (#17936) [7774c49](https://github.com/answerbook/vector/commit/7774c495b7ab4d014a16dc036b284a5b723dc19b) - GitHub
* **codecs**: Update syslog_loose to properly handle escapes (#18114) [b009e4d](https://github.com/answerbook/vector/commit/b009e4d72c7cf0864e5cd5dcb6a392e6559db786) - GitHub
* **core**: Expose shutdown errors (#18153) [cd8c8b1](https://github.com/answerbook/vector/commit/cd8c8b18eed10ccb59e6929d7ee30feac2a6ec25) - GitHub
* **deps**: Bump `nkeys` to 0.3.2 (#18264) [a1dfd54](https://github.com/answerbook/vector/commit/a1dfd54b6947f7766756e5eb24f5b6e1bcc46c98) - Jesse Szwedko
* **deps**: Bump anyhow from 1.0.71 to 1.0.72 (#17986) [9a6ffad](https://github.com/answerbook/vector/commit/9a6ffad33b128b37dae15dc161529112be19f6bc) - GitHub
* **deps**: Bump apache-avro from 0.14.0 to 0.15.0 (#17931) [d5b7fe6](https://github.com/answerbook/vector/commit/d5b7fe6ab070ae85b23d3959aa18b218d2e968a4) - GitHub
* **deps**: Bump assert_cmd from 2.0.11 to 2.0.12 (#17982) [fde77bd](https://github.com/answerbook/vector/commit/fde77bdd9c3acbbf84309b9dcd49d65eea394517) - GitHub
* **deps**: Bump async_graphql, async_graphql_warp from 5.0.10 to 6.0.0 (#18122) [7df6af7](https://github.com/answerbook/vector/commit/7df6af7cf5866e2f49b657b5ae3ec54521810e32) - GitHub
* **deps**: Bump async-compression from 0.4.0 to 0.4.1 (#17932) [5b1219f](https://github.com/answerbook/vector/commit/5b1219f17cb87c6e454f78011b666447d26e2cfd) - GitHub
* **deps**: Bump async-trait from 0.1.68 to 0.1.71 (#17881) [53b2854](https://github.com/answerbook/vector/commit/53b2854d95b2c4d06af0573ff9e02020e46653c5) - GitHub
* **deps**: Bump async-trait from 0.1.71 to 0.1.72 (#18053) [bbe2c74](https://github.com/answerbook/vector/commit/bbe2c74de044cf33ce0cd371c6bfff00c1f285ad) - GitHub
* **deps**: Bump async-trait from 0.1.72 to 0.1.73 (#18235) [20fa1bf](https://github.com/answerbook/vector/commit/20fa1bfc7d4edcb67d26d798249abdd767ba2b72) - GitHub
* **deps**: Bump axum from 0.6.18 to 0.6.19 (#18002) [52ac10a](https://github.com/answerbook/vector/commit/52ac10ac23fe120ac3e1c89ec3196be9ac894009) - GitHub
* **deps**: Bump axum from 0.6.19 to 0.6.20 (#18154) [0ddd221](https://github.com/answerbook/vector/commit/0ddd221f4f657801955102aba76c7f36db68f9fe) - GitHub
* **deps**: Bump bitmask-enum from 2.1.0 to 2.2.0 (#17833) [fc62e9c](https://github.com/answerbook/vector/commit/fc62e9c80f63e77fa8ca8113e952b791db48dd86) - GitHub
* **deps**: Bump bitmask-enum from 2.2.0 to 2.2.1 (#17921) [6326f37](https://github.com/answerbook/vector/commit/6326f372c00431544a2f18456bab72188c1c0be9) - GitHub
* **deps**: Bump bitmask-enum from 2.2.1 to 2.2.2 (#18236) [851e99c](https://github.com/answerbook/vector/commit/851e99ca77ade46fe2a01320db8d16e6bf610c00) - GitHub
* **deps**: Bump bstr from 1.5.0 to 1.6.0 (#17877) [17ccc56](https://github.com/answerbook/vector/commit/17ccc56fadae5009541063b3780c603e945e38a1) - GitHub
* **deps**: Bump clap from 4.3.19 to 4.3.21 (#18178) [0ae3d51](https://github.com/answerbook/vector/commit/0ae3d513711491fd50037a40b4741e3e1a52773d) - GitHub
* **deps**: Bump clap_complete from 4.3.1 to 4.3.2 (#17878) [2126707](https://github.com/answerbook/vector/commit/21267073cc957e9fb72a4fdf8f1e6b246344b0a9) - GitHub
* **deps**: Bump colored from 2.0.0 to 2.0.4 (#17876) [93f8144](https://github.com/answerbook/vector/commit/93f81443d28524c47d17e42167208d7f44e8e7a0) - GitHub
* **deps**: Bump console-subscriber from 0.1.9 to 0.1.10 (#17844) [f74d5dd](https://github.com/answerbook/vector/commit/f74d5dd39758eeb1adfb146dc517e3b3b7e1fda4) - GitHub
* **deps**: Bump darling from 0.20.1 to 0.20.3 (#17969) [656b1fe](https://github.com/answerbook/vector/commit/656b1fe18f0750a6c4d705bb29a771251c0a6b88) - GitHub
* **deps**: Bump dashmap from 5.4.0 to 5.5.0 (#17938) [b535d18](https://github.com/answerbook/vector/commit/b535d184f864af5903e2f7f37671371a32aa2ff2) - GitHub
* **deps**: Bump dyn-clone from 1.0.11 to 1.0.12 (#17987) [81de3e5](https://github.com/answerbook/vector/commit/81de3e54bfdbd112b4177db907e542bf540d97b0) - GitHub
* **deps**: Bump enum_dispatch from 0.3.11 to 0.3.12 (#17879) [bf1407c](https://github.com/answerbook/vector/commit/bf1407c158b653fef810f4d8e570c93e47367c1c) - GitHub
* **deps**: Bump gloo-utils from 0.1.7 to 0.2.0 (#18227) [e61c14f](https://github.com/answerbook/vector/commit/e61c14fdf8111530324878235deedb33526bb897) - GitHub
* **deps**: Bump governor from 0.5.1 to 0.6.0 (#17960) [467baab](https://github.com/answerbook/vector/commit/467baab82cab45acc84d3f3f962c4fbda4f3f632) - GitHub
* **deps**: Bump indicatif from 0.17.5 to 0.17.6 (#18146) [a7c95dd](https://github.com/answerbook/vector/commit/a7c95ddf287fb3f97f41cb662d07113ed5ddec73) - GitHub
* **deps**: Bump indoc from 2.0.1 to 2.0.2 (#17843) [ed5bc3a](https://github.com/answerbook/vector/commit/ed5bc3afb2edb577c80bfdd6f0d7b11cf6f58b99) - GitHub
* **deps**: Bump indoc from 2.0.2 to 2.0.3 (#17996) [3c25758](https://github.com/answerbook/vector/commit/3c257589dac737fcc245485d860b12b5ba7b2830) - GitHub
* **deps**: Bump infer from 0.14.0 to 0.15.0 (#17860) [97f4433](https://github.com/answerbook/vector/commit/97f4433f4689211877cf3042b5aaf14e38a32020) - GitHub
* **deps**: Bump inventory from 0.3.10 to 0.3.11 (#18070) [d8f211e](https://github.com/answerbook/vector/commit/d8f211eaa2b9d5c27089c17dfbbd762de167a988) - GitHub
* **deps**: Bump inventory from 0.3.6 to 0.3.8 (#17842) [bf2f975](https://github.com/answerbook/vector/commit/bf2f97554f3696b0716210013e6dfde0bddbc958) - GitHub
* **deps**: Bump inventory from 0.3.8 to 0.3.9 (#17995) [9c59fea](https://github.com/answerbook/vector/commit/9c59feaf08336123ff45a66b0cfa115523c010aa) - GitHub
* **deps**: Bump inventory from 0.3.9 to 0.3.10 (#18064) [684e43f](https://github.com/answerbook/vector/commit/684e43f5bb2be30a1bb63a742dbc6f6215604f37) - GitHub
* **deps**: Bump lapin from 2.2.1 to 2.3.1 (#17974) [38719a3](https://github.com/answerbook/vector/commit/38719a3b459fa9bf34552edc7deaf3a023b5257a) - GitHub
* **deps**: Bump log from 0.4.19 to 0.4.20 (#18237) [cb007fe](https://github.com/answerbook/vector/commit/cb007fea2ee81882943dec0c52e2883bb5a9de86) - GitHub
* **deps**: Bump lru from 0.10.1 to 0.11.0 (#17945) [4d4b393](https://github.com/answerbook/vector/commit/4d4b393e1eb9ad02b5c1bfad41d5317e6f26b09a) - GitHub
* **deps**: Bump metrics from 0.21.0 to 0.21.1 (#17836) [c8e1267](https://github.com/answerbook/vector/commit/c8e12672ffce5ba0ad1a948f0bcabf74ffab8f93) - GitHub
* **deps**: Bump metrics-util from 0.15.0 to 0.15.1 (#17835) [f91d1b2](https://github.com/answerbook/vector/commit/f91d1b204e3fd2ef4a464ba354aa6bb277e6a0a5) - GitHub
* **deps**: Bump nkeys from 0.3.0 to 0.3.1 (#18056) [087a0ac](https://github.com/answerbook/vector/commit/087a0ace58867c6152317360717e3c97f8e143be) - GitHub
* **deps**: Bump no-proxy from 0.3.2 to 0.3.3 (#18094) [9458b6c](https://github.com/answerbook/vector/commit/9458b6c63d5fa069ab4cb956c7044eb9f74ebfbe) - GitHub
* **deps**: Bump num-traits from 0.2.15 to 0.2.16 (#18039) [4de89f2](https://github.com/answerbook/vector/commit/4de89f23e7ead95e96d82334bd0815ce33359927) - GitHub
* **deps**: Bump opendal from 0.38.0 to 0.38.1 (#17999) [90f494c](https://github.com/answerbook/vector/commit/90f494c6b2a0df5c4d3c41aa94ed60fc8e219841) - GitHub
* **deps**: Bump OpenSSL base version to 3.1.* (#17669) [8454a6f](https://github.com/answerbook/vector/commit/8454a6f46099e95f6aef41a0830cda6bb3b22b0e) - GitHub
* **deps**: Bump openssl from 0.10.55 to 0.10.56 (#18170) [09610b3](https://github.com/answerbook/vector/commit/09610b3d8a998ca51db2823e4d39fd41071f385e) - GitHub
* **deps**: Bump paste from 1.0.12 to 1.0.13 (#17846) [51d8497](https://github.com/answerbook/vector/commit/51d849760824066f7ead64dc193831a5f85bdc14) - GitHub
* **deps**: Bump paste from 1.0.13 to 1.0.14 (#17991) [a36d36e](https://github.com/answerbook/vector/commit/a36d36e862a598a5f825b034f97971e6d7967ba7) - GitHub
* **deps**: Bump pin-project from 1.1.1 to 1.1.2 (#17837) [17e6632](https://github.com/answerbook/vector/commit/17e6632739182cc03497d9711a0470656c848338) - GitHub
* **deps**: Bump pin-project from 1.1.2 to 1.1.3 (#18169) [e125eee](https://github.com/answerbook/vector/commit/e125eee58eab3660dc203ff92653e7bd10229845) - GitHub
* **deps**: Bump proc-macro2 from 1.0.63 to 1.0.64 (#17922) [22b6c2b](https://github.com/answerbook/vector/commit/22b6c2b9fa6c68b3ab7bbbc2b521c212eef66493) - GitHub
* **deps**: Bump proc-macro2 from 1.0.64 to 1.0.66 (#17989) [fbc0308](https://github.com/answerbook/vector/commit/fbc03080515a8f14492b80c92b2fa5b38c62d639) - GitHub
* **deps**: Bump quote from 1.0.29 to 1.0.31 (#17990) [6e552f0](https://github.com/answerbook/vector/commit/6e552f01449bc55572314fa5d4853662126e538d) - GitHub
* **deps**: Bump quote from 1.0.31 to 1.0.32 (#18069) [dc2348a](https://github.com/answerbook/vector/commit/dc2348a8028c399ffef8939ad27161a7e5c62ef2) - GitHub
* **deps**: Bump rdkafka from 0.32.2 to 0.33.2 (#17891) [c8deeda](https://github.com/answerbook/vector/commit/c8deedab78cf45df70edb6ad8ee85fff6e888511) - GitHub
* **deps**: Bump redis from 0.23.0 to 0.23.1 (#18107) [48abad4](https://github.com/answerbook/vector/commit/48abad44407f36c494611a87a6698d909eb8a839) - GitHub
* **deps**: Bump redis from 0.23.1 to 0.23.2 (#18234) [ec3b440](https://github.com/answerbook/vector/commit/ec3b4401bb839d207b30ab9561533c958dcc4f99) - GitHub
* **deps**: Bump regex from 1.8.4 to 1.9.0 (#17874) [cb950b0](https://github.com/answerbook/vector/commit/cb950b0446b48f3c894a5913f7d4c416f0cbc47e) - GitHub
* **deps**: Bump regex from 1.9.0 to 1.9.1 (#17915) [bc5822c](https://github.com/answerbook/vector/commit/bc5822c5017ecad6d59f720f3f874142287f3c6a) - GitHub
* **deps**: Bump regex from 1.9.1 to 1.9.3 (#18167) [00037b0](https://github.com/answerbook/vector/commit/00037b075e1842139eb5a6c97eabfb09042c95e7) - GitHub
* **deps**: Bump rmp-serde from 1.1.1 to 1.1.2 (#18054) [497fdce](https://github.com/answerbook/vector/commit/497fdcede4ae828a00574c496122752b2a70e89c) - GitHub
* **deps**: Bump roaring from 0.10.1 to 0.10.2 (#18079) [f6c53d0](https://github.com/answerbook/vector/commit/f6c53d035e5c8d2c655c7c8b7ad82f7f341f6862) - GitHub
* **deps**: Bump ryu from 1.0.13 to 1.0.14 (#17848) [4613b36](https://github.com/answerbook/vector/commit/4613b36284781d442728c05468ada320a92f71c0) - GitHub
* **deps**: Bump ryu from 1.0.14 to 1.0.15 (#17993) [f53c687](https://github.com/answerbook/vector/commit/f53c6877eaf7c794b906f7f06ea3c1ab67c223f6) - GitHub
* **deps**: Bump schannel from 0.1.21 to 0.1.22 (#17850) [ae59be6](https://github.com/answerbook/vector/commit/ae59be62a3f8a87d5c12acbc8d60ed01b92e2ea3) - GitHub
* **deps**: Bump security-framework from 2.9.1 to 2.9.2 (#18051) [b305334](https://github.com/answerbook/vector/commit/b305334b99a2d3cefcc0dd48e6e60b371645a24d) - GitHub
* **deps**: Bump semver from 1.0.17 to 1.0.18 (#17998) [ca368d8](https://github.com/answerbook/vector/commit/ca368d8c6b9d67d79efe059336770522e410e057) - GitHub
* **deps**: Bump semver from 5.7.1 to 5.7.2 in /website (#17937) [784f3fe](https://github.com/answerbook/vector/commit/784f3fed15c76e9c1416726e595c22ebd2c070f1) - GitHub
* **deps**: Bump serde from 1.0.167 to 1.0.168 (#17920) [3989791](https://github.com/answerbook/vector/commit/39897919be2402c13284bab27125b2b8a62225a6) - GitHub
* **deps**: Bump serde from 1.0.168 to 1.0.171 (#17976) [66f4838](https://github.com/answerbook/vector/commit/66f483874b137c786765e2f8635f7a74b76c7c1a) - GitHub
* **deps**: Bump serde from 1.0.171 to 1.0.173 (#18032) [b36c531](https://github.com/answerbook/vector/commit/b36c5311c7f5787ea0770e83df3ce3ae5c7a7e0b) - GitHub
* **deps**: Bump serde from 1.0.173 to 1.0.174 (#18050) [437cad6](https://github.com/answerbook/vector/commit/437cad6fcc99266c92aa228269787e3b18a79c45) - GitHub
* **deps**: Bump serde from 1.0.174 to 1.0.175 (#18071) [16a42ed](https://github.com/answerbook/vector/commit/16a42ed29c832a39021b2822072f8a67d72ce7a8) - GitHub
* **deps**: Bump serde from 1.0.175 to 1.0.180 (#18127) [e6f2ccc](https://github.com/answerbook/vector/commit/e6f2cccc9dcb93d537dfa2aad5741a6c1c7bac6a) - GitHub
* **deps**: Bump serde from 1.0.180 to 1.0.181 (#18155) [2c51c5c](https://github.com/answerbook/vector/commit/2c51c5c5a0daf75803cd417781bd0e318d1ab9da) - GitHub
* **deps**: Bump serde from 1.0.181 to 1.0.183 (#18171) [6036d5c](https://github.com/answerbook/vector/commit/6036d5c8235dad865c1f32374726a618543bd046) - GitHub
* **deps**: Bump serde_bytes from 0.11.11 to 0.11.12 (#17988) [04f9ddc](https://github.com/answerbook/vector/commit/04f9ddce818f8f09499824be166ff7313a533e0e) - GitHub
* **deps**: Bump serde_bytes from 0.11.9 to 0.11.11 (#17898) [b262316](https://github.com/answerbook/vector/commit/b2623165d0bb0f8732020e3bbd27db197cd780c1) - GitHub
* **deps**: Bump serde_json from 1.0.100 to 1.0.102 (#17948) [4a377a7](https://github.com/answerbook/vector/commit/4a377a79f184c1f09ca5d516712257101a838a2b) - GitHub
* **deps**: Bump serde_json from 1.0.102 to 1.0.103 (#17992) [0ebe7a7](https://github.com/answerbook/vector/commit/0ebe7a7e0db26a4b88f9b3d3cabd35cf0279b810) - GitHub
* **deps**: Bump serde_json from 1.0.103 to 1.0.104 (#18095) [00ed120](https://github.com/answerbook/vector/commit/00ed120317b8673952fec5b8bad3baba482854f7) - GitHub
* **deps**: Bump serde_json from 1.0.99 to 1.0.100 (#17859) [1a427ed](https://github.com/answerbook/vector/commit/1a427ed2d33bfeefb2d3cbec814e3ab7a46d6e5e) - GitHub
* **deps**: Bump serde_with from 3.0.0 to 3.1.0 (#18004) [39a2bf5](https://github.com/answerbook/vector/commit/39a2bf56e4d8bdf23caedb177ad6c25ac439c28d) - GitHub
* **deps**: Bump serde_with from 3.1.0 to 3.2.0 (#18162) [be551c8](https://github.com/answerbook/vector/commit/be551c8c231d6c874edad0de5bdd9c14e6bdfb63) - GitHub
* **deps**: Bump serde_yaml from 0.9.22 to 0.9.24 (#18007) [3b91662](https://github.com/answerbook/vector/commit/3b9166249742a9dc114235550995eecf25288e64) - GitHub
* **deps**: Bump serde_yaml from 0.9.24 to 0.9.25 (#18040) [7050b7e](https://github.com/answerbook/vector/commit/7050b7ef4b73f0997e3f69be12ec34547f6e6ecb) - GitHub
* **deps**: Bump smallvec from 1.10.0 to 1.11.0 (#17880) [46dc18a](https://github.com/answerbook/vector/commit/46dc18adcd73ebd97f069d58329704192b27e43e) - GitHub
* **deps**: Bump snafu from 0.7.4 to 0.7.5 (#17919) [49714cf](https://github.com/answerbook/vector/commit/49714cfa8a242e7b56acef645d1e82d675c8ffa4) - GitHub
* **deps**: Bump strip-ansi-escapes from 0.1.1 to 0.2.0 (#18203) [8bbe6a6](https://github.com/answerbook/vector/commit/8bbe6a6f0c2a3cd3c97ec0495cbc067c88918264) - GitHub
* **deps**: Bump syn from 2.0.23 to 2.0.25 (#17970) [5dfede4](https://github.com/answerbook/vector/commit/5dfede4784c7a9457d2a15ad51f1ac13bcc6730c) - GitHub
* **deps**: Bump syn from 2.0.25 to 2.0.26 (#17994) [caf6103](https://github.com/answerbook/vector/commit/caf61032dd077516353957cd3959ec34e6333cf1) - GitHub
* **deps**: Bump syn from 2.0.26 to 2.0.27 (#18042) [983a92a](https://github.com/answerbook/vector/commit/983a92a8b7eeab3b262c02557ccc1cbd5f11d75e) - GitHub
* **deps**: Bump syn from 2.0.27 to 2.0.28 (#18117) [d3e5128](https://github.com/answerbook/vector/commit/d3e512881b2f7e7135c4b0fd917ac501815086c4) - GitHub
* **deps**: Bump thiserror from 1.0.40 to 1.0.43 (#17900) [ea0f5b1](https://github.com/answerbook/vector/commit/ea0f5b1e06f1e5e2eb22ef33168ad5ac862aaf63) - GitHub
* **deps**: Bump thiserror from 1.0.43 to 1.0.44 (#18052) [ee2396f](https://github.com/answerbook/vector/commit/ee2396ff926468ed94199a930f3e04db2e7bbd04) - GitHub
* **deps**: Bump tikv-jemallocator from 0.5.0 to 0.5.4 (#18102) [564104e](https://github.com/answerbook/vector/commit/564104eadbe5bcc230497ee22edc37039fd21bb2) - GitHub
* **deps**: Bump to syn 2, serde_with 3, darling 0.20, and serde_derive_internals 0.28 (#17930) [3921a24](https://github.com/answerbook/vector/commit/3921a24e13b6558db8aec29f19fcd68a1601460c) - GitHub
* **deps**: Bump tokio from 1.29.0 to 1.29.1 (#17811) [0454d9d](https://github.com/answerbook/vector/commit/0454d9dd938645af145001362908aa2a3342dc46) - GitHub
* **deps**: Bump tokio from 1.29.1 to 1.30.0 (#18202) [92c2b9c](https://github.com/answerbook/vector/commit/92c2b9cce248c250b962f4a1de1194e14f177ce3) - GitHub
* **deps**: Bump tokio-tungstenite from 0.19.0 to 0.20.0 (#18065) [3968325](https://github.com/answerbook/vector/commit/3968325707f90937e91b0ba12a6dbdae4719854b) - GitHub
* **deps**: Bump toml from 0.7.5 to 0.7.6 (#17875) [44d3a8c](https://github.com/answerbook/vector/commit/44d3a8c9612897029406ba25f563e445ddb367d0) - GitHub
* **deps**: Bump tower-http from 0.4.1 to 0.4.2 (#18030) [9b4cd44](https://github.com/answerbook/vector/commit/9b4cd44d599b08f3459fb1108b86921eb76a355d) - GitHub
* **deps**: Bump tower-http from 0.4.2 to 0.4.3 (#18055) [f1d4196](https://github.com/answerbook/vector/commit/f1d4196d295ae0e188ab4b2ca9ea2e4165467745) - GitHub
* **deps**: Bump typetag from 0.2.10 to 0.2.11 (#18048) [5bccafe](https://github.com/answerbook/vector/commit/5bccafe44931a12695f7ab0ba20e177a65fb2454) - GitHub
* **deps**: Bump typetag from 0.2.11 to 0.2.12 (#18066) [b70074c](https://github.com/answerbook/vector/commit/b70074cb73cb03a44909d80295828c46fc74f4de) - GitHub
* **deps**: Bump typetag from 0.2.8 to 0.2.9 (#17882) [b10d070](https://github.com/answerbook/vector/commit/b10d0709b6d1746fe481f6299f1e5c8518489cfa) - GitHub
* **deps**: Bump typetag from 0.2.9 to 0.2.10 (#17968) [f4b1111](https://github.com/answerbook/vector/commit/f4b11115c2245836d2bc607b07b2556e012871d3) - GitHub
* **deps**: Bump uuid from 1.4.0 to 1.4.1 (#18001) [60e765d](https://github.com/answerbook/vector/commit/60e765db182c568380849fc50396101f2b5476e9) - GitHub
* **deps**: Bump zstd from 0.12.3+zstd.1.5.2 to 0.12.4 (#18031) [752056c](https://github.com/answerbook/vector/commit/752056c06ae926b61ab33ea8d53dafd1e4f04f16) - GitHub
* **deps**: Remove an unneeded advisory ignore (#18226) [01295b0](https://github.com/answerbook/vector/commit/01295b0beab4d0a7b13c52515d1120618879dc97) - GitHub
* **deps**: Swap out bloom crate for bloomy (#17911) [d592b0c](https://github.com/answerbook/vector/commit/d592b0cf9d04de440e56a54687fd38bc33f1c3cd) - GitHub
* **deps**: Swap tui crate for ratatui (#18225) [8838faf](https://github.com/answerbook/vector/commit/8838faff9e29dab975580200c571b18b970696c6) - GitHub
* **deps**: Update to Rust 1.71.0 (#18075) [1dd505f](https://github.com/answerbook/vector/commit/1dd505fde140b0d64431346bfc72ee24144b8710) - GitHub
* **deps**: Update tokio-util fork to 0.7.8 (#18078) [421b421](https://github.com/answerbook/vector/commit/421b421bb988335316417c80129014ff80179246) - GitHub
* **deps**: Upgrade debian usages to use bookworm (#18057) [fecca5e](https://github.com/answerbook/vector/commit/fecca5ef183268f0034995a695e3424d8a86fd03) - GitHub
* **deps**: Upgrade to Rust 1.71.1 (#18221) [eaed0a8](https://github.com/answerbook/vector/commit/eaed0a899a22d5ab23ac4eb0ab23cc34280fb5da) - GitHub
* **deps**: Upgrading version of lading used (#18210) [91e48f6](https://github.com/answerbook/vector/commit/91e48f6834ee51ec2492080e8ebc21d380ee5a4b) - GitHub
* **dev**: Fix package install in Tiltfile (#18198) [f39a0e9](https://github.com/answerbook/vector/commit/f39a0e96cf18ffeb225908e21c7255d3d8550898) - GitHub
* **dev**: Install dd-rust-license-tool from crates.io (#18025) [7d0db6b](https://github.com/answerbook/vector/commit/7d0db6bbf33a7bc2e929d5d56b207dce42da4317) - GitHub
* **dev**: Mark loki-logproto crate as unpublished (#17979) [5dd2084](https://github.com/answerbook/vector/commit/5dd208424c4acb6c0cb0dab5b9b5768cc83daf37) - GitHub
* **docs**: Add macOS troubleshooting section to VRL web playground (#17824) [0fbdb33](https://github.com/answerbook/vector/commit/0fbdb335dd1cb9b467cc0280de314463ce108799) - GitHub
* **docs**: Fix links in CONTRIBUTING.md (#18061) [250cc95](https://github.com/answerbook/vector/commit/250cc950b0d3feb27755b614bb3402543195a683) - GitHub
* **docs**: Remove mentions of deprecated transforms from guides (#17933) [37fb02b](https://github.com/answerbook/vector/commit/37fb02ba114e86fa7aeb8f9ae54fc5daf724bc8c) - GitHub
* **external docs**: update sink tutorials with Data Volume tag changes (#18148) [b2d23a8](https://github.com/answerbook/vector/commit/b2d23a838e7b5409273d82afa647b960b24499d3) - GitHub
* Install script supports Apple ARM with Rosetta (#18016) [fd10e69](https://github.com/answerbook/vector/commit/fd10e69a3bb0880798ff3690db08050740e51084) - GitHub
* **observability**: add tests to sinks for Data Volume tags (#17853) [4915b42](https://github.com/answerbook/vector/commit/4915b429a81887736fd1864cd45697f052105277) - GitHub
* **observability**: consolidate `EventCountTags` with `TaggedEventsSent` (#17865) [81f5c50](https://github.com/answerbook/vector/commit/81f5c507793d73a0678968c4a596b213cfa5c619) - GitHub
* **observability**: count byte_size after transforming event (#17941) [0bf6abd](https://github.com/answerbook/vector/commit/0bf6abd03fc92c80f306a20da9825c8298efe041) - GitHub
* **observability**: Fix a couple typos with the registered event cache (#17809) [205300b](https://github.com/answerbook/vector/commit/205300b4bea826d342d68153d0ee542857ee27ca) - GitHub
* **releasing**: Add 0.32.0 highlight for legacy OpenSSL provider deprecation (#18263) [1a32e96](https://github.com/answerbook/vector/commit/1a32e969162d921f00c3ad67c242e8cf047d2c99) - Jesse Szwedko
* **releasing**: Add known issues for v0.32.0 (#18298) [38e95b5](https://github.com/answerbook/vector/commit/38e95b56178197224f3aead2d19050421fdb5464) - Jesse Szwedko
* **releasing**: Add note about protobuf codec addition for 0.32.0 release (#18275) [91f7612](https://github.com/answerbook/vector/commit/91f7612053204f5305ea2991429cf7ccfae4bf26) - Jesse Szwedko
* **releasing**: Add upgrade note for 0.31.0 about S3 path changes (#17934) [f8461cb](https://github.com/answerbook/vector/commit/f8461cbf356fe3c90d7d57a511c97d6fced31e47) - GitHub
* **releasing**: Bump Vector to 0.32.0 (#17887) [9c0d2f2](https://github.com/answerbook/vector/commit/9c0d2f2a9bd0b50c5e1c703f4087b1e297c8ece6) - GitHub
* **releasing**: Fix link in v0.31.0 release docs (#17888) [1260c83](https://github.com/answerbook/vector/commit/1260c83e7e0222bd29f96c0533b6af6147c3c2da) - GitHub
* **releasing**: Fix markdown syntax in minor release template (#17890) [0735ffe](https://github.com/answerbook/vector/commit/0735ffe5b29f8603da9cc5f4fc017015c6529343) - GitHub
* **releasing**: Prepare v0.31.0 release [aeccd26](https://github.com/answerbook/vector/commit/aeccd267c3abccaa8307152db7b9522803a87c17) - Jesse Szwedko
* **releasing**: Prepare v0.32.0 release [1b403e1](https://github.com/answerbook/vector/commit/1b403e1397152a76ee2f600f8d069561c6ec98f2) - Jesse Szwedko
* **releasing**: Prepare v0.32.1 release [9965884](https://github.com/answerbook/vector/commit/99658840c9f1b4a2414617d0cdaec74fc09c8a22) - Jesse Szwedko
* **releasing**: Prepare v0.32.2 release [0982551](https://github.com/answerbook/vector/commit/0982551331b20fd18bf31354ce0fa3d38f593124) - Jesse Szwedko
* **releasing**: Regenerate k8s manifests with v0.23.0 of the chart (#17892) [604fea0](https://github.com/answerbook/vector/commit/604fea0dcf54034dfab1ffcc27c12f0883c704e6) - GitHub
* **releasing**: Run hadolint on distributed Dockerfiles (#18224) [ad08d01](https://github.com/answerbook/vector/commit/ad08d010fbeb2df02e38433064916d8ee8bc37b3) - GitHub
* replace path tuples with actual target paths (#18139) [8068f1d](https://github.com/answerbook/vector/commit/8068f1d115666adafb95dac50ecc2a8879f1af8a) - GitHub
* replace various string paths with actual paths (#18109) [d8eefe3](https://github.com/answerbook/vector/commit/d8eefe331af0faa478b9fe2f58de2a25a83589e9) - GitHub
* **security**: Make the warning for the deprecated OpenSSL provider more verbose (#18278) [042fb51](https://github.com/answerbook/vector/commit/042fb51dbec93c1e1b644735ab749b9711c2e4c8) - Jesse Szwedko
* separate hanwritten and generated files in web-playground (#17871) [9ec0443](https://github.com/answerbook/vector/commit/9ec04438c9b59bc8ab8d4988c9f5744ad61c7248) - GitHub
* stop ignoring topology test (#17953) [a05542a](https://github.com/answerbook/vector/commit/a05542a1e392f0e18c8b305afd4d56bc146b6102) - GitHub
* update `rustls-webpki` due to security advisory (#18344) [1cb51a4](https://github.com/answerbook/vector/commit/1cb51a40ccfd648cbb92bff82ba29a97dc617b54) - Jesse Szwedko
* Update `smp` to its latest released version (#18204) [7603d28](https://github.com/answerbook/vector/commit/7603d2813e389a1103286c634d92d9da1e8a8b52) - GitHub


### Features

* **adaptive_concurrency**: support configuring the initial ARC limit (#18175) [3b53bcd](https://github.com/answerbook/vector/commit/3b53bcda04a06b365bc35965e8934eddac1b7fc2) - GitHub
* add support for `external_id` in AWS assume role (#17743) [689a79e](https://github.com/answerbook/vector/commit/689a79e20e0130fd2070be28173fa3ef565b27ac) - GitHub
* **clickhouse sink**: make `database` and `table` templateable (#18005) [536a7f1](https://github.com/answerbook/vector/commit/536a7f12cbeef373979f845ef3f1b565463cbccd) - GitHub
* **codecs**: add support for protobuf decoding (#18019) [a06c711](https://github.com/answerbook/vector/commit/a06c71102867af5e4526e445f9ba8f4506382a30) - GitHub
* **component validation**: validate `component_errors_total` for sources (#17965) [aa60520](https://github.com/answerbook/vector/commit/aa605206baaa6db0506ed0698cfd14847abbb5a9) - GitHub
* **deps, vrl**: Update VRL to 0.6.0 (#18150) [adfef2e](https://github.com/answerbook/vector/commit/adfef2eeca6e4047e372e530109d640e55b38478) - GitHub
* emit an error if the condition return type is not a boolean (#18196) [caf6103](https://github.com/answerbook/vector/commit/caf6103f76ce7cd913129f64e0d5c5d17bdbc799) - GitHub
* LogSchema metadata key refacoring (#18099) [a8bb9f4](https://github.com/answerbook/vector/commit/a8bb9f45867ab2435640258ad07babc1d0b8f747) - GitHub
* Migrate `LogSchema` `source_type_key` to new lookup code (#17947) [d29424d](https://github.com/answerbook/vector/commit/d29424d95dbc7c9afd039890df38681ba309853f) - GitHub
* Migrate LogSchema::host_key to new lookup code (#17972) [32950d8](https://github.com/answerbook/vector/commit/32950d8ddb5623637a84103dce5e4f3ac176ab3b) - GitHub
* Migrate LogSchema::message_key to new lookup code (#18024) [0f14c0d](https://github.com/answerbook/vector/commit/0f14c0d02d5f9bc4ed68236d07d74a70eab13c64) - GitHub
* Migrate LogSchema::metadata key to new lookup code (#18058) [8663602](https://github.com/answerbook/vector/commit/86636020f145bae0e0259b78cc9ffa789381505e) - GitHub
* migrate to `async_nats` client (#18165) [483e46f](https://github.com/answerbook/vector/commit/483e46fe4656d3636d6cbff18c2e9f86baa48d68) - GitHub
* **new sink**: Adding greptimedb metrics sink (#17198) [98f44ae](https://github.com/answerbook/vector/commit/98f44ae070ffdba58460e2262e0c70683fad3797) - GitHub
* **new sink**: Initial `datadog_events` sink (#7678) [53fc86a](https://github.com/answerbook/vector/commit/53fc86ae9b230ebd149580e5e5abb659537f9312) - Jesse Szwedko
* Refactor 'event.get()' to use path types (#18160) [e476e12](https://github.com/answerbook/vector/commit/e476e120503d8682c8aef511b7af9b8851f2d03c) - GitHub
* Refactor dnstap to use 'OwnedValuePath's (#18212) [ca7fa05](https://github.com/answerbook/vector/commit/ca7fa05ca98ac8ed097dc7a24b1652d62dbf283a) - GitHub
* Refactor TraceEvent insert to use TargetPath compatible types (#18090) [f015b29](https://github.com/answerbook/vector/commit/f015b299b0249d082f297f7aee15f42ae091c77b) - GitHub
* replace LogEvent 'String's with '&OwnedTargetPath's (#18084) [065eecb](https://github.com/answerbook/vector/commit/065eecbcafd37a99ba8667f69cee78d96bb132e1) - GitHub
* replace tuples with &OwnedTargetPath wherever possible (#18097) [28f5c23](https://github.com/answerbook/vector/commit/28f5c23aa84f70736fe5ef5132e274b3611cceb9) - GitHub
* switch to crates.io release of Azure SDK (#18166) [3c535ec](https://github.com/answerbook/vector/commit/3c535ecc289f2376133c8229ecc8316dbb4806bf) - GitHub


### Miscellaneous

* Merge pull request #379 from answerbook/feature/LOG-18882 [8bd9860](https://github.com/answerbook/vector/commit/8bd9860d42baa3d33d7af7a1ce8364e4e039f2fa) - GitHub [LOG-18882](https://logdna.atlassian.net/browse/LOG-18882)
* Merge branch 'master' into feature/LOG-18882 [d217387](https://github.com/answerbook/vector/commit/d2173872ac97327f3ba5ff5c8323694ba2e6afa5) - Darin Spivey [LOG-18882](https://logdna.atlassian.net/browse/LOG-18882)
* Merge tag 'v0.32.2' into feature/LOG-18882 [c05f969](https://github.com/answerbook/vector/commit/c05f9693084dbd80ca5b8fcffc09802aceb75a01) - Darin Spivey [LOG-18882](https://logdna.atlassian.net/browse/LOG-18882)
* Managed by Terraform provider [92e320a](https://github.com/answerbook/vector/commit/92e320a75f8774956af6370947679df6f2ceda1e) - Terraform
* 0.32.0.cue typo (#18270) [0f7d6e6](https://github.com/answerbook/vector/commit/0f7d6e6798d81bd1cae17c918f53a87406deb383) - Jesse Szwedko
* add PGO information (#18369) [3040ae2](https://github.com/answerbook/vector/commit/3040ae250b36e5dedda6fd635d364cbd77d0fef8) - Jesse Szwedko
* check VRL conditions return type at compile time (#17894) [fa489f8](https://github.com/answerbook/vector/commit/fa489f842b02fd7dd59a58e1339ae264050e92e4) - GitHub
* **ci**: combine build steps for integration test workflows (#17724) [911477a](https://github.com/answerbook/vector/commit/911477a191fe80d68203a3ab7669ce730cc0f43e) - GitHub
* describe the difference between configuration fields and runtime flags (#17784) [01e2dfa](https://github.com/answerbook/vector/commit/01e2dfaa15ce62d156e92d8deac354cd40edf9e7) - GitHub
* **elasticsearch sink**: Allow empty data_stream fields (#18193) [1dd7bb1](https://github.com/answerbook/vector/commit/1dd7bb1e439ef69f1adf2c1ca6dd59fd966b7f2a) - GitHub
* **file source**: fix some typos (#18401) [1164f55](https://github.com/answerbook/vector/commit/1164f5525780a9599864bdda46722e895a20fd4c) - Jesse Szwedko
* Fix "Bring your own toolbox" in `DEVELOPING.md` (#18014) [115bd7b](https://github.com/answerbook/vector/commit/115bd7b4dc4f065a99bb4e3dc464141026e6b3bf) - GitHub
* Fix schema.log_namespace and telemetry.tags documentation (#17961) [50736e2](https://github.com/answerbook/vector/commit/50736e2ed463bef20985329ee5c59d7261b070d8) - GitHub
* **internal docs**: Fix basic sink tutorial issues (#18136) [5a6ce73](https://github.com/answerbook/vector/commit/5a6ce731c999f0960e8411a9b286730314c4e7ac) - GitHub
* **lua transform**: Emit events with the `source_id` set (#17870) [bc1b83a](https://github.com/answerbook/vector/commit/bc1b83ad51a5118aa6a7c3cab62dfb5eb3ce2c91) - GitHub
* **observability**: add fixed tag option to `RegisteredEventCache` (#17814) [bc86222](https://github.com/answerbook/vector/commit/bc86222cd14327fdb459ceb0bb90e522aed3d2b3) - GitHub
* **prometheus_scrape source**: run requests in parallel with timeouts (#18021) [a9df958](https://github.com/answerbook/vector/commit/a9df9589b9ba1869ae354fea48419786fa41468e) - GitHub

## [1.32.1](https://github.com/answerbook/vector/compare/v1.32.0...v1.32.1) (2023-12-22)


### Bug Fixes

* Use buffer ref to account for event size in transforms [9842893](https://github.com/answerbook/vector/commit/9842893dd0db567c2604a64c181dee3833fce55b) - Jorge Bay [LOG-18897](https://logdna.atlassian.net/browse/LOG-18897)

# [1.32.0](https://github.com/answerbook/vector/compare/v1.31.0...v1.32.0) (2023-12-21)


### Features

* **s3-sink**: file consolidation off default [fb46e73](https://github.com/answerbook/vector/commit/fb46e7359b442466069e1fc89d24254821b2a869) - dominic-mcallister-logdna [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)


### Miscellaneous

* Merge pull request #378 from answerbook/dominic/LOG-18535-defaultoff [af67c9e](https://github.com/answerbook/vector/commit/af67c9e1af1bc4bb6d5add3ea88888709f500f38) - GitHub [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)

# [1.31.0](https://github.com/answerbook/vector/compare/v1.30.0...v1.31.0) (2023-12-20)


### Features

* **edge**: enable more local sources [be66c87](https://github.com/answerbook/vector/commit/be66c87ac0faf9cb28490eee52cd849c90213fe5) - Matt March [LOG-18838](https://logdna.atlassian.net/browse/LOG-18838)
* **s3 sink**: add file consolidation [e749861](https://github.com/answerbook/vector/commit/e7498613b8fdcc13ad64f08dad4b6dc02dd8c4d6) - dominic-mcallister-logdna [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)


### Miscellaneous

* Merge pull request #377 from answerbook/dominic/LOG-18535-after31 [b9a5ad3](https://github.com/answerbook/vector/commit/b9a5ad3afad14a0b6a2332913f9d345fa71c46a2) - GitHub [LOG-18535-after31](https://logdna.atlassian.net/browse/LOG-18535-after31)

# [1.30.0](https://github.com/answerbook/vector/compare/v1.29.2...v1.30.0) (2023-12-19)


### Bug Fixes

* `aws_ec2_metadata` transform when using log namespacing (#17819) [4786743](https://github.com/answerbook/vector/commit/4786743dcaa73e16781e8b43ce0a1ce0315a55d1) - GitHub
* **auth**: Vector does not put the Proxy-Authorization header on the wire (#17353) (#17363) [6705bdd](https://github.com/answerbook/vector/commit/6705bdde058b1a532eda9398c9610dff46bb783b) - GitHub
* **buffers**: deadlock when seeking after entire write fails to be flushed (#17657) [37a662a](https://github.com/answerbook/vector/commit/37a662a9c2e388dc1699f90288c5d856381d15d4) - GitHub
* **ci**: add missing env var (#17872) [4f67695](https://github.com/answerbook/vector/commit/4f67695942c7f44f807cc92a43c6d6456fcebd92) - Jesse Szwedko
* **ci**: add missing logic to mark required checks failed (#17543) [3b87e00](https://github.com/answerbook/vector/commit/3b87e00f3a62be93f55a89df676b47a8fad22201) - GitHub
* **ci**: change command to find baseline sha from issue comment trigger (#17622) [5791083](https://github.com/answerbook/vector/commit/579108353e50546081b830d4e5788be7bb76a892) - GitHub
* **ci**: checkout a greater depth in regression workflow (#17604) [baa04e5](https://github.com/answerbook/vector/commit/baa04e59d9b234c4e71f8545a6ad8fdb2517f805) - GitHub
* **ci**: post failed status to PR and isolate branch checkout on comment trigger (#17544) [e2c0255](https://github.com/answerbook/vector/commit/e2c025591c572efdd04728fac301b2e025596516) - GitHub
* **ci**: reg workflow alt approach to getting baseline sha (#17645) [f1e1ae3](https://github.com/answerbook/vector/commit/f1e1ae36ec4f244a03cbc7084cde64ea2d9631fa) - GitHub
* **ci**: use correct ID for Triage in Gardener Board (#17647) [2638cca](https://github.com/answerbook/vector/commit/2638cca6cbf5103f71944383255b3e335d7f5790) - GitHub
* **ci**: use correct secret for gardener board comment (#17605) [9395eba](https://github.com/answerbook/vector/commit/9395eba89ed10488914ac042aabba068356bb84b) - GitHub
* **config**: Fix preloading log_schema (#17759) [659e1e6](https://github.com/answerbook/vector/commit/659e1e69f56d32939871bc097c6eeb0b950012db) - GitHub
* **databend sink**: use get for page request (#17373) [c7d7cf8](https://github.com/answerbook/vector/commit/c7d7cf8e36b9de6de7cd963e472d33b792c24413) - GitHub
* **datadog_agent source**: remove duplicate internal metrics emission (#17720) [48ec2e8](https://github.com/answerbook/vector/commit/48ec2e8bc51f3f4f68566a64e6fe7d7327a73591) - GitHub
* **distribution**: Fix architecture detection for ARMv7 (#17484) [78fb469](https://github.com/answerbook/vector/commit/78fb4694c26d061314e8a01236a67633d8035d5c) - GitHub
* **docs**: fix copy-paste issue in component spec (#17616) [b400acc](https://github.com/answerbook/vector/commit/b400acced6bd61d5927ab75bb82643b5927c0cbd) - GitHub
* **file source**: Fix tailing problem when source number greater than 512 (#17717) [23a3e0e](https://github.com/answerbook/vector/commit/23a3e0ebf44fd8efa46a6861aa91404806be3831) - GitHub
* **fluent source**: fix ack message format (#17407) [d194992](https://github.com/answerbook/vector/commit/d1949921a81181e2eeb1780d7e081d767f758f5e) - GitHub
* **http_client source**: adapt int test to use breaking change of dep (#17583) [d7df520](https://github.com/answerbook/vector/commit/d7df52055152d9f85a6e48082d385e84c45f1501) - GitHub
* **http_client source**: remove utf8 lossy conversion (#17655) [59e2cbf](https://github.com/answerbook/vector/commit/59e2cbff7bce014209813369d2a33a25ac193bb3) - GitHub
* **install.sh**: Correctly `shift` all parsed arguments (#17684) [f883575](https://github.com/answerbook/vector/commit/f88357515c12240ae2a594324253e7f203ea27f9) - GitHub
* **loki sink, observability**: Drop non-fatal template render errors to warnings (#17746) [4ebc3e1](https://github.com/answerbook/vector/commit/4ebc3e1171cba4f00023f0ef860a6b66c98763a9) - GitHub
* **loki sink**: use json size of unencoded event (#17572) [25e7699](https://github.com/answerbook/vector/commit/25e7699bb505e1856d04634ed6571eb22631b140) - GitHub
* **observability**: correct emitted metrics (#17562) [7a4f1f7](https://github.com/answerbook/vector/commit/7a4f1f77470fbc804299e2c1be867b193052d275) - GitHub
* **observability**: issues with event_cache PR (#17768) [fdf02d9](https://github.com/answerbook/vector/commit/fdf02d954286288f435c84395bc9b9be13806899) - GitHub
* remap behavior for root types when using the `Vector` namespace (#17807) [c19938c](https://github.com/answerbook/vector/commit/c19938c9213539da6b4ca6d50554557c87d6fde4) - GitHub
* **sinks**: Add missing component span for sink building (#17765) [219883e](https://github.com/answerbook/vector/commit/219883eb2fc6fb7020b38d9e62d1a4ae0c2ba9e7) - GitHub


### Chores

* Add docker config to dependabot (#17696) [079d895](https://github.com/answerbook/vector/commit/079d895ebffeb62cf51cb11144b17fd481292510) - GitHub
* add sink prelude (#17595) [da939ca](https://github.com/answerbook/vector/commit/da939ca645e49cd02cbd739cddcdfe00dcb88a55) - GitHub
* Add submodules to all checkouts (#17770) [7196622](https://github.com/answerbook/vector/commit/719662280205d47ce9497646368911c8f5c28b0d) - GitHub
* **administration**: add domain label for vdev (#17748) [c35ebd1](https://github.com/answerbook/vector/commit/c35ebd167b029eb0fb6c180301e8ff911f938f9f) - GitHub
* **aws_s3 sink**: Update metadata to match the editorial review for the schema. (#17475) [c1262cd](https://github.com/answerbook/vector/commit/c1262cd162e04550b69913877d6b97037aceaea4) - GitHub
* Bump version to 0.31.0 (#17466) [78bbfbc](https://github.com/answerbook/vector/commit/78bbfbc0205d97b401b5ba3084fe71e2bfdd7f33) - GitHub
* **ci**: Add apt retries to cross builds (#17683) [ab1169b](https://github.com/answerbook/vector/commit/ab1169bd40ff7f1fa8cf1e77d24cd779112b2178) - GitHub
* **ci**: Add schedule to component features workflow conditional check (#17816) [708b7f6](https://github.com/answerbook/vector/commit/708b7f6088c14180945d80e2a8f13ed471ded77a) - GitHub
* **ci**: Bump aws-actions/configure-aws-credentials from 2.0.0 to 2.1.0 (#17565) [8a741d5](https://github.com/answerbook/vector/commit/8a741d55b8bfe361d6c5449cab4fd3728e1dae8d) - GitHub
* **ci**: Bump aws-actions/configure-aws-credentials from 2.1.0 to 2.2.0 (#17697) [12bc4a7](https://github.com/answerbook/vector/commit/12bc4a7d116273cda322fccf41b4e3ea6c333be3) - GitHub
* **ci**: Bump docker/build-push-action from 4.0.0 to 4.1.0 (#17656) [cb9a3a5](https://github.com/answerbook/vector/commit/cb9a3a548877b222afb14159393b8bc7bc3f8518) - GitHub
* **ci**: Bump docker/build-push-action from 4.1.0 to 4.1.1 (#17687) [bce5e65](https://github.com/answerbook/vector/commit/bce5e65d9562983f0094f1b7359775cf17043285) - GitHub
* **ci**: Bump docker/metadata-action from 4.4.0 to 4.5.0 (#17624) [a54a12f](https://github.com/answerbook/vector/commit/a54a12faae72ee64f4ba842746837a4787af5dc2) - GitHub
* **ci**: Bump docker/metadata-action from 4.5.0 to 4.6.0 (#17686) [71273df](https://github.com/answerbook/vector/commit/71273dfc64206dd66290426fe7d65a68afb13d51) - GitHub
* **ci**: Bump docker/setup-buildx-action from 2.5.0 to 2.6.0 (#17625) [15bc42a](https://github.com/answerbook/vector/commit/15bc42a21bed188819da4d12e38d108f2e840202) - GitHub
* **ci**: Bump docker/setup-buildx-action from 2.6.0 to 2.7.0 (#17685) [8006987](https://github.com/answerbook/vector/commit/80069871df7d0809411053435486c604b7b8c15d) - GitHub
* **ci**: Bump docker/setup-buildx-action from 2.7.0 to 2.8.0 (#17786) [dbdff9e](https://github.com/answerbook/vector/commit/dbdff9e8b1df36dd45fb8ef2181926224b5dd294) - GitHub
* **ci**: Bump docker/setup-qemu-action from 2.1.0 to 2.2.0 (#17623) [3005141](https://github.com/answerbook/vector/commit/3005141f2097169a05af418e5f80765468645700) - GitHub
* **ci**: bump myrotvorets/set-commit-status-action from 1.1.6 to 1.1.7 (#17460) [bca45eb](https://github.com/answerbook/vector/commit/bca45eb32bff27429a6beb3cf1d7b241d6de8c70) - GitHub
* **ci**: Bump up OSX runners for release builds (#17823) [fe730ad](https://github.com/answerbook/vector/commit/fe730adee64c45bc9a0737838a8aaa2bd8ef61d8) - GitHub
* **ci**: bump xt0rted/pull-request-comment-branch from 1 to 2 (#17461) [c425006](https://github.com/answerbook/vector/commit/c425006f299c7a5f91509f7bdb18963f4da0748f) - GitHub
* **ci**: correctly validate comment author in k8s e2e job (#17818) [b8e3dbe](https://github.com/answerbook/vector/commit/b8e3dbe1cf55e4d117023531e19891fc8c19ccf9) - GitHub
* **ci**: Drop VRL license exceptions (#17529) [aa01452](https://github.com/answerbook/vector/commit/aa014528ca83bd3f1d17604d8c138ac2d0484074) - GitHub
* **ci**: fix a few logic bugs and more strict comment parsing (#17502) [bf372fd](https://github.com/answerbook/vector/commit/bf372fd7cdef40704205e5fb5bf10bc50e002d94) - GitHub
* **ci**: fix comment author validation (#17794) [75ae967](https://github.com/answerbook/vector/commit/75ae967ebed2231a93b62e2c7a5a08685fa7d654) - GitHub
* **ci**: fix failure notify job conditional in publish workflow (#17468) [3699842](https://github.com/answerbook/vector/commit/36998428099da9b3ce4bcf0fd6f8787be1920363) - GitHub
* **ci**: fix gardener issues comment workflow (#17825) [47c3da1](https://github.com/answerbook/vector/commit/47c3da1f21d3cc3d4af09d321ae3754972e0a150) - GitHub
* **ci**: fix team membership action (#17791) [13c3c78](https://github.com/answerbook/vector/commit/13c3c788e2225ba25ca49500ebde270915c2e7bc) - GitHub
* **ci**: int test yaml file detection (#17590) [fa8a553](https://github.com/answerbook/vector/commit/fa8a55385dd391aa2429c3f2e9821198c364c6a0) - GitHub
* **ci**: minor fixes to workflows post merge queue enabling  (#17462) [9f6f6ec](https://github.com/answerbook/vector/commit/9f6f6ecde0db3ffdd7b904647f490511433836b5) - GitHub
* **ci**: move component features check out of merge queue (#17773) [e6e776d](https://github.com/answerbook/vector/commit/e6e776ddb7c93db243f989413581d9116275939e) - GitHub
* **ci**: Move most CI checks to merge queue (#17340) [060399a](https://github.com/answerbook/vector/commit/060399a4bbef4280d1cea7c04304ed1308504ca0) - GitHub
* **ci**: reduce runner sizing to 4 core and free tier (#17785) [53a575f](https://github.com/answerbook/vector/commit/53a575f21d65bc188324ad3bcd2e89d03bbf548c) - GitHub
* **ci**: remove /ci-run-install comment trigger (#17803) [a3770d8](https://github.com/answerbook/vector/commit/a3770d872708d7f28a1f834d31baa142f1f11ea4) - GitHub
* **ci**: Remove remaining Discord notification (#17805) [ed59f37](https://github.com/answerbook/vector/commit/ed59f37e006d63130413e7a4ed21042f4b90dd0e) - GitHub
* **ci**: Remove upload of config schema (#17740) [ff6a1b4](https://github.com/answerbook/vector/commit/ff6a1b4f06b1e32f3192f2bc391e8ab59f466993) - GitHub
* **ci**: Retry `make check-component-docs` check (#17718) [e8e7e04](https://github.com/answerbook/vector/commit/e8e7e0448f51ed9646c484123fd4953442545c86) - GitHub
* **ci**: revert fix gardener issues comment workflow (#17829) [ee10b8c](https://github.com/answerbook/vector/commit/ee10b8cbae51b9c0bade8d8bd8273a8dbeb3bb58) - GitHub
* **ci**: Set HOMEBREW_NO_INSTALL_FROM_API in CI (#17867) [00cc584](https://github.com/answerbook/vector/commit/00cc584aa43d6e975a118667badd20be4030bb84) - Jesse Szwedko
* **ci**: temporarily disable comment_trigger workflow (#17480) [58d7f3d](https://github.com/answerbook/vector/commit/58d7f3dfb0b57445db931604c6f72d93015da505) - GitHub
* **ci**: temporarily disable flakey `aws_s3` integration test case `handles_errored_status`  (#17455) [8e40b68](https://github.com/answerbook/vector/commit/8e40b6850a57f874476f071d4ec98d699a99a65e) - GitHub
* **ci**: update comment_trigger note about concurrency groups (#17491) [7699f4d](https://github.com/answerbook/vector/commit/7699f4ded19e520258adddd4c628a7a309c52c4e) - GitHub
* **ci**: Update publish workflow test Ubuntu versions (#17781) [20d62f1](https://github.com/answerbook/vector/commit/20d62f11f9bd255185fadd58c79891d730997768) - GitHub
* **clickhouse sink**: refactor to new style (#17723) [77ac63c](https://github.com/answerbook/vector/commit/77ac63c5bd87309b1ddd54e55b933072b40e34ea) - GitHub
* **codecs**: consolidate enum types (#17688) [9c45394](https://github.com/answerbook/vector/commit/9c4539436ecbbf48dc0dd454ea25230d539b2c9b) - GitHub
* Codify flag naming including sentinel values (#17569) [134578d](https://github.com/answerbook/vector/commit/134578db2165b4b522013d0e7d6ac974f9e4e744) - GitHub
* Codify the use of abbreviate time units in config option names (#17582) [8823561](https://github.com/answerbook/vector/commit/8823561a8ad544b4acd29273b466b1a5bd606cc2) - GitHub
* **config**: Convert top-level sinks enum to typetag (#17710) [9cd5404](https://github.com/answerbook/vector/commit/9cd54043fab1e82722adaeeaee290d7084074439) - GitHub
* **config**: Make config schema output ordered (#17694) [9606353](https://github.com/answerbook/vector/commit/960635387235ea270d748038a3a0ddd615813f29) - GitHub
* **config**: Update field labels for commonly used sources and transforms  (#17517) [f523f70](https://github.com/answerbook/vector/commit/f523f70d12053bd8d1d5ceee41c7c843780ded84) - GitHub
* **config**: Update field labels for sinks (#17560) [e1ddd0e](https://github.com/answerbook/vector/commit/e1ddd0e99c0290a645a484c45cc42a391803c6c0) - GitHub
* **config**: Update field labels for the rest of the sources and transforms fields (#17564) [6e45477](https://github.com/answerbook/vector/commit/6e45477ddc27147887346c8d09dd077225ea2ef3) - GitHub
* **datadog_archives sink**: Remove this component (#17749) [53f8bff](https://github.com/answerbook/vector/commit/53f8bff371cdfa96770c03be38ae3c83a497043f) - GitHub
* **datadog_metrics sink**: incrementally encode sketches (#17764) [3f6df61](https://github.com/answerbook/vector/commit/3f6df61c4d90ed9d587c2935d188b5ada2f9ff02) - GitHub
* **datadog_traces sink**: Add additional warning around APM stats for `peer.service` (#17733) [9a899c5](https://github.com/answerbook/vector/commit/9a899c5d7c40a271b17eafec2f840c1bfd082b04) - GitHub
* **deps, releasing**: Update to Alpine 3.18 (#17695) [2263756](https://github.com/answerbook/vector/commit/2263756d0a39cb99d62a826ff0993f461ae80937) - GitHub
* **deps**: Bump async-graphql from 5.0.8 to 5.0.9 (#17486) [077a294](https://github.com/answerbook/vector/commit/077a294d10412552e80c41429f23bd6a4f47724b) - GitHub
* **deps**: Bump async-graphql from 5.0.9 to 5.0.10 (#17619) [2931542](https://github.com/answerbook/vector/commit/29315428b2c93ae0a5682ddb1fb25137b5eb3931) - GitHub
* **deps**: Bump async-graphql-warp from 5.0.8 to 5.0.9 (#17489) [ac81fc1](https://github.com/answerbook/vector/commit/ac81fc1318b229e2b9c6bbcd080af7438afde85a) - GitHub
* **deps**: Bump async-graphql-warp from 5.0.9 to 5.0.10 (#17642) [b3885f6](https://github.com/answerbook/vector/commit/b3885f693ebbdddd338b72bfd594e164d4fa361d) - GitHub
* **deps**: bump aws-sigv4 from 0.55.1 to 0.55.3 (#17481) [2ad5b47](https://github.com/answerbook/vector/commit/2ad5b478f8948d0c3d92197f90100148cebda237) - GitHub
* **deps**: bump base64 from 0.21.0 to 0.21.1 (#17451) [95cbba9](https://github.com/answerbook/vector/commit/95cbba9116f12e1aa3665f89050132a28f9a0327) - GitHub
* **deps**: Bump base64 from 0.21.1 to 0.21.2 (#17488) [f261781](https://github.com/answerbook/vector/commit/f261781b5ce4389fb23017a2d4892c7f16753ad9) - GitHub
* **deps**: bump bstr from 1.4.0 to 1.5.0 (#17453) [7554d9c](https://github.com/answerbook/vector/commit/7554d9c8cc7b9b7134c7879dc941f8f55bc837e2) - GitHub
* **deps**: Bump cached from 0.43.0 to 0.44.0 (#17599) [7a55210](https://github.com/answerbook/vector/commit/7a55210ed814e0c47618905a299eba0d896a0646) - GitHub
* **deps**: Bump chrono to 0.4.26 (#17537) [0dfa09c](https://github.com/answerbook/vector/commit/0dfa09c4a9b7e753802a4fa0700557752e2fc945) - GitHub
* **deps**: Bump chrono-tz from 0.8.2 to 0.8.3 (#17789) [f79947c](https://github.com/answerbook/vector/commit/f79947cf0125468b141ff8bc09d1c2bc6366780e) - GitHub
* **deps**: bump clap_complete from 4.2.3 to 4.3.0 (#17447) [05bf262](https://github.com/answerbook/vector/commit/05bf262536031d199c06d980f47be317c97520ea) - GitHub
* **deps**: Bump clap_complete from 4.3.0 to 4.3.1 (#17586) [8549809](https://github.com/answerbook/vector/commit/854980945e685485388bda2dd8f9cd9ad040029e) - GitHub
* **deps**: bump criterion from 0.4.0 to 0.5.0 (#17477) [84f0ada](https://github.com/answerbook/vector/commit/84f0adac7a8e6306e12eaf13dc8c28f23e33f867) - GitHub
* **deps**: Bump criterion from 0.5.0 to 0.5.1 (#17500) [da7bc95](https://github.com/answerbook/vector/commit/da7bc951c450c1274fa37abb2d19b83dd3f965ab) - GitHub
* **deps**: Bump crossbeam-utils from 0.8.15 to 0.8.16 (#17674) [714ccf8](https://github.com/answerbook/vector/commit/714ccf8e77426b916ab88121c45a611106ebd6fe) - GitHub
* **deps**: Bump csv from 1.2.1 to 1.2.2 (#17555) [bcc5b6c](https://github.com/answerbook/vector/commit/bcc5b6c5c883e16bd959b610890f67ffc0405860) - GitHub
* **deps**: bump data-encoding from 2.3.3 to 2.4.0 (#17452) [9aaf864](https://github.com/answerbook/vector/commit/9aaf864254bb05a92504533cd3d072341dbcb7e9) - GitHub
* **deps**: Bump getrandom from 0.2.9 to 0.2.10 (#17613) [bd880f5](https://github.com/answerbook/vector/commit/bd880f55d2d8605733297acb4f96a8100a60dad4) - GitHub
* **deps**: Bump gloo-utils from 0.1.6 to 0.1.7 (#17707) [53e1785](https://github.com/answerbook/vector/commit/53e178570b5b87bc2124f4299865cbb00916fe20) - GitHub
* **deps**: Bump graphql_client from 0.12.0 to 0.13.0 (#17541) [ecb707a](https://github.com/answerbook/vector/commit/ecb707a633020bca8c805d5764b85302b74ca477) - GitHub
* **deps**: Bump h2 from 0.3.19 to 0.3.20 (#17767) [ac3bc72](https://github.com/answerbook/vector/commit/ac3bc72bb5e0fd905e2680d4046a2984de5d07b8) - GitHub
* **deps**: Bump hashbrown from 0.13.2 to 0.14.0 (#17609) [154e393](https://github.com/answerbook/vector/commit/154e39382f4e80998814a693f9d6bb5c89ebebf7) - GitHub
* **deps**: Bump hyper from 0.14.26 to 0.14.27 (#17766) [a2a3609](https://github.com/answerbook/vector/commit/a2a3609c58287240631c409172c4b1944bc7864f) - GitHub
* **deps**: Bump indexmap from 1.9.3 to 2.0.0 (#17755) [248ccb8](https://github.com/answerbook/vector/commit/248ccb8d8252fff386d1b67c17424cd263361cb3) - GitHub
* **deps**: Bump indicatif from 0.17.3 to 0.17.4 (#17532) [1565985](https://github.com/answerbook/vector/commit/1565985746868265a1582a1b33b4eb56cc046c26) - GitHub
* **deps**: Bump indicatif from 0.17.4 to 0.17.5 (#17597) [a164952](https://github.com/answerbook/vector/commit/a164952a145109d95c465645bf08b387a61e408a) - GitHub
* **deps**: Bump infer from 0.13.0 to 0.14.0 (#17737) [326ad08](https://github.com/answerbook/vector/commit/326ad0861215f22c83f681e725abb88b33107e2e) - GitHub
* **deps**: Bump itertools from 0.10.5 to 0.11.0 (#17736) [6e1878b](https://github.com/answerbook/vector/commit/6e1878b1c151a19d7a99fd6c8c8a847cc69db3c8) - GitHub
* **deps**: Bump lalrpop to 0.19.12 (#17457) [1f54415](https://github.com/answerbook/vector/commit/1f54415cb3fd4dc8f3f1b5989aa8d051cbe1faa5) - GitHub
* **deps**: bump lapin from 2.1.1 to 2.1.2 (#17439) [a8b7899](https://github.com/answerbook/vector/commit/a8b7899bea771e6f2ca2e7c78c5a1c578f03d78f) - GitHub
* **deps**: bump lapin from 2.1.2 to 2.2.0 (#17443) [b639422](https://github.com/answerbook/vector/commit/b6394228d53508f22c6a65c69961baff19457c05) - GitHub
* **deps**: bump lapin from 2.2.0 to 2.2.1 (#17448) [618379a](https://github.com/answerbook/vector/commit/618379a27583f6233a76c5b788616816b74bee03) - GitHub
* **deps**: Bump libc from 0.2.144 to 0.2.146 (#17615) [10cfd0a](https://github.com/answerbook/vector/commit/10cfd0aec905c605248ad9d36abb312d4bfc1a5b) - GitHub
* **deps**: Bump libc from 0.2.146 to 0.2.147 (#17753) [1a75ec6](https://github.com/answerbook/vector/commit/1a75ec6656cc194068fb98f3d18e14705ef32c91) - GitHub
* **deps**: Bump log from 0.4.17 to 0.4.18 (#17526) [5a2fea1](https://github.com/answerbook/vector/commit/5a2fea10da7eaa04b7e51af84cdea87ab6e8326b) - GitHub
* **deps**: Bump log from 0.4.18 to 0.4.19 (#17662) [e1b3357](https://github.com/answerbook/vector/commit/e1b335748ef3b1345db9f5b9af11b5df2f24868a) - GitHub
* **deps**: Bump lru from 0.10.0 to 0.10.1 (#17810) [96e68f7](https://github.com/answerbook/vector/commit/96e68f76efe2208a8899b3f8961125ba5424a9ba) - GitHub
* **deps**: bump memmap2 from 0.6.1 to 0.6.2 (#17482) [79f7dfb](https://github.com/answerbook/vector/commit/79f7dfb4d4633badf8ee89f0e940fa44f5bd59aa) - GitHub
* **deps**: Bump memmap2 from 0.6.2 to 0.7.0 (#17641) [593ea1b](https://github.com/answerbook/vector/commit/593ea1bc89303f2f2344cca58d7c1aa5de939084) - GitHub
* **deps**: Bump memmap2 from 0.7.0 to 0.7.1 (#17752) [4236e32](https://github.com/answerbook/vector/commit/4236e32cb1fe514e117fa8737e43f6dd51b937dd) - GitHub
* **deps**: Bump mock_instant from 0.3.0 to 0.3.1 (#17574) [1c1beb8](https://github.com/answerbook/vector/commit/1c1beb8123e1b0c82537ae3c2e26235bc6c0c43b) - GitHub
* **deps**: Bump mongodb from 2.5.0 to 2.6.0 (#17726) [c96e3be](https://github.com/answerbook/vector/commit/c96e3be34c239e94a366f9ced8e0e8b69570a562) - GitHub
* **deps**: Bump notify from 6.0.0 to 6.0.1 (#17700) [cd6d154](https://github.com/answerbook/vector/commit/cd6d1540bf74d13ad6bc9c90fc3fe2affb11e6dc) - GitHub
* **deps**: Bump once_cell from 1.17.1 to 1.17.2 (#17531) [8e113ad](https://github.com/answerbook/vector/commit/8e113addc48328f3918e6abc7623284d93d4030b) - GitHub
* **deps**: Bump once_cell from 1.17.2 to 1.18.0 (#17596) [dc6bef2](https://github.com/answerbook/vector/commit/dc6bef2a2e6c47e145c776b4fd91042b112a0890) - GitHub
* **deps**: bump opendal from 0.34.0 to 0.35.0 (#17471) [ebf958b](https://github.com/answerbook/vector/commit/ebf958b1355b4b729e7c99232bc40e2f7e809abf) - GitHub
* **deps**: Bump opendal from 0.35.0 to 0.36.0 (#17540) [dbd7151](https://github.com/answerbook/vector/commit/dbd7151aa4128638765e360f3f0f4e6582735041) - GitHub
* **deps**: Bump opendal from 0.36.0 to 0.37.0 (#17614) [b5bd85f](https://github.com/answerbook/vector/commit/b5bd85f87e39389a2ea3bb9a3d588fcbdfd0e29d) - GitHub
* **deps**: Bump opendal from 0.37.0 to 0.38.0 (#17777) [ec4785a](https://github.com/answerbook/vector/commit/ec4785a9f2ce948a2d44f777f13e69d1e8b7400c) - GitHub
* **deps**: Bump openssl from 0.10.52 to 0.10.53 (#17534) [078de66](https://github.com/answerbook/vector/commit/078de661e7146a1924c0c31fed65b8b0ccbb7316) - GitHub
* **deps**: Bump openssl from 0.10.53 to 0.10.54 (#17573) [4af5e6d](https://github.com/answerbook/vector/commit/4af5e6d8886cfc326209f8d6aa65d27f86f6e579) - GitHub
* **deps**: Bump openssl from 0.10.54 to 0.10.55 (#17716) [dd2527d](https://github.com/answerbook/vector/commit/dd2527dcea295f4f9f6eb617306a822892e08a59) - GitHub
* **deps**: Bump percent-encoding from 2.2.0 to 2.3.0 (#17602) [8e04259](https://github.com/answerbook/vector/commit/8e042590117989394f8bc246dc6d7de61d00123a) - GitHub
* **deps**: Bump pin-project from 1.1.0 to 1.1.1 (#17806) [0b32626](https://github.com/answerbook/vector/commit/0b32626848f5189d6832e6f8ea3c66ebaa553975) - GitHub
* **deps**: Bump PR limit for Dependabot to 100 (#17459) [85703e7](https://github.com/answerbook/vector/commit/85703e792fe0ff70a466380823cf2d4b14b21603) - GitHub
* **deps**: bump proc-macro2 from 1.0.57 to 1.0.58 (#17426) [ae656c7](https://github.com/answerbook/vector/commit/ae656c7124b9c148e7a678967f58edc2a32501e5) - GitHub
* **deps**: Bump proc-macro2 from 1.0.58 to 1.0.59 (#17495) [4ce3278](https://github.com/answerbook/vector/commit/4ce3278ba5c2b92391818ff85c410a01f6b71cbf) - GitHub
* **deps**: Bump proc-macro2 from 1.0.59 to 1.0.60 (#17643) [f20eb2f](https://github.com/answerbook/vector/commit/f20eb2ff554c0163ea4955c9a5ad1ef0acd9f492) - GitHub
* **deps**: Bump proc-macro2 from 1.0.60 to 1.0.63 (#17757) [63ba2a9](https://github.com/answerbook/vector/commit/63ba2a95d972bbba11cd9a1f913f2606bb2ba20b) - GitHub
* **deps**: bump proptest from 1.1.0 to 1.2.0 (#17476) [9235fc2](https://github.com/answerbook/vector/commit/9235fc249f4a0aa34d1119ed7dd334e23e5c3674) - GitHub
* **deps**: bump pulsar from 5.1.1 to 6.0.0 (#17587) [3395cfd](https://github.com/answerbook/vector/commit/3395cfdb90b165653dda7e9014057aac1dba2d28) - GitHub
* **deps**: Bump pulsar from 6.0.0 to 6.0.1 (#17673) [8d98bb8](https://github.com/answerbook/vector/commit/8d98bb8c4f4a4dd44e433caf8846aee4df1eec2b) - GitHub
* **deps**: Bump quanta from 0.11.0 to 0.11.1 (#17524) [2388c2f](https://github.com/answerbook/vector/commit/2388c2f492a4952e48f1c1f8469045378ec60739) - GitHub
* **deps**: Bump quote from 1.0.27 to 1.0.28 (#17496) [cc30746](https://github.com/answerbook/vector/commit/cc307460df2b45af6f33311d493c6bd7f9d44da5) - GitHub
* **deps**: Bump quote from 1.0.28 to 1.0.29 (#17798) [cba983e](https://github.com/answerbook/vector/commit/cba983e381af933ac360812aec82d013e7e84fa4) - GitHub
* **deps**: Bump quote from 1.0.28 to 1.0.29 (#17815) [bf9828d](https://github.com/answerbook/vector/commit/bf9828d03b92a6b7ce0295d3468eb1c139f5a1fc) - GitHub
* **deps**: bump rdkafka from 0.30.0 to 0.31.0 (#17428) [e7fa8d3](https://github.com/answerbook/vector/commit/e7fa8d373b74117c4d0d90902c3124e620c3c6c3) - GitHub
* **deps**: Bump rdkafka from 0.31.0 to 0.32.2 (#17664) [ac68a7b](https://github.com/answerbook/vector/commit/ac68a7b8d8238f4d64d5f3850e15dc9931e39349) - GitHub
* **deps**: bump regex from 1.8.1 to 1.8.2 (#17469) [897e45d](https://github.com/answerbook/vector/commit/897e45d5aa3d9ede6aa9115dae41a90b5a200ffa) - GitHub
* **deps**: bump regex from 1.8.2 to 1.8.3 (#17494) [5d90cff](https://github.com/answerbook/vector/commit/5d90cff55c04701692dfe2b92416c3cf4ded5a4d) - GitHub
* **deps**: Bump regex from 1.8.3 to 1.8.4 (#17601) [657758d](https://github.com/answerbook/vector/commit/657758db74496ec9adede09fc8f132bd8bed3bc3) - GitHub
* **deps**: bump reqwest from 0.11.17 to 0.11.18 (#17420) [2ed8ec7](https://github.com/answerbook/vector/commit/2ed8ec77d6effb6c373f56209aa52d9f6158f571) - GitHub
* **deps**: bump security-framework from 2.9.0 to 2.9.1 (#17441) [ac0c7e8](https://github.com/answerbook/vector/commit/ac0c7e82fc5877a58a60da872c40ad9b63143953) - GitHub
* **deps**: Bump serde from 1.0.163 to 1.0.164 (#17632) [e35150e](https://github.com/answerbook/vector/commit/e35150e8b376db1f19b60b828233eb47393bb2dd) - GitHub
* **deps**: Bump serde_json from 1.0.96 to 1.0.97 (#17701) [25131ef](https://github.com/answerbook/vector/commit/25131efdbe855a8f4d2491bd68fb76c58f7f8ad4) - GitHub
* **deps**: Bump serde_json from 1.0.97 to 1.0.99 (#17754) [e07158c](https://github.com/answerbook/vector/commit/e07158c7a80352d2d36216eb90141033b863964a) - GitHub
* **deps**: Bump serde_yaml from 0.9.21 to 0.9.22 (#17756) [e164b36](https://github.com/answerbook/vector/commit/e164b36436b85a332b5a3b4c492caab6b53578d3) - GitHub
* **deps**: Bump sha2 from 0.10.6 to 0.10.7 (#17698) [d122d32](https://github.com/answerbook/vector/commit/d122d32b8c83133b753c9e31d19be6c6609fb9a5) - GitHub
* **deps**: Bump tempfile from 3.5.0 to 3.6.0 (#17617) [c55c9ec](https://github.com/answerbook/vector/commit/c55c9ecbf904d9166c88af65a9a3f76f18289f58) - GitHub
* **deps**: Bump tokio from 1.28.1 to 1.28.2 (#17525) [cc703da](https://github.com/answerbook/vector/commit/cc703da814928b41e0d9c0d7d211181f4aa5758a) - GitHub
* **deps**: Bump tokio from 1.28.2 to 1.29.0 (#17776) [e26e8b8](https://github.com/answerbook/vector/commit/e26e8b804c7fed0affad156160b65ba5e0df5a6e) - GitHub
* **deps**: bump toml from 0.7.3 to 0.7.4 (#17440) [91ba052](https://github.com/answerbook/vector/commit/91ba052ba59d920761a02f7999c4b5d8b39d1766) - GitHub
* **deps**: Bump toml from 0.7.4 to 0.7.5 (#17751) [35c4581](https://github.com/answerbook/vector/commit/35c458163ac11baa4cba73b37dadaf71d41fd13a) - GitHub
* **deps**: Bump tower-http from 0.4.0 to 0.4.1 (#17711) [e5e6b96](https://github.com/answerbook/vector/commit/e5e6b9635cf3fd13676d845f184ef3a04167ceef) - GitHub
* **deps**: Bump url from 2.3.1 to 2.4.0 (#17608) [d956092](https://github.com/answerbook/vector/commit/d956092efdcc4ccea718365d9e9ef7bd537563a8) - GitHub
* **deps**: Bump uuid from 1.3.3 to 1.3.4 (#17682) [c97d619](https://github.com/answerbook/vector/commit/c97d619d47b1171d592dcf55692b5caa01e97992) - GitHub
* **deps**: Bump uuid from 1.3.4 to 1.4.0 (#17775) [935babf](https://github.com/answerbook/vector/commit/935babf1ab6edcc345960af77a387712ffe36304) - GitHub
* **deps**: Bump wasm-bindgen from 0.2.86 to 0.2.87 (#17672) [19c4d4f](https://github.com/answerbook/vector/commit/19c4d4f72a4c08fdf51299bd7b3b906f8f8d08c1) - GitHub
* **deps**: Bump wiremock from 0.5.18 to 0.5.19 (#17618) [460bbc7](https://github.com/answerbook/vector/commit/460bbc7b9e532f93ac015ff871535c16135e4793) - GitHub
* **deps**: Bump xml-rs from 0.8.4 to 0.8.14 (#17607) [a932489](https://github.com/answerbook/vector/commit/a9324892a289e94214707f1e09ea2931ae27d5e3) - GitHub
* **deps**: Drop use of `hashlink` crate (#17678) [41ee394](https://github.com/answerbook/vector/commit/41ee39414ea3210c841659f1f41b3295ad8bfd23) - GitHub
* **deps**: Export more common bits for components (#17788) [062224b](https://github.com/answerbook/vector/commit/062224b485f27193288443cede0ae1c2f5c66196) - GitHub
* **deps**: Update fs_extra to 1.3.0 (#17458) [299fd6a](https://github.com/answerbook/vector/commit/299fd6ab53b1e818d09ae38f4321c20bdce4f30e) - GitHub
* **deps**: Upgrade Ruby version to 3.1.4 (#17722) [ddebde9](https://github.com/answerbook/vector/commit/ddebde97bac79eaecb7feb286bfe5a25591e7d13) - GitHub
* **deps**: Upgrade rust to 1.70.0 (#17585) [6c48565](https://github.com/answerbook/vector/commit/6c4856595410ee77d52d62ceb2cd808b1cdff04e) - GitHub
* **dev**: Add @dsmith3197 to CODEOWNERS (#17729) [a08443c](https://github.com/answerbook/vector/commit/a08443c890cc0e3223e4d17c71eb267f0305d50c) - GitHub
* **docs**: Add info about Vector Operator to Kubernetes instalation page (#17432) [54d9c99](https://github.com/answerbook/vector/commit/54d9c99492ec14924994a4857961aaafe3200f9b) - GitHub
* **docs**: add instructions for regenerating component docs and licenses (#17828) [93ef6c3](https://github.com/answerbook/vector/commit/93ef6c3e9241601253b48e27ee817e73474a89c6) - GitHub
* **docs**: Add Log Namespacing docs (#16571) [7d098e4](https://github.com/answerbook/vector/commit/7d098e42dfd08ea1f2e63355e2a95c2b38e3b768) - GitHub
* **docs**: add note about const strings (#17774) [d7bc531](https://github.com/answerbook/vector/commit/d7bc531ee29f563822ed152c97adfc7d7bb0ef81) - GitHub
* **docs**: Clarify `bytes` framing for streams (#17745) [7d10fc9](https://github.com/answerbook/vector/commit/7d10fc97f32c053f9336d1d69d530f39ef258268) - GitHub
* **docs**: Clarify when component received and sent bytes events should be emitted (#17464) [547783d](https://github.com/answerbook/vector/commit/547783d17e8d2d3d351213a034e8d38fdcaa3047) - GitHub
* **docs**: Move CONTRIBUTING.md to top-level (#17744) [7a0dec1](https://github.com/answerbook/vector/commit/7a0dec13537211b4a7e460cdf57b079709649b5f) - GitHub
* Download submodules in the CI checkouts (#17760) [5417a06](https://github.com/answerbook/vector/commit/5417a06e29f7a6050f916df993edba0149084b57) - GitHub
* Dropped error field from StreamClosed Error (#17693) [ee480cd](https://github.com/answerbook/vector/commit/ee480cd08a5451bc3f0b83a2b037ba131e38d4b9) - GitHub
* **enrichment**: avoid importing vector-common in enrichment module (#17653) [45a28f8](https://github.com/answerbook/vector/commit/45a28f88a910c8492872773cc2e86045c8e2f4b6) - GitHub
* **enterprise**: Extend library functionality for secret scanning (#17483) [541bb00](https://github.com/answerbook/vector/commit/541bb0087eb95b8d67c98547240c8104c5b2a69f) - GitHub
* **external docs**: fix reference to supported aarch64 architecture (#17553) [247bb80](https://github.com/answerbook/vector/commit/247bb807cae195c5c987a43e3c4e6ab6b885a94b) - GitHub
* **external docs**: update fluentd link (#17436) [187f142](https://github.com/answerbook/vector/commit/187f142ef5c28dec8e9b1ffbdfe0196acbe45804) - GitHub
* Fix publish workflow for older OS images (#17787) [ab39c6a](https://github.com/answerbook/vector/commit/ab39c6ac6816c8499cc87050a21945f984638dab) - GitHub
* **flush on shutdown**: validate s3 sink flushes (#17667) [c21f892](https://github.com/answerbook/vector/commit/c21f892e574579e323742da009f15a39c43555af) - GitHub
* **kubernetes_logs source**: Add warning about Windows support (#17762) [a53c7a2](https://github.com/answerbook/vector/commit/a53c7a2153960038b8e68e13d6beede09eb1a69a) - GitHub
* **kubernetes**: Bump k8s manifests to 0.22.0 (#17467) [f547871](https://github.com/answerbook/vector/commit/f54787190119255c1f97b2fe603ea5e65355b1cd) - GitHub
* **observability**: emit `component_sent` events by `source` and `service` (#17549) [dcf7f9a](https://github.com/answerbook/vector/commit/dcf7f9ae538c821eb7b3baf494d3e8938083832c) - GitHub
* **observability**: ensure `sent_event` and `received_event` metrics are estimated json size (#17465) [3b2a2be](https://github.com/answerbook/vector/commit/3b2a2be1b075344a92294c1248b09844f895ad72) - GitHub
* **observability**: Have `tower_limit` use configured log level (#17715) [08099a8](https://github.com/answerbook/vector/commit/08099a8b567663416d907600e2f9c678482af272) - GitHub
* **observability**: remove deprecated internal metrics + massive cleanup to vector top and graphql API (#17516) [98c54ad](https://github.com/answerbook/vector/commit/98c54ad3a371ac710151367a953252f9eb293548) - GitHub
* **observability**: remove more deprecated internal metrics (#17542) [b0ed167](https://github.com/answerbook/vector/commit/b0ed167d1ae22b8f0a7a762ad50750c912f0833b) - GitHub
* **observability**: set source fields to mean service (#17470) [670bdea](https://github.com/answerbook/vector/commit/670bdea00ab7a13921aa3194667068b27f58e35a) - GitHub
* **releasing**: Prepare v0.30.0 release [af2b2af](https://github.com/answerbook/vector/commit/af2b2afdd95c8ed092beabc443fbd5c5d263a53e) - Jesse Szwedko
* **releasing**: Prepare v0.31.0 release [0f13b22](https://github.com/answerbook/vector/commit/0f13b22a4cebbba000444bdb45f02bc820730a13) - Jesse Szwedko
* remove custom async sleep impl (#17493) [b28d915](https://github.com/answerbook/vector/commit/b28d915cb6a48da836bb4736c027f1ca5d623fe2) - GitHub
* Remove links to roadmap (#17554) [349c718](https://github.com/answerbook/vector/commit/349c7183067f0aa91b05914f34a68ee899fea88b) - GitHub
* Revert all submodule introductions to fix CI (#17800) [d8d57e5](https://github.com/answerbook/vector/commit/d8d57e55c0c51c6fdb8c41f2fa48b0876ef4d356) - GitHub
* RFC for Data Volume Insights (#17322) [a551f33](https://github.com/answerbook/vector/commit/a551f33da2b752229bd8139c72af80ce8b149638) - GitHub
* **sinks**: Drop the custom `SinkContext::default` implementation (#17804) [e66e285](https://github.com/answerbook/vector/commit/e66e285cbd944bcb65b1262fb91bc1913b1885a6) - GitHub
* **sinks**: mark VectorSink::from_event_sink as deprecated (#17649) [0dc450f](https://github.com/answerbook/vector/commit/0dc450fac14ac0236ca48466fd4fe42630d421ed) - GitHub
* **statsd sink**: refactor `statsd` sink to stream-based style (#16199) [2a76cac](https://github.com/answerbook/vector/commit/2a76cac4d327eac537996d3409a64633c96f5ac8) - GitHub
* update `vrl` to `0.4.0` (#17378) [426d660](https://github.com/answerbook/vector/commit/426d6602d22193940ac6e495fc5c175aa3bc8f90) - GitHub
* Update the NOTICE file (#17430) [9a44e6e](https://github.com/answerbook/vector/commit/9a44e6e8763c5d2bc91de1c24b14662d10d0b434) - GitHub
* Upgrade aws-smithy and aws-sdk crates (#17731) [6a6b42b](https://github.com/answerbook/vector/commit/6a6b42bedbd27dec0c91e274698785cc73f805df) - GitHub
* **website**: Fix upgrade guide dates [80de738](https://github.com/answerbook/vector/commit/80de738b8de91c378f1ab7a58a0a02201f4402fd) - Jesse Szwedko


### Features

* add metadata support to `set_semantic_meaning` (#17730) [44be378](https://github.com/answerbook/vector/commit/44be37843c0599abb64073fe737ce146e30b3aa5) - GitHub
* **codecs**: add lossy option to `gelf`, `native_json`, and `syslog` deserializers (#17680) [2dfa850](https://github.com/answerbook/vector/commit/2dfa8509bcdb4220d32e3d91f7fdd61c081db5ea) - GitHub
* **codecs**: Add lossy option to JSON deserializer (#17628) [bf7d796](https://github.com/answerbook/vector/commit/bf7d79623c0b575dd0bb6f851cc12c15cea5eb5f) - GitHub
* **configurable shutdown duration**: make shutdown duration configurable (#17479) [23ed0e3](https://github.com/answerbook/vector/commit/23ed0e3adbffdd770a257635c3d6720a3bf072e7) - GitHub
* **error code when shutdown fails**: set exit flag to non-zero when shutdown times out (#17676) [cc52c0e](https://github.com/answerbook/vector/commit/cc52c0ea99e03f451c24c165b24430c045ff365d) - GitHub
* **internal telemetry at shutdown**: close internal sources after external ones (#17741) [812929b](https://github.com/answerbook/vector/commit/812929b1761355e2209ce33b3fc439d9b8b0d182) - GitHub
* **journald source**: add journal_namespace option (#17648) [a324a07](https://github.com/answerbook/vector/commit/a324a07ba1b62baac08d74b287595846b787b887) - GitHub
* **kinesis sinks**: implement full retry of partial failures in firehose/streams (#17535) [bebac21](https://github.com/answerbook/vector/commit/bebac21cb699be64d1b009d3619d5af5c5be20ec) - GitHub
* **prometheus**: add more compression algorithms to Prometheus Remote Write (#17334) [380d7ad](https://github.com/answerbook/vector/commit/380d7adb72a02e8da0af35fd3d80ecb1d8b0b541) - GitHub
* track runtime schema definitions for log events (#17692) [6eecda5](https://github.com/answerbook/vector/commit/6eecda55020214364fda844cf8ed16a9b6cc2a5c) - GitHub


### Miscellaneous

* Merge pull request #371 from answerbook/feature/LOG-18200 [491ea31](https://github.com/answerbook/vector/commit/491ea31c9246e2cbfa6bd8f4fd223c1a13ad63d4) - GitHub [LOG-18200](https://logdna.atlassian.net/browse/LOG-18200)
* Merge pull request #374 from answerbook/dominic/LOG-18535-temp-revert [df87df7](https://github.com/answerbook/vector/commit/df87df78fa1a97b519729e4ecca7bf0d6e656b32) - GitHub [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535) [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)
* Revert "Merge pull request #370 from answerbook/dominic/LOG-18535" [73a8b28](https://github.com/answerbook/vector/commit/73a8b289e09f0386c851a79c8a2aa773f208ee66) - dominic-mcallister-logdna [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)
* Merge branch 'master' into feature/LOG-18200 [fcd21f3](https://github.com/answerbook/vector/commit/fcd21f3d9eb2254ce01fad69edd7c112e4a36c5e) - Darin Spivey [LOG-18200](https://logdna.atlassian.net/browse/LOG-18200)
* Merge tag 'v0.31.0' into upstream-0.31.0 [9443fb1](https://github.com/answerbook/vector/commit/9443fb16196cf498024f01dac3bd39492ac820e1) - Darin Spivey
* Update VRL to `0.5.0` (#17793) [671aa79](https://github.com/answerbook/vector/commit/671aa795136e319889a710986d41fadae9ec980f) - GitHub
* fix `demo_logs` metadata source name (#17689) [83af7ea](https://github.com/answerbook/vector/commit/83af7ea47f661ff22ba5aae728584390ea80743f) - GitHub
* enhancement(s3 source) Add minimal support to unwrap an S3-SQS event from an SNS event (#17352) [7a7bc9a](https://github.com/answerbook/vector/commit/7a7bc9a3fe65d04d4e945186b1cbb31517ed8a64) - GitHub
* Additional notes on proposing new integrations (#17658) [2ad964d](https://github.com/answerbook/vector/commit/2ad964d43b9a47808104eced885cebf6541f4a72) - GitHub
* **ci**: reduce billable time of Test Suite (#17714) [bc69255](https://github.com/answerbook/vector/commit/bc6925592f8d954212efb99f2f17bcac8a454169) - GitHub
* **ci**: refactor logic for int test file path changes detection (#17725) [92a36e0](https://github.com/answerbook/vector/commit/92a36e0119e0e1f50b8bfcdcaf1c536018b69d5f) - GitHub
* **compression**: zstd compression support (#17371) [ced219e](https://github.com/answerbook/vector/commit/ced219e70405c9ed9012444cc04efad8f91d3590) - GitHub
* **dev**:  move blocked/waiting gardener issues to triage on comment (#17588) [6b34868](https://github.com/answerbook/vector/commit/6b34868e285a4608914405b7701ae1ee82deb536) - GitHub
* explain how to run tests locally (#17783) [3b67a80](https://github.com/answerbook/vector/commit/3b67a80f44c8abf9ba0e0a9bd77ee19d4a51d91a) - GitHub
* remove aggregator beta warning (#17750) [94e3f15](https://github.com/answerbook/vector/commit/94e3f1542be0c4ba93f554803973c9e26e7dc566) - GitHub

## [1.29.2](https://github.com/answerbook/vector/compare/v1.29.1...v1.29.2) (2023-12-19)


### Chores

* Add counter for usage metric insert failure [b3869c1](https://github.com/answerbook/vector/commit/b3869c130dca97df7c128472b8ae1ff4091c34b6) - Jorge Bay [LOG-18638](https://logdna.atlassian.net/browse/LOG-18638)

## [1.29.1](https://github.com/answerbook/vector/compare/v1.29.0...v1.29.1) (2023-12-19)


### Chores

* **otlp traces**: move resource and scope attributes to metadata [f7bfe6c](https://github.com/answerbook/vector/commit/f7bfe6c5f0443933146bc94c26f5bf8e0e6914e9) - Sergey Opria [LOG-18614](https://logdna.atlassian.net/browse/LOG-18614)


### Miscellaneous

* Merge pull request #367 from answerbook/sopria/LOG-18614 [25460f4](https://github.com/answerbook/vector/commit/25460f4749b50d8753a4656dc1a487692de34f3d) - GitHub [LOG-18614](https://logdna.atlassian.net/browse/LOG-18614)

# [1.29.0](https://github.com/answerbook/vector/compare/v1.28.2...v1.29.0) (2023-12-19)


### Bug Fixes

* **test**: do not use system clock in unit tests [4096793](https://github.com/answerbook/vector/commit/4096793d348a9d74f5ae3e5119e61169e9c0a8b5) - Dan Hable [LOG-18873](https://logdna.atlassian.net/browse/LOG-18873)


### Features

* **s3 sink**: add file consolidation [1484d29](https://github.com/answerbook/vector/commit/1484d293035aa5083fd326f5cf5b72f762779a63) - dominic-mcallister-logdna [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)


### Miscellaneous

* Merge pull request #370 from answerbook/dominic/LOG-18535 [f67cd9f](https://github.com/answerbook/vector/commit/f67cd9fa4a6cfaf39dfaa99d2c1b7fb3d02c4cd9) - GitHub [LOG-18535](https://logdna.atlassian.net/browse/LOG-18535)

## [1.28.2](https://github.com/answerbook/vector/compare/v1.28.1...v1.28.2) (2023-12-18)


### Chores

* **otlp metrics**: move resource and scope attributes to metadata [fb92547](https://github.com/answerbook/vector/commit/fb925477fe41627115a0bd0d6eef1f5813f19ce1) - Sergey Opria [LOG-18613](https://logdna.atlassian.net/browse/LOG-18613)


### Miscellaneous

* Merge pull request #361 from answerbook/sopria/LOG-18613 [aa4bdb2](https://github.com/answerbook/vector/commit/aa4bdb2568f0707f27e53dd056321030cf29a8e6) - GitHub [LOG-18613](https://logdna.atlassian.net/browse/LOG-18613)

## [1.28.1](https://github.com/answerbook/vector/compare/v1.28.0...v1.28.1) (2023-12-14)


### Chores

* **classifier**: Use first string line field as message_key [6386d79](https://github.com/answerbook/vector/commit/6386d797720921bce24d4c4cedf0827d37005087) - Jorge Bay [LOG-18851](https://logdna.atlassian.net/browse/LOG-18851)

# [1.28.0](https://github.com/answerbook/vector/compare/v1.27.3...v1.28.0) (2023-12-14)


### Bug Fixes

* **test**: increased window size to ensure nothing expires [6fa2fbe](https://github.com/answerbook/vector/commit/6fa2fbe5fc2fd6d8b60b239d3e31321d981e3348) - Dan Hable [LOG-18845](https://logdna.atlassian.net/browse/LOG-18845)


### Features

* Capture log type in data profiling [c8d6828](https://github.com/answerbook/vector/commit/c8d6828cc47701786a63548b52b08370df67624d) - Jorge Bay [LOG-18784](https://logdna.atlassian.net/browse/LOG-18784) [LOG-18834](https://logdna.atlassian.net/browse/LOG-18834)

## [1.27.3](https://github.com/answerbook/vector/compare/v1.27.2...v1.27.3) (2023-12-13)


### Bug Fixes

* Avoid resetting the log cluster templates [90258ff](https://github.com/answerbook/vector/commit/90258ff44108559c067d1d8a3e696ac5cc6e586e) - Jorge Bay [LOG-18842](https://logdna.atlassian.net/browse/LOG-18842)

## [1.27.2](https://github.com/answerbook/vector/compare/v1.27.1...v1.27.2) (2023-12-12)


### Bug Fixes

* Reference to configmap for data profiling env vars [0349935](https://github.com/answerbook/vector/commit/0349935dde34ac0d7ef68cb57d2371d8fbdc918d) - Jorge Bay [LOG-18636](https://logdna.atlassian.net/browse/LOG-18636)

## [1.27.1](https://github.com/answerbook/vector/compare/v1.27.0...v1.27.1) (2023-12-11)


### Bug Fixes

* **log_clustering**: Always set the current node [e41e152](https://github.com/answerbook/vector/commit/e41e1528a217d8ba16c8c30a75d5f050e1068bce) - Jorge Bay [LOG-18799](https://logdna.atlassian.net/browse/LOG-18799)

# [1.27.0](https://github.com/answerbook/vector/compare/v1.26.0...v1.27.0) (2023-12-08)


### Features

* **clickhouse**: enable clickhouse feature [ee4b606](https://github.com/answerbook/vector/commit/ee4b6063231eba92b0029a0cff2ea128c364f7e0) - Mike Del Tito [LOG-18821](https://logdna.atlassian.net/browse/LOG-18821)


### Miscellaneous

* Merge pull request #362 from answerbook/mdeltito/LOG-18821 [8f8d258](https://github.com/answerbook/vector/commit/8f8d258f202ec62aa1da8a02660cf9e2f1b00bbe) - GitHub [LOG-18821](https://logdna.atlassian.net/browse/LOG-18821)

# [1.26.0](https://github.com/answerbook/vector/compare/v1.25.7...v1.26.0) (2023-12-07)


### Chores

* **deps**: Bump warp and async-graphql-warp versions [d5626b0](https://github.com/answerbook/vector/commit/d5626b0edfaec01863cb8386eeae284486a4eaac) - Dan Hable [LOG-18567](https://logdna.atlassian.net/browse/LOG-18567)


### Features

* **transform**: sliding window aggregator [848037b](https://github.com/answerbook/vector/commit/848037b459d37a51786cda96d026a5d0ddc395a5) - Dan Hable [LOG-18567](https://logdna.atlassian.net/browse/LOG-18567)

## [1.25.7](https://github.com/answerbook/vector/compare/v1.25.6...v1.25.7) (2023-12-04)


### Bug Fixes

* **otlp**: parse trace, span ids correctly [85ee7b3](https://github.com/answerbook/vector/commit/85ee7b315c6a4394c5a042f00547b6a1e2bd3a73) - Matt March [LOG-18753](https://logdna.atlassian.net/browse/LOG-18753)

## [1.25.6](https://github.com/answerbook/vector/compare/v1.25.5...v1.25.6) (2023-12-01)


### Chores

* Support enabling data profile metrics collection [a922406](https://github.com/answerbook/vector/commit/a922406dfef88f4f3fa221da47b20b2dab16f627) - Jorge Bay [LOG-18636](https://logdna.atlassian.net/browse/LOG-18636)

## [1.25.5](https://github.com/answerbook/vector/compare/v1.25.4...v1.25.5) (2023-11-30)


### Bug Fixes

* **classification**: Remove tracking undefined event types [2cec43e](https://github.com/answerbook/vector/commit/2cec43ee08640dfaedba4f13a6f2151901b20bd8) - Darin Spivey [LOG-18724](https://logdna.atlassian.net/browse/LOG-18724)


### Miscellaneous

* Merge pull request #357 from answerbook/darinspivey/LOG-18724 [77bc735](https://github.com/answerbook/vector/commit/77bc7355959926b81ea26f1432bbe0fceabcae58) - GitHub [LOG-18724](https://logdna.atlassian.net/browse/LOG-18724)

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
