#!/usr/bin/env node
'use strict'

// TODO: make this less barbaric
// read input and check if it's a typechecking error

// credit: https://github.com/sindresorhus/dev-null-cli
// just wanted to get rid of the `meow` dependency

const { promisify } = require('util')
const stream = require('stream')
const { readableNoopStream, writableNoopStream } = require('noop-stream')

const streamPipeline = promisify(stream.pipeline);

(async () => {
	if (process.stdin.isTTY) {
		await streamPipeline(readableNoopStream(), process.stdout)
	} else {
		await streamPipeline(process.stdin, writableNoopStream())
	}
})()
