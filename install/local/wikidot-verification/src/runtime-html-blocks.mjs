import {createHash} from 'node:crypto';

import {parseFragment, serialize} from 'parse5';

const LOCAL_HANDLE = 'https://example.com/';
const LOCAL_IDENTITY_PREFIX = 'urn:wikijump-runtime-html-block:';

function attribute(node, name) {
  return node.attrs?.find((entry) => entry.name === name)?.value ?? null;
}

function hasHtmlBlockClass(node) {
  return attribute(node, 'class')?.split(/\s+/u).includes('html-block-iframe') ?? false;
}

function visit(nodes, callback) {
  for (const node of nodes) {
    callback(node);
    visit(node.childNodes ?? [], callback);
  }
}

function identityUrl(block) {
  return `${LOCAL_IDENTITY_PREFIX}${block.index}:${block.sha1}`;
}

function localIdentity(src) {
  const match = /^urn:wikijump-runtime-html-block:(?<index>[1-9][0-9]*):(?<sha1>[0-9a-f]{40})$/u.exec(
    src,
  );
  if (!match) return null;
  return {index: Number(match.groups.index), sha1: match.groups.sha1};
}

function liveIdentity(src, pageSlug) {
  if (typeof pageSlug !== 'string' || !pageSlug) return null;
  const prefix = `/${pageSlug}/html/`;
  if (!src.startsWith(prefix)) return null;
  const match = /^(?<sha1>[0-9a-f]{40})-(?<nonce>[1-9][0-9]*)$/u.exec(
    src.slice(prefix.length),
  );
  if (!match) return null;
  return {sha1: match.groups.sha1, nonce: match.groups.nonce};
}

function fixedAttributesMatch(node) {
  return (
    node.type === 'element' &&
    node.name === 'iframe' &&
    hasHtmlBlockClass(node) &&
    attribute(node, 'allowtransparency') === 'true' &&
    attribute(node, 'frameborder') === '0'
  );
}

export function sha1(value) {
  return createHash('sha1').update(value).digest('hex');
}

export function countLocalHtmlBlockHandles(html) {
  let count = 0;
  visit(parseFragment(html).childNodes, (node) => {
    if (
      node.tagName === 'iframe' &&
      hasHtmlBlockClass({
        type: 'element',
        attrs: node.attrs,
      }) &&
      attribute({attrs: node.attrs}, 'src') === LOCAL_HANDLE
    ) {
      count += 1;
    }
  });
  return count;
}

export function bindLocalHtmlBlockPayloads(html, blocks) {
  const fragment = parseFragment(html);
  const candidates = [];
  visit(fragment.childNodes, (node) => {
    if (
      node.tagName === 'iframe' &&
      hasHtmlBlockClass({type: 'element', attrs: node.attrs}) &&
      attribute({attrs: node.attrs}, 'src') === LOCAL_HANDLE
    ) {
      candidates.push(node);
    }
  });
  const contiguous = blocks.every((block, offset) =>
    block.index === offset + 1 &&
    Number.isSafeInteger(block.bytes) &&
    block.bytes >= 0 &&
    /^[0-9a-f]{40}$/u.test(block.sha1) &&
    /^[0-9a-f]{64}$/u.test(block.sha256)
  );
  const countMatches = candidates.length === blocks.length;
  if (contiguous && countMatches) {
    for (const [offset, node] of candidates.entries()) {
      const src = node.attrs.find((entry) => entry.name === 'src');
      src.value = identityUrl(blocks[offset]);
    }
  }
  return {
    html: serialize(fragment),
    binding: {
      status: contiguous && countMatches ? 'tracked' : 'mismatch',
      iframe_count: candidates.length,
      stored_block_count: blocks.length,
      blocks,
    },
  };
}

export function projectRuntimeHtmlBlocks(dom, {side, pageSlug}) {
  const blocks = [];
  let invalid = false;
  const projectNodes = (nodes) => nodes.map((node) => {
    if (node.type !== 'element') return node;
    const children = projectNodes(node.children ?? []);
    if (!fixedAttributesMatch(node)) return {...node, children};
    const src = attribute(node, 'src');
    const identity = side === 'wikidot'
      ? liveIdentity(src, pageSlug)
      : localIdentity(src);
    if (!identity) {
      invalid = true;
      return {...node, children};
    }
    const ordinal = blocks.length + 1;
    blocks.push({
      ordinal,
      sha1: identity.sha1,
      ...(side === 'wikidot'
        ? {nonce: identity.nonce}
        : {stored_index: identity.index}),
    });
    return {
      ...node,
      attrs: node.attrs.map((entry) =>
        entry.name === 'src'
          ? {...entry, value: `urn:wikijump-runtime-html-block:${ordinal}:${identity.sha1}`}
          : entry
      ),
      children,
    };
  });
  return {dom: projectNodes(dom), blocks, invalid};
}
