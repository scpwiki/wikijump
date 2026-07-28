# The 'date' field type

- Feature ID: `data-forms-date-field`
- Category: `data-forms`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Implement the documented data-form capability “The 'date' field type”, including its template syntax, storage meaning, editing behavior, display variables, validation, and integrations.

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

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:date-field/source.wikidot.txt:1` through line 138 (canonical)

## Documentation-derived behavioral evidence

### doc-data-forms:date-field (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-data-forms:date-field/source.wikidot.txt:1` through line 138  
SHA-256 of complete source file: `e43635adb502c5c706eaedbe23e96e01977c4f4cf7a9c1995fc762867300e88c`

```wikidot
L0001 [[div class="alert alert-info"]]
L0002 October 2, 2014
L0003 This is a work in progress. Some options still need to be tested before being added to this documentation.
L0004 [[/div]]
L0005 Defines a date input field that uses the [*http://api.jqueryui.com/datepicker/ jQuery UI Datepicker Widget] to select dates. It uses most of the options available that are of type //String// or //Int//. A basic datepicker can be added using just the //type// property:
L0006 
L0007 [[code]]
L0008 [[form]]
L0009 fields:
L0010   date-1:
L0011     type: date
L0012 [[/form]]
L0013 [[/code]]
L0014 
L0015 The specific properties you can use on a text field:
L0016 
L0017 * **width**: specifies the visible field width in columns (fixed spaced characters, more or less). If the //autoSize// option is enabled, it overrides this property setting.
L0018 * **options**: specifies the jQuery UI Datepicker Widget options to apply.
L0019 
L0020 Here is an example using a few of the available options:
L0021 [[code]]
L0022 [[form]]
L0023 fields:
L0024   mydate:
L0025     type: date
L0026     label: 'My Date Widget'
L0027     options:
L0028       appendText: ' This Demo is Cool!'
L0029       autoSize: true
L0030       changeYear: true
L0031       dateFormat: 'DD, d MM yy'
L0032       firstDay: 1
L0033       showOn: button
L0034       yearRange: '2014:2025'
L0035 [[/form]]
L0036 [[/code]]
L0037 The above example creates a datepicker calendar widget with the following options:
L0038 * **showon: button** adds a button after the input field and it must be clicked to open the datepicker.
L0039 * **autoSize: true** automatically sets the width of the input box to fit the date format defined in the //dateFormat// option.
L0040 * **changeYear: true** creates a dropdown selector of changing the year. This is useful if you don't want to make users click the previous/next links 12 times to move to another year.
L0041 * **dateFormat: 'DD, d MM yy** formats the date selected to something like "Wednesday, 1 October 2014". See the date format reference below for more details on date formats.
L0042 * **firstDay: 1** tells the calendar to use Monday as the first day of the week for the widget's calendar instead of the default of Sunday (0=Sunday, 1=Monday, ..., 6=Saturday).
L0043 * **yearRange: '2014:2025'** limits the year selection to the range specified.
L0044 
L0045 Dates in a date field are stored as a date number and displayed based on the //dateFormat// option you specifiy. add the //altFormat// option to use a different date format for your alternate date. If you want to save and use the date as //text// and not a number, you can use the //altField// option to place a text version of the date into another field in your data form. The example below will place a copy of the date selected in the **mydate** field as text into the **alt-date** field text box on the form. **@@%%form_data{alt-date}%%@@** will be stored in the data form as text.
L0046 ++ altDate/altFormat Example
L0047 [[code]]
L0048 [[form]]
L0049 fields:
L0050   mydate:
L0051     type: date
L0052     label: 'This date will fill in'
L0053     options:
L0054       appendText: ' altField Demo'
L0055       autoSize: true
L0056       changeYear: true
L0057       dateFormat: 'DD, d MM yy'
L0058       altField: 'input[name=field-alt-date]'
L0059       altFormat: 'm/d/yy'
L0060       yearRange: '2014:2025'
L0061   alt-date:
L0062     type: text
L0063     label: 'Filled in by date above'
L0064     width: 10
L0065 [[/form]]
L0066 [[/code]]
L0067 ++ Live Examples
L0068 A very comprehensive demo has been created on the [*http://bootstrap-playground.wikidot.com/dataform-dates:demo Bootstrap Playground site].
L0069 The **//_template//** page that drives the demo is located [*http://bootstrap-playground.wikidot.com/dataform-dates:_template here].
L0070 
L0071 ++ Datepicker Widget Options:
L0072 ||~ Option||~ Syntax||~ Description||~ Example||
L0073 ||**altField**||altField: 'input[name=field-<target field name>]'||The alternate field on your form to place a text copy of the date based on the current //dateFormat// option. Use //altFormat// to define a different date format.||altField: 'input[name=field-alt-date]'||
L0074 ||**altFormat**||altFormat : format string||Used to apply an alternate //dateFormat// to the //altDate// option||altFormat: 'm/d/yy'||
L0075 ||**appendText**||appendText : 'string to display'||The text to display after each date field.||appendText: ' This Demo is Cool!'||
L0076 ||**autoSize**||autoSize : true|false||Set to true to automatically resize the input field to accommodate dates in the current dateFormat.||autoSize: true||
L0077 ||**buttonImage**||buttonImage : 'url of image file'||URL of an image to use to display the datepicker when the showOn option is set to "button" or "both".||buttonImage: 'http://community.wikidot.com/local--files/files/calendar-icon.png'||
L0078 ||**buttonImageOnly**||buttonImageOnly : true|false||Whether the button image should be rendered by itself instead of inside a button element. This option is only relevant if the buttonImage option has also been set.||buttonImageOnly: false||
L0079 ||**buttonText**||buttonText : 'string to display'||The text to display on the trigger button. Use in conjunction with the showOn option set to "button" or "both". If //buttonImage// is set, the text becomes the alt value and is not directly displayed.||buttonText: 'Pick!'||
L0080 ||**changeMonth**||changeMonth : true|false||Whether the month should be rendered as a dropdown instead of text.||changeMonth: true||
L0081 ||**changeYear**||changeYear : true|false||Whether the year should be rendered as a dropdown instead of text. Use the //yearRange// option to control which years are made available for selection.||changeYear: true||
L0082 ||**closeText**||closeText : 'string to display'||The text to display for the close link. Use the //showButtonPanel: true// to display this button.||closeText: 'Abort Mission'||
L0083 ||**currentText**||currentText : 'string to display'||The text to display for the close link. Use //showButtonPanel: true// to display this button.||currentText: 'Go to Today'||
L0084 ||**dateFormat**||dateFormat : format string||The format for parsed and displayed dates. For a full list of the possible formats see the table below.||dateFormat: 'DD, MM yy'||
L0085 ||**dayNames**||dayNames : [array of names]||The list of long day names, starting from Sunday. Useful for languages other than English. Used with the //DD// date format option.||dayNames: [Sonntag, Montag, Dienstag, Mittwoch, Donnerstag, Freitag, Samstag]||
L0086 ||**dayNamesMin**||dayNamesMin : [array of names]||The list of minimised day names, starting from Sunday, for use as column headers within the datepicker. Useful for languages other than English.||dayNamesMin: [So, Mo, Di, Mi, Do, Fr, Sa]||
L0087 ||**dayNamesShort**||dayNamesShort : [array of names]||The list of abbreviated day names, starting from Sunday. Useful for languages other than English. Used with the //D// date format option.||dayNamesShort: [Son, Mon, Die, Mit, Don, Fre, Sam]||
L0088 ||**defaultDate**||defaultDate : 'date string'|+/- number of days from today|string of values and periods||Set the default date on first opening of the widget. Specify either an actual date via a string in the current dateFormat, or a number of days from today (e.g. +7) or a string of values and periods ('y' for years, 'm' for months, 'w' for weeks, 'd' for days, e.g. '+1m +7d'), or null for today.||defaultDate: '+1m -1d'||
L0089 ||**duration**||duration : number of milliseconds|slow|normal|fast||Control the speed at which the datepicker appears, it may be a time in milliseconds or a string representing one of the three predefined speeds ("slow", "normal", "fast").||duration : slow||
L0090 ||**firstDay**||firstDay: number||Set the first day of the week: Sunday is 0, Monday is 1, etc.||firstDay: 1||
L0091 ||**hideIfNoPrevNext**||hideIfNoPrevNext : true|false|\Normally the previous and next links are disabled (greyed out) when not applicable (as determined by the //minDate// and //maxDate// options). You can hide them altogether by setting this attribute to true.||hideIfNoPrevNext: true||
L0092 ||**isRTL**||isRTL : true|false||Whether the current language is drawn from right to left.||isRTL: true||
L0093 ||**maxDate**||maxDate : 'date string'|+/- number of days from today|string of values and periods.||The maximum selectable date.||- maxDate: '+2y -1m'||
L0094 ||**minDate**||minDate : 'date string'|+/- number of days from today|string of values and periods.||The minimum selectable date.||minDate: 0||
L0095 ||**monthNames**||monthNames : [array of names]||The list of full month names. Useful for languages other than English. Used with the //MM// date format option.||monthNames: [Jannuar, Februar, März, April, Mai, Juni, Juli, August, September, Oktober, November, Dezember]||
L0096 ||**monthNamesShort**||monthNamesShort : [array of names]||The list of abbreviated month names, as used in the month header and with the //M// date format option. Useful for languages other than English. ||monthNamesShort: [Jan, Feb, Mär, Apr, Mai, Jun, Jul, Aug, Sep, Okt, Nov, Dez]||
L0097 ||**nextText**||nextText : string||The text to display for the next month link. With the default styling, this value is used as the alt text when hovering over the icon.||nextText: 'Forward'||
L0098 ||**numberOfMonths**||numberOfMonths : number|[rows, columns]||The number of months to show at once. **Number**: The number of months to display in a single row. **Array**: An array defining the number of rows and columns to display.||numberOfMonths: [ 2, 3 ]||
L0099 ||**prevText**||prevText :  'string'||The text to display for the next month link. With the default styling, this value is used as the alt text when hovering over the icon.||prevText: 'Back'||
L0100 ||**shortYearCutoff**||shortYearCutoff : number|date string||The cutoff year for determining the century for a date (used in conjunction with dateFormat 'y'). Any dates entered with a year value less than or equal to the cutoff year are considered to be in the current century, while those greater than it are deemed to be in the previous century.||shortYearCutoff: '+20'||
L0101 ||**showAnim**||showAnim : show|slideDown|fadeIn||The name of the animation used to show and hide the datepicker. Use "show" (the default), "slideDown" or "fadeIn"[*http://api.jqueryui.com/category/effects/ Other effects] need testing and should be added as they are confirmed to work here.||showAnim: slideDown||
L0102 ||**showButtonPanel**||showButtonPanel : true|false||Whether to display a button pane underneath the calendar. The button pane contains two buttons, a Today button that links to the current day, and a Done button that closes the datepicker. The buttons' text can be customized using the currentText and closeText options respectively.||showButtonPanel: true||
L0103 ||**showCurrentAtPos**||showCurrentAtPos : number||When displaying multiple months via the //numberOfMonths// option, this option defines which position to display the current month in.|| showCurrentAtPos: 1 ||
L0104 ||**showMonthAfterYear**||showMonthAfterYear: true|false||Whether to show the month after the year in the header.||showMonthAfterYear: true||
L0105 ||**showOn**||showOn : focus|button|both||When the datepicker should appear. The datepicker can appear when the field receives focus ("focus"), when a button is clicked ("button"), or when either event occurs ("both").||showOn: both||
L0106 ||**showWeek**||showWeek: true|false||When true, a column is added to show the week of the year.||showWeek: true||
L0107 ||**stepMonths**||stepMonths: number||Set how many months to move when clicking the previous/next links.||stepMonths: 3||
L0108 ||**weekHeader**||weekheader: 'string'|| Text to display for the week number column header when the //showWeek// option is enabled.)||weekHeader: 'wk#'||
L0109 ||**yearRange**||yearRange: 'string'||The range of years displayed in the year drop-down: either relative to today's year ("-nn:+nn"), relative to the currently selected year ("c-nn:c+nn"), absolute ("nnnn:nnnn"), or combinations of these formats ("nnnn:-nn"). Note that this option only affects what appears in the drop-down, to restrict which dates may be selected use the //minDate// and/or //maxDate// options.||yearRange: '2010:2020'||
L0110 ||**yearSuffix**||yearSuffix: 'string'||Additional text to display after the year in the month headers.||yearSuffix: ' CE'||
L0111 
L0112 ++ Date Formats Reference
L0113 ||||**Datepicker Date Format Options**||||**Wikidot Date Format Options**||
L0114 ||||##880000|The format can be combinations of the following:##||||##880000|Use to display Data Form Datepicker date fields: _
L0115 **@@[[date@@ @@%%@@form_data{datefield}@@%%@@ @@format="%b %d, %Y"]]@@**##||
L0116 ||d||day of month (no leading zero)||%a||abbreviated weekday name (3 letters)||
L0117 ||dd||day of month (two digit)||{{%A}}||full weekday name||
L0118 ||o||day of the year (no leading zeros)||%b||abbreviated month name (3 letters)||
L0119 ||oo||day of the year (three digit)||%B||full month name||
L0120 ||D||day name short||%c||local date representation||
L0121 ||DD||day name long||%d||day of the month (01…31)||
L0122 ||m||month of year (no leading zero)||%D||is equivalent to "%m/%d/%y"||
L0123 ||mm||month of year (two digit)||%e||day of the month (1...9, 10…31)||
L0124 ||M||month name short||%H||hours (00...23)||
L0125 ||MM||month name long||%I||hours (00...12)||
L0126 ||y||year (two digit)||%m||month (01…12)||
L0127 ||yy||year (four digit)||%M||minutes (00...59)||
L0128 ||@||Unix timestamp (ms since 01/01/1970)||%O||//nn// seconds/minutes/hours/days||
L0129 ||!||Windows ticks (100ns since 01/01/0001)||%p||AM/PM||
L0130 ||'...'||literal text||%r||is equivalent to "%I:%M:%S %p"||
L0131 ||''||single quote||%R||is equivalent to "%H:%M"||
L0132 ||@<&nbsp;>@||@<&nbsp;>@||anything else||literal text||
L0133 ||@<&nbsp;>@||@<&nbsp;>@||%S||seconds (00...59)||
L0134 ||@<&nbsp;>@||@<&nbsp;>@||%T||is equivalent to "%H:%M:%S"||
L0135 ||@<&nbsp;>@||@<&nbsp;>@||{{%y}}||year (00...99)||
L0136 ||@<&nbsp;>@||@<&nbsp;>@||{{%Y}}||year (1970...2999)||
L0137 ||@<&nbsp;>@||@<&nbsp;>@||{{%z _
L0138 %Z}}||time zone||
```
