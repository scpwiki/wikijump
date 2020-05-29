dojo.provide("dojox.date.tests.HebrewDate");
dojo.require("dojox.date.HebrewDate");
dojo.require("dojo.date");

tests.register("dojox.date.tests.HebrewDate", 
	[
		{
			// Test formatting and parsing of dates in various locales pre-built in dojo.cldr
			// NOTE: we can't set djConfig.extraLocale before bootstrapping unit tests, so directly
			// load resources here for specific locales:

			name: "date.locale",
			setUp: function(){
				var partLocaleList = ["en-us"];

				dojo.forEach(partLocaleList, function(locale){
					dojo.requireLocalization("dojo.cldr", "hebrew", locale);
				});
			},
			runTest: function(t){
			},
			tearDown: function(){
				//Clean up bundles that should not exist if
				//the test is re-run.
				delete dojo.cldr.nls.hebrew;
			}
		},
		{
			name: "toGregorian",
			runTest: function(t){
				var dateHebrew = new dojox.date.HebrewDate(5769, 2, 22); // Dec 19, 2008 -- is this right??
				var dateGregorian = dateHebrew.toGregorian();
				t.is(0, dojo.date.compare(new Date(2008, 11, 19), dateGregorian, "date"));
				// add exhaustive tests here
			}
		}/*,
		{
			name: "fromGregorian",
			runTest: function(t){
				var dateHebrew = new dojox.date.HebrewDate();
				var dateGregorian = new Date(2008,10,12);
				dateHebrew.fromGregorian(dateGregorian);
				t.is(0, dojo.date.compare(new Date(...), dateHebrew, "date"));
				// add more tests here
			}
		}*/
	]
);
