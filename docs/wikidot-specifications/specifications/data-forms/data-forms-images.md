# Images in Data Forms

- Feature ID: `data-forms-images`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “Images in Data Forms”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

## Implementation contract

- Category templates MUST recognize the documented field and layout syntax.
- Create and edit flows MUST validate, normalize, store, and redisplay field values as documented.
- Page rendering, template variables, CSS hooks, ListPages selection, and ordering MUST expose stored values as documented.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Data-form template parsing and saved page rendering
- Public create/edit/view flow and ListPages query behavior where documented

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:images/source.wikidot.txt:1` through line 75 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:images (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:images/source.wikidot.txt:1` through line 75  
SHA-256 of complete source file: `4a3c3b4b834da51328c23c2fafdcd85ccbedfbf3a3b679eae996f68d4188be35`

```wikidot
L0001 +++ Data form field
L0002 To upload an image to your dataform you need to use a **file** field.
L0003 
L0004 +++ Layout
L0005 You display the image using @@%%form_raw{field}%%@@,  __not__  @@%%form_data{field}%%@@.
L0006 
L0007 As with images on normal pages you can set parameters like a float or the width. For example:
L0008 @@[[f<image %%form_raw{field}%% width="150px"]]@@
L0009 
L0010 You can also combine data forms with the [*http://snippets.wikidot.com/code:image-box image box snippet] created by [[*user timothy-foster]] which will allow you to easily add a header, caption, a float left or float-right, specify the width and add a link for the image. In the data form use a file field for the image, a text field for the header and a text field for the caption.
L0011 
L0012 Then to display them, above the @@====@@ separator use the following snippet code with @@%%form_raw{field}%%@@ for the image and @@%%form_data{field}%%@@ for each of the header and caption. You do not need to have a value in each parameter line of the snippet code. An example of how it would look is below.
L0013 
L0014 @@[[include :snippets:image@@
L0015 @@|image=%%form_raw{field}%%@@
L0016 @@|width=150px@@
L0017 @@|float=right@@
L0018 @@|heading=%%form_data{field}%%@@
L0019 @@|caption=%%form_data{field}%%@@
L0020 @@|link=*%%form_data{bandwebsite}%%@@
L0021 @@]]@@
L0022 
L0023 +++ Where images are stored
L0024 On normal wiki pages you can upload an image to the page. This is not the case with data forms. When you upload an image using a data form, it places the image on its own page in the //file// category and the page is called the name of the image.  So for example if your user presses the browse button in the data form and uploads an image called **queen.jpg**, that image is saved on the page @@http://yoursite.wikidot.com/@@**file:queen**
L0025 
L0026 Although the //file//category is used by default for images, you can change the category the images on a data form are saved in. To do this use the category attribute as follows:
L0027 
L0028 [[code]]
L0029 [[form]]
L0030 fields:
L0031   eventimage:
L0032     label: Image
L0033     type: file
L0034     category: eventimages
L0035 [[/form]]
L0036 [[/code]]
L0037 
L0038 If you had specified this in the data form structure in your live template before uploading the queen.jpg image it would have saved it at @@http://yoursite.wikidot.com/@@**eventimages:queen**
L0039 
L0040 +++ Displaying a default image
L0041 
L0042 If you don't upload an image to a file field in a data form, older browsers like IE8 will show a box with a red x or similar. This doesn't look good and makes it seem that a mistake has been made. So, instead you can display a default image which will be displayed instead. This could be a blank image or a general image related to the site. If an image __is__ uploaded to the field in the data form then that image will be used instead..
L0043 
L0044 It needs a css module in the live template and this example also uses the image box snippet from *http://snippets.wikidot.com/code:image-box to display the relevant image:
L0045 
L0046 [[code]]
L0047 [!-- Following CSS module is needed for the default image code below --]
L0048 [[module CSS]]
L0049 .form-image-default%%form_raw{bandimage}%%{ display: block !important; }
L0050 .form-image%%form_raw{bandimage}%%{ display: none !important; }
L0051 [[/module]]
L0052 [[/code]]
L0053 
L0054 [!-- Following snippets code will add a default image if an image is not uploaded to the bandimage dataform field --]
L0055 
L0056 @@[[div class="form-image"]]@@
L0057 @@[[include :snippets:image@@
L0058 @@|image=%%form_raw{bandimage}%%@@
L0059 @@|width=150px@@
L0060 @@|float=right@@
L0061 @@|heading=@@
L0062 @@|caption=%%form_data{band_caption}%%@@
L0063 @@|link=%%form_data{band_link}%%@@
L0064 @@]]@@
L0065 @@[[/div]]@@
L0066 @@[[div class="form-image-default" style="display: none;"]]@@
L0067 @@[[include :snippets:image@@
L0068 @@|image=/css:theme/blankjpg@@
L0069 @@|width=150px@@
L0070 @@|float=right@@
L0071 @@|heading=@@
L0072 @@|caption=@@
L0073 @@|link=@@
L0074 @@]]@@
L0075 @@[[/div]]@@
```
