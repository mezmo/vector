const logdnaConfig = require('@answerbook/release-config-logdna')

const config = {
  ...logdnaConfig
, "branches": ["master"]
, "changelogFile": "MEZMO_CHANGELOG.md"
}

/**
 * NOTE: we've run into an issue where the generated release
 * notes were too long to create a release when we've waited too
 * long to pull in changes from up-stream.  This block remains
 * so that we can easily disable the notes generator plugin in
 * case we run into this scenario again.

  const skipMe = '@semantic-release/release-notes-generator'
  config.plugins = logdnaConfig.plugins.filter(plugin => {
    const pluginName = Array.isArray(plugin) ? plugin[0] : plugin
    return pluginName !== skipMe
  })

  */

module.exports = config
