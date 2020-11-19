const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

module.exports = {
  context: path.resolve(__dirname, '../web/files--common/javascript'),
  entry: './index.ts',
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, '../web/files--common/dist'),
    libraryTarget: 'window'
  },
  module: {
    rules: [
      { test: /\.tsx?$/, use: 'ts-loader', exclude: /node_modules/ }
    ]
  },
  resolve: {
    extensions: ['.ts', '.js'],
    alias: { '@': path.resolve(__dirname, '../web/files--common') }
  },
  plugins: [
    new CleanWebpackPlugin()
  ]
};
