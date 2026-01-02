const HtmlBundlerPlugin = require('html-bundler-webpack-plugin');

module.exports = {
    target: ['web', 'es5'],
    plugins: [
        new HtmlBundlerPlugin({
            entry: {
                index: './src/index.html', // path to template file
            },
            css: {
                inline: true,
            },
            js: {
                inline: true,
            },
        }),
    ],
    module: {
        rules: [
            {
                test: /\.s?css$/,
                use: ['css-loader', 'sass-loader'],
            },
            {
                test: /\.(png|jpe?g|webp|svg)$/,
                type: 'asset/resource',
            },
        ],
    },
};