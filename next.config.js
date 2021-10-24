const {BugsnagBuildReporterPlugin, BugsnagSourceMapUploaderPlugin} = require("webpack-bugsnag-plugins");

module.exports = {
    productionBrowserSourceMaps: true,
    webpack: (config, {buildId, dev, isServer, defaultLoaders, webpack}) => {
        if(buildId !== "development") {
            config.plugins.push(
                new BugsnagBuildReporterPlugin({
                    apiKey: process.env.NEXT_PUBLIC_BUGSNAG_API_KEY,
                    appVersion: buildId
                }),
                new BugsnagSourceMapUploaderPlugin({
                    apiKey: process.env.NEXT_PUBLIC_BUGSNAG_API_KEY,
                    appVersion: buildId
                })
            );
        }
        return config;
    },
}