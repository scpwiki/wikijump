dojo.provide("dojox.form.DateTextBox");

dojo.require("dojox.widget.Calendar");
dojo.require("dijit.form._DateTimeTextBox");

dojo.declare(
	"dojox.form.DateTextBox",
	dijit.form._DateTimeTextBox,
	{
		// summary:
		//		A validating, serializable, range-bound date text box with a popup calendar

		// popupClass: String
		//  The popup widget to use. In this case, a calendar with Day, Month and Year views.
		popupClass: "dojox.widget.Calendar",
		_selector: "date",
		
		_open: function(){
			this.inherited(arguments);
			dojo.style(this._picker.domNode.parentNode, "position", "absolute");
		}
	}
);

dojo.declare(
	"dojox.form.DayTextBox",
	dojox.form.DateTextBox,
	{
		// summary:
		//		A validating, serializable, range-bound date text box with a popup calendar that contains just months.
		
		
		// popupClass: String
		//  The popup widget to use. In this case, a calendar with just a Month view.
		popupClass: "dojox.widget.DailyCalendar",
		
		format: function(value){return value.getDate();},
		validator: function(value) {
			var num = Number(value);
			var isInt = /(^-?\d\d*$)/.test(String(value));
			return value == "" || value == null || (isInt && num >= 1 && num <= 31);
		},		
		_open: function(){
			this.inherited(arguments);
			
			this._picker.onValueSelected = dojo.hitch(this, function(value){
				this.focus(); // focus the textbox before the popup closes to avoid reopening the popup
				setTimeout(dojo.hitch(this, "_close"), 1); // allow focus time to take
				dijit.form.TextBox.prototype._setValueAttr.call(this,value.getDate(), true, value.getDate());
			});			
		}
	}
);

dojo.declare(
	"dojox.form.MonthTextBox",
	dojox.form.DateTextBox, 
	{
		// summary:
		//		A validating, serializable, range-bound date text box with a popup calendar that contains just months.
		
		
		// popupClass: String
		//  The popup widget to use. In this case, a calendar with just a Month view.
		popupClass: "dojox.widget.MonthlyCalendar",
		validator: function(value) {
			var num = Number(value);
			var isInt = /(^-?\d\d*$)/.test(String(value));
			return value == "" || value == null || (isInt && num >= 1 && num <= 12);
		},
		_open: function(){
			this.inherited(arguments);
			
			this._picker.onValueSelected = dojo.hitch(this, function(value){
				this.focus(); // focus the textbox before the popup closes to avoid reopening the popup
				setTimeout(dojo.hitch(this, "_close"), 1); // allow focus time to take
				dijit.form.TextBox.prototype._setValueAttr.call(this,value + 1, true, value + 1);
			});			
		}
	}
);

dojo.declare(
	"dojox.form.YearTextBox",
	dojox.form.DateTextBox, 
	{
		// summary:
		//		A validating, serializable, range-bound date text box with a popup calendar that contains only years
		
		// popupClass: String
		//  The popup widget to use. In this case, a calendar with just a Year view.
		popupClass: "dojox.widget.YearlyCalendar",
		validator: function(value) {
			return value == "" || value == null || /(^-?\d\d*$)/.test(String(value));
		},
		
		_open: function(){
			this.inherited(arguments);
			
			this._picker.onValueSelected = dojo.hitch(this, function(value){
				this.focus(); // focus the textbox before the popup closes to avoid reopening the popup
				setTimeout(dojo.hitch(this, "_close"), 1); // allow focus time to take
				dijit.form.TextBox.prototype._setValueAttr.call(this,value, true, value);
			});						
		}
	}
);
