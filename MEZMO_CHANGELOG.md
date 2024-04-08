## Changelog

## [3.14.1](https://github.com/mezmo/vector/compare/v3.14.0...v3.14.1) (2024-04-04)
## [3.14.3](https://github.com/answerbook/vector/compare/v3.14.2...v3.14.3) (2024-04-08)


### Bug Fixes

* **ci**: Clean up `package.json` for linting and release [36baa53](https://github.com/mezmo/vector/commit/36baa53b6ca34057e65968a2ba10c7bcb7a35f20) - Darin Spivey [LOG-18250](https://logdna.atlassian.net/browse/LOG-18250)

# [3.14.0](https://github.com/mezmo/vector/compare/v3.13.6...v3.14.0) (2024-04-04)


### Bug Fixes

* **ci**: Clean disk space after ci runs [ci skip] [7963582](https://github.com/mezmo/vector/commit/796358215aaa5eecd597addbd39a6e5fad998a13) - Darin Spivey [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590)
* **throttle**: use ms for window config [f898c1f](https://github.com/mezmo/vector/commit/f898c1ff8b873a63937e061d722a9499cd8da9b4) - Mike Del Tito [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)
* **ci**: Make all vector dev volumnes unique [ee0749c](https://github.com/answerbook/vector/commit/ee0749c296fdeef2220b889b6bce8ec74771f040) - Darin Spivey [LOG-19643](https://logdna.atlassian.net/browse/LOG-19643)

## [3.14.2](https://github.com/answerbook/vector/compare/v3.14.1...v3.14.2) (2024-04-05)


### Chores

* **release**: 3.13.3 [skip ci] [a033400](https://github.com/mezmo/vector/commit/a033400f5d6f2c1ba1bc042159a1862977b02788) - LogDNA Bot [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155) [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155) [LOG-19506](https://logdna.atlassian.net/browse/LOG-19506) [LOG-19506](https://logdna.atlassian.net/browse/LOG-19506) [LOG-19506](https://logdna.atlassian.net/browse/LOG-19506) [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155) [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155) [LOG-19155](https://logdna.atlassian.net/browse/LOG-19155)
* **release**: 3.14.0 [skip ci] [9163805](https://github.com/mezmo/vector/commit/9163805d665c022d7be60a3ceef8ad34cd1ba386) - LogDNA Bot [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565) [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)
* **release**: 3.14.1 [skip ci] [21f683c](https://github.com/mezmo/vector/commit/21f683c05bf9f0c924d4ffb55ad0db0a46e6baad) - LogDNA Bot [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590) [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590) [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590) [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590) [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590)


### Features

* **throttle**: add state persistence [ad882f2](https://github.com/mezmo/vector/commit/ad882f26c0f88b660b75b6be9f9c443fac554620) - Mike Del Tito [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)
* **throttle**: reimplement throttle transform with basic rate limiting [066cac3](https://github.com/mezmo/vector/commit/066cac3957882804c6174f11924e1b2ea4a0b91f) - Mike Del Tito [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)
* reenable `profiling` feature for jemalloc [6e3e3d5](https://github.com/answerbook/vector/commit/6e3e3d57f9c67e4a4683cba22cf838e0f33588d5) - Mike Del Tito [LOG-19647](https://logdna.atlassian.net/browse/LOG-19647)


### Miscellaneous

* Merge pull request #10 from mezmo/darinspivey/sync_3.14.1 [d4585e2](https://github.com/mezmo/vector/commit/d4585e23d66839f950049d5322ff1140b5e3ca57) - GitHub
* Merge remote-tracking branch 'answerbook/master' into darinspivey/sync_3.14.1 [eeae52e](https://github.com/mezmo/vector/commit/eeae52e68e22b5ba85e62a1690b8e98a4a7828a4) - Darin Spivey
* Merge pull request #439 from answerbook/darinspivey/LOG-19590 [b658874](https://github.com/mezmo/vector/commit/b658874d925621326410d26a187ef3087efa24a1) - GitHub [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590)
* Merge pull request #438 from answerbook/mdeltito/LOG-19565 [f8854d6](https://github.com/mezmo/vector/commit/f8854d6c07b4cc173531acb70ced6a2aea424b17) - GitHub [LOG-19565](https://logdna.atlassian.net/browse/LOG-19565)

## [3.13.6](https://github.com/mezmo/vector/compare/v3.13.5...v3.13.6) (2024-04-03)
* Merge pull request #442 from answerbook/mdeltito/LOG-19647 [10e26f6](https://github.com/answerbook/vector/commit/10e26f6e4cb0d7fd311f4c3ab0e432f208903414) - GitHub [LOG-19647](https://logdna.atlassian.net/browse/LOG-19647)

## [3.14.1](https://github.com/answerbook/vector/compare/v3.14.0...v3.14.1) (2024-04-03)


### Bug Fixes

* **ci**: Add `package.json` into the semantic-release assets [24e8774](https://github.com/mezmo/vector/commit/24e8774b206645d4357452161c1417acbcdcb8ce) - Darin Spivey [LOG-18250](https://logdna.atlassian.net/browse/LOG-18250)

## [3.13.5](https://github.com/mezmo/vector/compare/v3.13.4...v3.13.5) (2024-04-03)


### Bug Fixes

* **ci**: update `package.json` version via `npm` plugin [243c05f](https://github.com/mezmo/vector/commit/243c05ffd9192f479108568d0a4046063b7271ed) - Darin Spivey [LOG-18250](https://logdna.atlassian.net/browse/LOG-18250)

## [3.13.4](https://github.com/mezmo/vector/compare/v3.13.3...v3.13.4) (2024-04-03)


### Chores

* **ci**: Publish to `docker.io` and use public repos [631a8ab](https://github.com/mezmo/vector/commit/631a8ab6b3f9482610b80c74ac9d290e5c63787d) - Darin Spivey [LOG-18250](https://logdna.atlassian.net/browse/LOG-18250)
* **ci**: Clean disk space after ci runs [ci skip] [7963582](https://github.com/answerbook/vector/commit/796358215aaa5eecd597addbd39a6e5fad998a13) - Darin Spivey [LOG-19590](https://logdna.atlassian.net/browse/LOG-19590)


### Miscellaneous

* Merge pull request #7 from mezmo/darinspivey/LOG-18250 [7f3158e](https://github.com/mezmo/vector/commit/7f3158e172e6eae32ba00f608f33718db4316c1c) - GitHub [LOG-18250](https://logdna.atlassian.net/browse/LOG-18250)
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
