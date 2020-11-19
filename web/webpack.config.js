const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

module.exports = {
  entry: './web/files--common/javascript/index.ts',
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'web/files--common/dist'),
    libraryTarget: 'window'
  },
  devtool: "source-map",
  module: {
    rules: [
      { test: /\.tsx?$/, use: 'ts-loader', exclude: /node_modules/ }
    ]
  },
  resolve: {
    extensions: ['.ts', '.js'],
    alias: { '@': path.resolve(__dirname, 'web/files--common') }
  },
  plugins: [
    new CleanWebpackPlugin()
  ]
}
