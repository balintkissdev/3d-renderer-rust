const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    // Start building a dependency graph from "index.js"
    entry: "./js/index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",   // Name of webpack-generated JS
    },
    module: {
        rules: [
            {
                test: /\.css$/i,
                use: [
                    MiniCssExtractPlugin.loader,
                    "css-loader",
                ],
            },
        ],
    },
    plugins: [
        // Inject Webpack-generated JS into HTML template. No need to specify separate `<script>` tags.
        new HtmlWebpackPlugin({
            template: "site/index.html"
        }),
        // Extract CSS
        new MiniCssExtractPlugin({
            filename: "styles.css",
        }),
        // Automatically install wasm-pack locally and call
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "."),
            extraArgs: "--release"
        }),
    ],
    mode: "production",
    experiments: {
        asyncWebAssembly: true
    },
    // Default recommended size limit is 244 KiB, but it is exceeded here
    // because of the bundling of assets into the WASM binary. Disable warning
    // about it.
    // TODO: Revisit asset sizes once switching to Fetch API
    performance: {
        hints: false
    }
};
