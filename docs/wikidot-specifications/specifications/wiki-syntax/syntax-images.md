# Images syntax

- Feature ID: `syntax-images`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented images syntax, including every documented form, option, output rule, and limitation.

## Implementation contract

- The parser MUST recognize every documented spelling and structural form in the evidence below.
- The renderer MUST produce the described visible text, HTML structure, links, and context-sensitive behavior.
- Whitespace, escaping, nesting, and malformed-input behavior MUST follow explicit documentation; unspecified cases require oracle evidence before widening acceptance.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:images/source.wikidot.txt:1` through line 94 (canonical)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:images (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:images/source.wikidot.txt:1` through line 94  
SHA-256 of complete source file: `c98c5de90fda596730c0e25a023b3cd0b1f764f5e37df33e279bf63d9deb61cb`

```wikidot
L0001 ++ [[# single]] Single images
L0002 
L0003 To insert an image into the page use the following syntax:
L0004 
L0005 [[code]]
L0006 [[image image-source attribute1="value1" attribute2="value2" ...]]
L0007 [[/code]]
L0008 
L0009 And here is the list of allowed attributes:
L0010 
L0011 ||~ attribute name ||~ allowed values ||~ example value ||~ description ||
L0012 || link || wiki page name or URL || {{@@"wiki-page"@@}} _
L0013 {{@@"http://www.example.com"@@}} _
L0014 {{@@"#anchor"@@}} _
L0015 {{@@"#"@@}} || makes image a link to another page or web address; this is __ignored__ when using Flickr as a source; prepend the link with '*' to make it open in a new window; can link to an [#toc22 Anchor] within a page; "#" prevents any actions when image is clicked ||
L0016 || alt || any string || {{@@"a photo of me"@@}} || Text substitution when image not available. It is also used by screen readers to describe an image. ||
L0017 || title || any string || {{@@"a photo of me"@@}} || Displays mouse-over text for the image. ||
L0018 || width || number of pixels || {{@@"200px"@@}} || forces  width of a image when displaying ||
L0019 || height || number of pixels || {{@@"200px"@@}} || forces  height of a image when displaying ||
L0020 || style || valid CSS style definition || {{@@"border: 1px solid red; padding: 2em;"@@}} || adds extra CSS style to the image ||
L0021 || class || CSS class || {{@@"mystyle"@@}} || forces the image CSS class - suggested use only with customized themes ||
L0022 || size || {{"square"}} - 75x75 pixels _
L0023 {{"thumbnail"}} - 100 on longest side _
L0024 {{"small"}} - 240 on longest side _
L0025 {{"medium"}} - 500 on longest side _
L0026 {{"medium640"}} - 640 on longest side (Flickr only)  _
L0027 {{"large"}} - 1024 on longest side (only for Flickr large images) _
L0028 {{"original"}} - original image (Flickr only) || any of allowed ;-) || displays a __resized__ image; great for thumbnails _
L0029 (transparency is lost and clicking the thumbnail opens the original image, unless link parameter is also supplied) _
L0030 if flickr is the source it pulls required size from a Flickr server; _
L0031 this option has effect only on local images or Flickr images||
L0032 
L0033 {{size}} attribute works very well with local files (attached to pages) not only with image files, but with e.g. PDF or  PostScript. See [http://www.imagemagick.org/script/formats.php this page] for more details.
L0034 
L0035 The //image-source// can be one of the following:
L0036 
L0037 ||~ source type ||~ format ||~ example value ||~ description ||
L0038 || URL address || any valid URL address || {{@@http://www.example.com/image.jpg@@}} || displays image from the web address ||
L0039 || file attachment (current page) || {{//filename//}} || {{@@exampleimage.jpg@@}} || displays image attached to the current page ||
L0040 || {{:first}} || {{:first}} || {{:first}} || displays first image attached to the current page (or nothing at all) ||
L0041 || file attachment (different page) || {{///another-page-name/filename//}} || {{@@/another-page/exampleimage.jpg@@}} || displays image attached to a different page ||
L0042 || [http://www.flickr.com flickr] image || {{@@flickr:@@//photoid//}} || {{@@flickr:83001279@@}} || displays image from Flickr and links to the original Flickr page ||
L0043 || [http://www.flickr.com flickr] image (private)|| {{@@flickr:@@//photoid_secret//}} || {{@@flickr:149666562_debab08866@@}} || displays image from Flickr and links to the original Flickr page; if the //secret// is provided the image is available despite being marked as non-public  ||
L0044 
L0045 To make the linked document in a new window you can either prepend the {{link}} attribute with '*' (e.g. {{@@link="*http://www.example.com"@@}} or prepend the {{src}} element with '*' (e.g. {{@@*flickr:149666562_debab08866@@}}, {{*//image-file//}} etc.) for images that automatically generate links.
L0046 
L0047 To choose horizontal alignment use:
L0048 
L0049 * {{[[=image...}} - centered image
L0050 * {{[[<image...}} - image on left
L0051 * {{[[>image...}} - image on right
L0052 * {{[[f<image...}} - image on left floating (surrounded by text)
L0053 * {{[[f>image...}} - image on right floating (surrounded by text)
L0054 
L0055 ++ [[# gallery]]Gallery of images
L0056 
L0057 To insert a series of images into a page content use the {{@@[[gallery]]@@}} element:
L0058 
L0059 [[code]]
L0060 [[gallery size="image-size"]]
L0061 [[/code]]
L0062 or
L0063 [[code]]
L0064 [[gallery size="image-size"]]
L0065 : image-source1 attribute1="value1" attribute2="value2" ...
L0066 : image-source2 attribute1="value1" attribute2="value2" ...
L0067 ...
L0068 [[/gallery]]
L0069 [[/code]]
L0070 
L0071 The allowed attributes within the {{@@[[gallery]]@@}} tag are:
L0072 ||~ attribute ||~ allowed values ||~ default ||~ description ||
L0073 || {{size}} || {{"square"}}, {{"thumbnail"}}, {{"small"}}, {{"medium"}} || {{"thumbnail"}} || sets the size of preview image _
L0074 this option has effect only on local images or Flickr images||
L0075 || {{order}} || {{"name"}}, {{"name desc"}}, {{"created_at"}}, {{"created_at desc"}} || {{"name"}} || sets order type ||
L0076 || {{viewer}} || {{"false"}}, {{"no"}}, {{"true"}}, {{"yes"}} || {{"yes"}} || disables LightBox viewer ||
L0077 
L0078 Order parameter also takes the following deprecated values: {{"nameDesc"}}, {{"dateAddedDesc"}} and {{"dateAdded"}}. For consistency with ListPages module it also takes the following values: {{"name desc desc"}} and {{"created_at desc desc"}} (meaning the same as without the "desc desc").
L0079 
L0080 If the {{@@[[gallery]]@@}} tag is invoked without a list of images it automatically displays rescaled images (thumbnails) of image files attached to the current page (without .pdf and .ps documents as gallery displays only images by default).
L0081 
L0082 If {{@@[[gallery]]@@}} is invoked with a list of images, only these images are displayed. {{image-source}} must not be a URL in this case. Allowed "per-image attributes are:
L0083 * {{link}} - URL or wiki page name (does not work with Flickr images to be o.k. with Flickr terms)
L0084 * {{alt}} - alternative text when the image is not available
L0085 
L0086 To make a document open in a new window the same rules as with a single image applies.
L0087 
L0088 The gallery by default is using LightBox to view images. It means that if you click on an image in the gallery, a very nice looking pop-up will show up with a possibility to scroll images forward / backward without reloading page / opening new tab or window. To disable LightBox view use parameter:
L0089 
L0090 {{@@[[gallery viewer="no"]]@@}} or {{@@[[gallery viewer="false"]]@@}}
L0091 
L0092 Also see [[[doc:flickrgallery-module | FlickrGallery module]]] if you wish to import images from Flickr.
L0093 
L0094 Put the @@[[gallery]]@@ tag on its own line or the parser will not recognize it.
```
