# MailForm Module

- Feature ID: `module-mailform`
- Category: `module`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the `MailForm` module interface, attributes, defaults, selection or side-effect behavior, templates, output, and documented limitations.

## Implementation contract

- The module dispatcher MUST recognize every documented module name and compatibility alias.
- The evaluator MUST implement documented attributes, aliases, defaults, limits, selection rules, permissions, side effects, and URL behavior.
- The renderer MUST implement documented templates, variables, wrappers, generated links, empty states, and interactive behavior.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.

## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- Saved-page or preview rendering through Deepwell's public page-view interface
- Framerail HTTP/browser boundary when the module is interactive or URL-driven

## Feature-specific implementation notes

- Module names and attribute names are compatibility-sensitive and must not be modernized.
- Examples are acceptance-test inputs, not permission to infer behavior beyond the documented case.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:mailform-module/source.wikidot.txt:1` through line 136 (canonical)

## Documentation-derived behavioral evidence

### doc-modules:mailform-module (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:mailform-module/source.wikidot.txt:1` through line 136  
SHA-256 of complete source file: `090c9e5eb7b89b658fc9097ae5c6f0ba740000337a9206ca4fc451e6c75b4f03`

```wikidot
L0001 ++ Description
L0002 
L0003 This module can be used to collect user input via a web form and receive the filled form via email.  The email can be sent to any registered Wikidot users.
L0004 
L0005 ++ Attributes
L0006 
L0007 ||~ attribute ||~ required ||~ allowed values ||~ default ||~ description ||
L0008 || {{to}} || no || user names || site admins || comma-delimited list of Wikidot user names ||
L0009 || {{button}} || no || any string || "send" || text displayed within the //send// button ||
L0010 || {{format}} || no || "csv" || none || chooses alternative format of serializing the form data ||
L0011 || {{title}} || no || text || "Wikidot.com - MailForm form data" || title of the email containing the submitted form ||
L0012 || {{successPage}} || no || valid page name || none || after the form is sent the browser will be redirected to the specified page. put a "thank you" there if you wish ;-) ||
L0013 
L0014 The names of the 'to' recipients may not contain spaces: to specify a user name that has spaces, replace each space with a hyphen or underscore.  If you do not specify a 'to' argument, the email will be sent to all site admins.
L0015 
L0016 The definition of the form must be enclosed within the [[module ... ]] ... [[/module]] tags. The full specification of how to do this:
L0017 
L0018 The definition of the fields is a nested list that looks like this:
L0019 [[code]]
L0020 # field1_name
L0021  * option1_name: value
L0022  * option2_name: value2
L0023   * suboption1_nam: value
L0024 # field2_name
L0025  * ...
L0026 [[/code]]
L0027 
L0028 where {{field_name}} is the alphanumeric identifier of the field, e.g. {{first_name}}. The options are:
L0029 ||~ option name ||~ required ||~ allowed values ||~ default ||~ description ||
L0030 || title || no, but recommended! || any string || field_name || title of the field displayed in the same row on the left ||
L0031 || type || no || text, textarea, select, checkbox || text || type of the input field ||
L0032 || size || no || integer || 30 || size of the input field ||
L0033 || default || no || value of the input || none || in case of the "text" or "textarea" fields it is just a string that appears inside the field; in case  of "select" it must be one of the option labels ||
L0034 || hint || no || any text || none || text that will be displayed below the input field
L0035 || options |||||||| only for "select", see below||
L0036 || rules |||||||| validation rules, see below ||
L0037 
L0038 +++ "Select" type
L0039 
L0040 If your field is a "select" field, you must also provide options for it. Do so as shown:
L0041 
L0042 [[code]]
L0043 # field_name 
L0044  * title: Gender
L0045  * type: select
L0046  * default: male
L0047  * options:
L0048   * male: Male
L0049   * female: Female
L0050   * option_name: Displayed value
L0051 [[/code]]
L0052 
L0053 where the "default: ..." is not required, but if provided it should match one of the names of the options.
L0054 
L0055 +++ Validation
L0056 
L0057 This module offers a powerful way to validate the input data. To use validation do:
L0058 
L0059 [[code]]
L0060 # field_name
L0061  * title: Validated field
L0062  * type: text
L0063  * rules:
L0064   * rule1_name: value
L0065   * rule2_name: value
L0066 [[/code]]
L0067 
L0068 where rules are:
L0069 
L0070 ||~ rule name ||~ allowed values ||~ description ||
L0071 || required || anything, e.g. "true" || if the field is required ||
L0072 || minLength || integer || does not allow strings shorter than limit ||
L0073 || maxLength || integer || does not allow strings longer than limit ||
L0074 || match || perl regular expression || checks the value against expression, e.g. {{/[a-z0-9]+/}} allows only lowercase letters and numbers ||
L0075 || number || anything, e.g. "true" || checks if numeric ||
L0076 || minValue || number || for numerical fields sets the lower limit ||
L0077 || maxValue || number || for numerical fields sets the upper limit ||
L0078 
L0079 ++ Examples
L0080 
L0081 Ok, suppose you are making some kind of conference registration and want to grab participants' data:
L0082 
L0083 [[code]]
L0084 
L0085 [[module MailForm title="New message from MailForm documentation page"]]
L0086 # name
L0087  * title: Your name
L0088  * type: text
L0089  * rules:
L0090   * required: true 
L0091 # affiliation
L0092  * title: Institute/Organization/Company
L0093  * hint: leave blank in none
L0094 # address 
L0095  * title: Address
L0096  * rules:
L0097   * required: true 
L0098   * minLength: 2
L0099 # address2
L0100  * title: Address (cont.)
L0101 # country
L0102  * title: Country
L0103  * rules:
L0104   * minLength: 2
L0105 # phone
L0106  * title: Phone
L0107 # email
L0108  * title: Email
L0109  * type: text
L0110  * rules:
L0111   * match: /^[_a-zA-Z0-9-]+(\.[_a-zA-Z0-9-]+)*@[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+$/
L0112 # payment
L0113  * title: I will pay by
L0114  * type: select
L0115  * options:
L0116   * creditcard: Credit card
L0117   * banktransfer: Bank wire transfer
L0118   * desk: At the registration desk
L0119   * na: Not applicable
L0120 # logging
L0121  * title: Please find me a hotel
L0122  * type: checkbox
L0123  * hint: we will contact you via email if yes
L0124 # comments
L0125  * type: textarea
L0126  * title: Extra comments
L0127  * rules:
L0128   * maxLength: 500
L0129 [[/module]]
L0130 [[/code]]
L0131 
L0132 **Note:** The following image depicts the output of the prior MailForm code and is non-interactive.
L0133 
L0134 [[div style="max-width: 472px;"]]
L0135 [[image wikidot-form-example.png alt="Example form requesting for contact information and a message." style="width: 100%;"]]
L0136 [[/div]]
```
