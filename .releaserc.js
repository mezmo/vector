const logdnaConfig = require('@answerbook/release-config-logdna')

const skipMe = '@semantic-release/release-notes-generator'

module.exports = {
  ...logdnaConfig
, plugins: logdnaConfig.plugins.filter(plugin => {
    const pluginName = Array.isArray(plugin) ? plugin[0] : plugin
    return pluginName !== skipMe
  })
, "branches": ["master"]
, "changelogFile": "MEZMO_CHANGELOG.md"
}
