const CopyWebpackPlugin = require("copy-webpack-plugin")
const path = require('path')

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({
      patterns: ['index.html'],
    })
  ],
  devServer: {
    headers: {
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Opener-Policy': 'same-origin',
    },
  },
  devtool: "eval-source-map",
  ignoreWarnings: [
    /Circular dependency between chunks with runtime/
  ],
}
