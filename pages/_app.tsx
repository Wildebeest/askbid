import 'tailwindcss/tailwind.css';
import Bugsnag from '@bugsnag/js';
import BugsnagPluginReact from '@bugsnag/plugin-react';
import React from 'react';

const enableBugsnag = process.env.NEXT_PUBLIC_BUGSNAG_API_KEY && process.browser;

if (enableBugsnag) {
    Bugsnag.start({
        apiKey: process.env.NEXT_PUBLIC_BUGSNAG_API_KEY,
        plugins: [new BugsnagPluginReact()]
    });
}

function AskBidApp({Component, pageProps}) {
    if (enableBugsnag) {
        const ErrorBoundary = Bugsnag.getPlugin('react')
            .createErrorBoundary(React);
        return (
            <ErrorBoundary>
                <Component {...pageProps} />
            </ErrorBoundary>
        );
    }
    return <Component {...pageProps} />;
}

export default AskBidApp;