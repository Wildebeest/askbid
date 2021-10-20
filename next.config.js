const {BugsnagBuildReporterPlugin, BugsnagSourceMapUploaderPlugin} = require("webpack-bugsnag-plugins");

module.exports = {
    webpack: (config, {buildId, dev, isServer, defaultLoaders, webpack}) => {
        config.plugins.push(
            new BugsnagBuildReporterPlugin({
                apiKey: process.env.BUGSNAG_API_KEY,
                appVersion: buildId
            }),
            new BugsnagSourceMapUploaderPlugin({
                apiKey: process.env.BUGSNAG_API_KEY,
                appVersion: buildId
            })
        );
        return config;
    },
}