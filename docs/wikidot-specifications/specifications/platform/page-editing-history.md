# Page editing modes and revision history

- Feature ID: `page-editing-history`
- Category: `platform`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Provide Wikidot page editing modes, publishing behavior, source syntax workflow, and recoverable revision history.

## Implementation contract

- The public route, UI, persistent state, permissions, and user-visible side effects MUST match the documented contract.
- Account, site, category, page, and actor context MUST be enforced at the public service boundary.
- Browser behavior MUST be tested when the feature exposes navigation, dynamic controls, or intermediate visible states.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Public HTTP route and browser-visible UI
- Public service/API boundary for persistent state and permissions

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:editing-pages/source.wikidot.txt:1` through line 46 (canonical)

## Documentation-derived behavioral evidence

### faq:editing-pages (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/faq:editing-pages/source.wikidot.txt:1` through line 46  
SHA-256 of complete source file: `384456459f2247f06ae0a8f2ea44a31b92b2ab39cf2b903582a86298aecd76ff`

```wikidot
L0001 +++ How do I edit a page?
L0002 
L0003 [[embedvideo]]
L0004 <object width="425" height="344"><param name="movie" value="http://www.youtube.com/v/xvHgb1S7Qkw&hl=en&fs=1"></param><param name="allowFullScreen" value="true"></param><embed src="http://www.youtube.com/v/xvHgb1S7Qkw&hl=en&fs=1" type="application/x-shockwave-flash" allowfullscreen="true" width="425" height="344"></embed></object>
L0005 [[/embedvideo]]
L0006 [[size small]]##gray|Created by: [[*user samoore]]##[[/size]]
L0007 
L0008 +++ What is the "Wiki Syntax"?
L0009 
L0010 Wiki syntax (also known as Wiki markup, Wiki language, Wiki text) is a markup language as a simple alternative to HTML code that allows quick content creation. There is no common Wiki syntax but each Wiki engine (such as MediaWiki (Wikipedia), MoinMoin, TWiki and others) have their own specific syntax. In our (Wikidot) syntax e.g. to create link to a different website you simply write {{``[http://www.example.com visit this!]``}} instead of HTML:
L0011 {{``<a href="http://www.example.com">visit this!</a>``}}
L0012 
L0013 To learn more about the Wiki Syntax go to our [[[doc:wiki syntax|Wiki Syntax documentation]]].
L0014 
L0015 +++ So I have edited a page. Is the previous content lost?
L0016 
L0017 Not at all. All pages have "history" which consists of a series of __revisions__. Each change (title, content, rename, file upload etc.) creates a new revision. By clicking the //history// button at the bottom of a page you can browse the list of all revisions of a page.
L0018 
L0019 In principle the rule is **no content is lost**. This applies perfectly to pages - pages have history.
L0020 
L0021 The rule does not apply to uploaded files due to limited file storage size. So Users (who have permission) can replace/delete files.
L0022 
L0023 +++ Why are there 3 modes of editing a page?
L0024 
L0025 For convenience. It works fine when your pages are very long and editing the whole content is not always the best solution.
L0026 So the modes are:
L0027 
L0028 * whole page edit
L0029 * section edit 
L0030 * append 
L0031 
L0032 Each of these modes introduces a page edit lock such that no Users can both edit a page at the same time of their locks conflict. But as you might expect different users can edit non-overlapping sections of the same page at the same time. Or edit a section and append. These locks do not conflict.
L0033 
L0034 Anyway - with long pages it is much easier to use the section mode or append mode than editing the whole long page.
L0035 
L0036 +++ Why is there no WYSIWYG editor?
L0037 
L0038 WYSIWYG editor (What You See Is What You Get, such as [*http://tinymce.moxiecode.com/ TinyMCE] or [*http://www.fckeditor.net/ FCKeditor]) are still not very suitable for editing Wiki content. Although we would like to have an intuitive content editor the available ones do not meet our requirements since it is very difficult to produce well-structured documents with such tools.
L0039 
L0040 So what we ended with is "aided editing" - you still edit the raw source (which is a great advantage - produces clean, structured code) but the editor provides numerous buttons and wizards to make things a lot easier.
L0041 
L0042 +++ Why the buttons above the text input area do not work for me?
L0043 
L0044 Probably you use a non-standard browser. At the moment the interactive editor works with all major browsers, i.e. Mozilla Firefox, Opera and Internet Explorer.
L0045 
L0046 **Remember the buttons are only for your convenience and you can edit and save the content even if the buttons do not work!**
```
