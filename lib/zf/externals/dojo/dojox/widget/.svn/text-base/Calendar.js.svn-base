dojo.provide("dojox.widget.Calendar");
dojo.experimental("dojox.widget.Calendar");

dojo.require("dijit._Calendar");
dojo.require("dijit._Container");

dojo.declare("dojox.widget._CalendarBase", [dijit._Widget, dijit._Templated, dijit._Container], {

	// templatePath: URL
	//  the path to the template to be used to construct the widget.
	templatePath: dojo.moduleUrl("dojox.widget","Calendar/Calendar.html"),

	// _views: Array
	//  The list of mixin views available on this calendar.
	_views: null,

	// useFx: Boolean
	//  Specifies if visual effects should be applied to the widget.
	//  The default behavior of the widget does not contain any effects.
	//  The dojox.widget.CalendarFx package is needed for these.
	useFx: true,

	// widgetsInTemplate: Boolean
	//  This widget is a container of other widgets, so this is true.
	widgetsInTemplate: true,

	// value: Date
	// 	the currently selected Date
	value: null,

	constructor: function(){
		// summary: constructor for the widget
		this._views = [];
		this.value = this.value || new Date();
	},

	postCreate: function(){
		// summary: Instantiates the mixin views
		this._height = dojo.style(this.containerNode, "height");
		this.displayMonth = new Date(this.value);

		var mixin = {
			parent: this,
			getValue: dojo.hitch(this, function(){return new Date(this.displayMonth);}),
			getLang: dojo.hitch(this, function(){return this.lang;}),
			isDisabledDate: dojo.hitch(this, this.isDisabledDate),
			getClassForDate: dojo.hitch(this, this.getClassForDate),
			addFx: this.useFx ? dojo.hitch(this, this.addFx) : function(){}
		};

		//Add the mixed in views.
		dojo.forEach(this._views, function(widgetType){
			var div = document.createElement("div");
			var widget = new widgetType(mixin, div);
			this.addChild(widget);

			//place the views's header node in the header of the main widget
			this.header.appendChild(widget.getHeader());

			//hide the header node of the widget
			dojo.style(widget.getHeader(), "display", "none");

			//Hide all views
			dojo.style(widget.domNode, "visibility", "hidden");

			//Listend for the values in a view to be selected
			dojo.connect(widget, "onValueSelected", this, "_onDateSelected");
			widget.setValue(this.value);
		}, this);

		if(this._views.length < 2) {
			dojo.style(this.header, "cursor", "auto");
		}

		this.inherited(arguments);

		// Cache the list of children widgets.
		this._children = this.getChildren();

		this._currentChild = 0;

		//Populate the footer with today's date.
		var today = new Date();

		this.footer.innerHTML = "Today: " + dojo.date.locale.format(today, {formatLength:'full',selector:'date', locale:this.lang});
		dojo.connect(this.footer, "onclick", this, "goToToday");

		dojo.style(this._children[0].domNode, "top", "0px");
		dojo.style(this._children[0].domNode, "visibility", "visible");
//		
		dojo.style(this._children[0].getHeader(), "display", "");

		var _this = this;

		var typematic = function(nodeProp, dateProp, adj){
			dijit.typematic.addMouseListener(_this[nodeProp], _this, function(count){				
				if(count >= 0){	_this._adjustDisplay(dateProp, adj);}
			}, 0.8, 500);
		};
		typematic("incrementMonth", "month", 1);
		typematic("decrementMonth", "month", -1);
	},

	addFx: function(query, fromNode) {
		// Stub function than can be overridden to add effects.
	},

	setValue: function(/*Date*/ value){
		// summary: set the current date and update the UI.  If the date is disabled, the selection will
		//	not change, but the display will change to the corresponding month.
		if(!this.value || dojo.date.compare(value, this.value)){
			value = new Date(value);
			this.displayMonth = new Date(value);
			if(!this.isDisabledDate(value, this.lang)){
				this.value = value;
				this.value.setHours(0,0,0,0);
				this.onChange(this.value);
			}
			this._children[this._currentChild].setValue(this.value);
			return true;
		}
		return false;
	},

	isDisabledDate: function(/*Date*/dateObject, /*String?*/locale){
		// summary:
		//	May be overridden to disable certain dates in the calendar e.g. `isDisabledDate=dojo.date.locale.isWeekend`
/*=====
		return false; // Boolean
=====*/
	},

	onValueSelected: function(/*Date*/date){
		// summary: a date cell was selected.  It may be the same as the previous value.
	},

	_onDateSelected: function(date, formattedValue){
		this.displayMonth = date;
		//Only change the selected value if it was chosen from the
		//first child.
		if(this.setValue(date)) {
			if (!this._transitionVert(-1)) {
				if (!formattedValue && formattedValue !== 0) {
					formattedValue = this.value;
				}
				this.onValueSelected(formattedValue);
			}
		}
	},

	onChange: function(/*Date*/date){
		// summary: called only when the selected date has changed
	},

	onHeaderClick: function(e) {
		// summary: Transitions to the next view.
		this._transitionVert(1);
	},

	goToToday: function(){
		this.setValue(new Date());
		this.onValueSelected(this.value);
	},

	_transitionVert: function(/*Number*/direction){
		// summary: Animates the views to show one and hide another, in a
		//   vertical direction.
		//   If 'direction' is 1, then the views slide upwards.
		//   If 'direction' is -1, the views slide downwards.
		var curWidget = this._children[this._currentChild];
		var nextWidget = this._children[this._currentChild + direction];
		if(!nextWidget){return false;}
		
		var height = dojo.style(this.containerNode, "height");
		nextWidget.setValue(this.displayMonth);

		dojo.style(curWidget.header, "display", "none");
		dojo.style(nextWidget.header, "display", "");
		dojo.style(nextWidget.domNode, "top", (height * -1) + "px");
		dojo.style(nextWidget.domNode, "visibility", "visible");

		this._currentChild += direction;

		var height1 = height * direction;
		var height2 = 0;
		dojo.style(nextWidget.domNode, "top", (height1 * -1) + "px");

		// summary: Slides two nodes vertically.
		var anim1 = dojo.animateProperty({node: curWidget.domNode, properties: {top: height1}});
		var anim2 = dojo.animateProperty({node: nextWidget.domNode, properties: {top: height2}});

		anim1.play();
		anim2.play();
		return true;
	},

	_slideTable: function(/*String*/widget, /*Number*/direction, /*Function*/callback){
		// summary: Animates the horizontal sliding of a table.
		var table = widget.domNode;

		//Clone the existing table
		var newTable = table.cloneNode(true);
		var left = dojo.style(table, "width");

		table.parentNode.appendChild(newTable);

		//Place the existing node either to the left or the right of the new node,
		//depending on which direction it is to slide.
		dojo.style(table, "left", (left * direction) + "px");

		//Call the function that generally populates the new cloned node with new data.
		//It may also attach event listeners.
		callback();

		//Animate the two nodes.
		var anim1 = dojo.animateProperty({node: newTable, properties:{left: left * direction * -1}, duration: 500, onEnd: function(){
			newTable.parentNode.removeChild(newTable);
		}});
		var anim2 = dojo.animateProperty({node: table, properties:{left: 0}, duration: 500});

		anim1.play();
		anim2.play();
	},

	_addView: function(view) {
		//Insert the view at the start of the array.
		this._views.push(view);
	},

	getClassForDate: function(/*Date*/dateObject, /*String?*/locale){
		// summary:
		//  May be overridden to return CSS classes to associate with the date entry for the given dateObject,
		//  for example to indicate a holiday in specified locale.

/*=====
		return ""; // String
=====*/
	},

	_adjustDisplay: function(/*String*/part, /*int*/amount, noSlide){
		// summary: This function overrides the base function defined in dijit._Calendar.
		//   It changes the displayed years, months and days depending on the inputs.
		var child = this._children[this._currentChild];

		var month = this.displayMonth = child.adjustDate(this.displayMonth, amount);

		this._slideTable(child, amount, function(){
			child.setValue(month);
		});
	}
});

dojo.declare("dojox.widget._CalendarView", dijit._Widget, {
	// summary: Base implementation for all view mixins.
	//   All calendar views should extend this widget.
	headerClass: "",

	cloneClass: function(clazz, n){
		// summary: Clones all nodes with the class 'clazz' in a widget
		var template = dojo.query(clazz, this.domNode)[0];
		for(var i=0; i<n; i++){
			template.parentNode.appendChild(template.cloneNode(true));
		}
	},

	_setText: function(node, text){
		// summary: sets the text inside a node
		while(node.firstChild){
			node.removeChild(node.firstChild);
		}
		node.appendChild(dojo.doc.createTextNode(text));
	},

	getHeader: function() {
		// summary: returns the header node of a view. If none exists,
		//   an empty DIV is created and returned.
		if (!this.header) {
			this.header = document.createElement("div");
			dojo.addClass(this.header, this.headerClass);
		}
		return this.header;
	},

	onValueSelected: function(date){
		//Stub function called when a date is selected
	},

	adjustDate: function(date, amount){
		// summary: Adds or subtracts values from a date.
		//   The unit, e.g. "day", "month" or "year", is
		//   specified in the "datePart" property of the
		//   calendar view mixin.
		return dojo.date.add(date, this.datePart, amount);
	}
});

dojo.declare("dojox.widget._CalendarDay", null, {
	// summary: Mixin for the dojox.widget.Calendar which provides
	//  the standard day-view. A single month is shown at a time.
	parent: null,

	constructor: function(){
		this._addView(dojox.widget._CalendarDayView);
	}
});

dojo.declare("dojox.widget._CalendarDayView", [dojox.widget._CalendarView, dijit._Templated], {
	// summary: View class for the dojox.widget.Calendar.
	//   Adds a view showing every day of a single month to the calendar.
	//   This should not be mixed in directly with dojox.widget._CalendarBase.
	//   Instead, use dojox.widget._CalendarDay

	// templatePath: URL
	//  the path to the template to be used to construct the widget.
	templatePath: dojo.moduleUrl("dojox.widget","Calendar/CalendarDay.html"),

	// datePart: String
	//  Specifies how much to increment the displayed date when the user
	//  clicks the array button to increment of decrement the view.
	datePart: "month",

	// dayWidth: String
	//  Specifies the type of day name to display.  "narrow" causes just one letter to be shown.
	dayWidth: "narrow",

	postCreate: function(){
		// summary: Constructs the calendar view.
		this.cloneClass(".dijitCalendarDayLabelTemplate", 6);
		this.cloneClass(".dijitCalendarDateTemplate", 6);

		// now make 6 week rows
		this.cloneClass(".dijitCalendarWeekTemplate", 5);

		// insert localized day names in the header
		var dayNames = dojo.date.locale.getNames('days', this.dayWidth, 'standAlone', this.getLang());
		var dayOffset = dojo.cldr.supplemental.getFirstDayOfWeek(this.getLang());

		// Set the text of the day labels.
		dojo.query(".dijitCalendarDayLabel", this.domNode).forEach(function(label, i){
			this._setText(label, dayNames[(i + dayOffset) % 7]);
		}, this);

		// Add visual effects to the view, if any has been specified.
		this.addFx(".dijitCalendarDateTemplate div", this.domNode);
	},

	_onDayClick: function(e) {
		// summary: executed when a day value is clicked.
		var date = this.getValue();
		var p = e.target.parentNode;
		var c = "dijitCalendar";
		var d = dojo.hasClass(p, c + "PreviousMonth") ? -1 :
							(dojo.hasClass(p, c + "NextMonth") ? 1 : 0);
		if (d){date = dojo.date.add(date, "month", d)}

		date.setDate(e.target._date);
		this.value = date;
		this.parent._onDateSelected(date);
	},

	setValue: function(value) {
		//Change the day values
		this._populateDays();
	},

	_populateDays: function() {
		// summary: Fills the days of the current month.
		var month = this.getValue();
		month.setDate(1);
		var firstDay = month.getDay();
		var daysInMonth = dojo.date.getDaysInMonth(month);
		var daysInPreviousMonth = dojo.date.getDaysInMonth(dojo.date.add(month, "month", -1));
		var today = new Date();
		var selected = this.value;

		var dayOffset = dojo.cldr.supplemental.getFirstDayOfWeek(this.getLang());
		if(dayOffset > firstDay){ dayOffset -= 7; }

		// Iterate through dates in the calendar and fill in date numbers and style info
		dojo.query(".dijitCalendarDateTemplate", this.domNode).forEach(function(template, i){
			i += dayOffset;
			var date = new Date(month);
			var number, clazz = "dijitCalendar", adj = 0;

			if(i < firstDay){
				number = daysInPreviousMonth - firstDay + i + 1;
				adj = -1;
				clazz += "Previous";
			}else if(i >= (firstDay + daysInMonth)){
				number = i - firstDay - daysInMonth + 1;
				adj = 1;
				clazz += "Next";
			}else{
				number = i - firstDay + 1;
				clazz += "Current";
			}

			if(adj){
				date = dojo.date.add(date, "month", adj);
			}
			date.setDate(number);

			if(!dojo.date.compare(date, today, "date")){
				clazz = "dijitCalendarCurrentDate " + clazz;
			}

			if(!dojo.date.compare(date, selected, "date")){
				clazz = "dijitCalendarSelectedDate " + clazz;
			}

			if(this.isDisabledDate(date, this.getLang())){
				clazz = " dijitCalendarDisabledDate " + clazz;
			}

			var clazz2 = this.getClassForDate(date, this.getLang());
			if(clazz2){
				clazz += clazz2 + " " + clazz;
			}

			template.className =  clazz + "Month dijitCalendarDateTemplate";
			template.dijitDateValue = date.valueOf();
			var label = dojo.query(".dijitCalendarDateLabel", template)[0];
			this._setText(label, date.getDate());
			label._date = date.getDate();
		}, this);

		// Fill in localized month name
		var monthNames = dojo.date.locale.getNames('months', 'wide', 'standAlone', this.getLang());
		this._setText(this.monthLabelNode, monthNames[month.getMonth()]);
		this._setText(this.yearLabelNode, month.getFullYear());
	}
});


dojo.declare("dojox.widget._CalendarMonth", null, {
	// summary: Mixin class for adding a view listing all 12 months of the year to the
	//   dojox.widget._CalendarBase

	// headerClass: String
	//  Specifies the CSS class to apply to the header node for this view.
	headerClass: "dojoxCalendarYearHeader",

	constructor: function(){
		// summary: Adds a dojox.widget._CalendarMonthView view to the calendar widget.
		this._addView(dojox.widget._CalendarMonthView);
	}
});

dojo.declare("dojox.widget._CalendarMonthView", [dojox.widget._CalendarView, dijit._Templated], {
	// summary: A Calendar view listing the 12 months of the year

	// templatePath: URL
	//  the path to the template to be used to construct the widget.
	templatePath: dojo.moduleUrl("dojox.widget","Calendar/CalendarMonth.html"),

	// datePart: String
	//  Specifies how much to increment the displayed date when the user
	//  clicks the array button to increment of decrement the view.
	datePart: "year",

	postCreate: function(){
		// summary: Constructs the view
		this.cloneClass(".dojoxCalendarMonthTemplate", 3);
		this.cloneClass(".dojoxCalendarMonthGroupTemplate", 2);
		this._populateMonths();

		// Add visual effects to the view, if any have been mixed in
		this.addFx(".dojoxCalendarMonthLabel", this.domNode);
	},

	setValue: function(value){
		this.header.innerHTML = value.getFullYear();
	},

	_getMonthNames: function(format) {
		// summary: Returns localized month names
		this._monthNames  = this._monthNames || dojo.date.locale.getNames('months', format, 'standAlone', this.getLang());
		return this._monthNames;
	},

	_populateMonths: function() {
		// summary: Populate the month names using the localized values.
		var monthNames = this._getMonthNames('abbr');
		dojo.query(".dojoxCalendarMonthLabel", this.monthContainer).forEach(dojo.hitch(this, function(node, cnt){
			this._setText(node, monthNames[cnt]);
		}));
	},

	onClick: function(evt) {
		// summary: Handles clicks on month names
		if(!dojo.hasClass(evt.target, "dojoxCalendarMonthLabel")){dojo.stopEvent(evt); return;}
		var month = evt.target.parentNode.cellIndex + (evt.target.parentNode.parentNode.rowIndex * 4);
		var date = this.getValue();
		date.setMonth(month);
		this.onValueSelected(date, month);
	}
});

dojo.declare("dojox.widget._CalendarYear", null, {
	// summary: Mixin class for adding a view listing 12 years to the
	//   dojox.widget._CalendarBase
	parent: null,

	constructor: function(){
		// summary: Adds a dojox.widget._CalendarYearView view to the
		//  dojo.widget._CalendarBase widget.
		this._addView(dojox.widget._CalendarYearView);
	}
});

dojo.declare("dojox.widget._CalendarYearView", [dojox.widget._CalendarView, dijit._Templated], {
	// summary: A Calendar view listing 12 years

	// templatePath: URL
	//  the path to the template to be used to construct the widget.
	templatePath: dojo.moduleUrl("dojox.widget","Calendar/CalendarYear.html"),

	postCreate: function(){
		// summary: Constructs the view
		this.cloneClass(".dojoxCalendarYearTemplate", 3);
		this.cloneClass(".dojoxCalendarYearGroupTemplate", 2);
		this._populateYears();
		this.addFx(".dojoxCalendarYearLabel", this.domNode);
	},

	setValue: function(value){
		this._populateYears(value.getFullYear());
	},

	_populateYears: function(year) {
		// summary: Writes the years to display to the view
		if(this._displayedYear && year && year >= this._displayedYear - 5 && year <= this._displayedYear + 6){
			return;
		}

		// summary: Fills the list of years with a range of 12 numbers, with the current year
		//   being the 6th number.
		this._displayedYear = year || this.getValue().getFullYear();
		var firstYear = this._displayedYear - 5;

		dojo.query(".dojoxCalendarYearLabel", this.yearContainer).forEach(dojo.hitch(this, function(node, cnt){
			this._setText(node, firstYear + (cnt));
		}));
		this._setText(this.getHeader(), firstYear + " - " + (firstYear + 11));
	},

	adjustDate: function(date, amount){
		// summary: Adjusts the value of a date. It moves it by 12 years each time.
		return dojo.date.add(date, "year", amount * 12);
	},

	onClick: function(evt) {
		// summary: Handles clicks on year values.
		if(!dojo.hasClass(evt.target, "dojoxCalendarYearLabel")){dojo.stopEvent(evt); return;}
		var year = Number(evt.target.innerHTML);
		var date = this.getValue();
		date.setYear(year);
		this.onValueSelected(date, year);
	}
});

dojo.declare("dojox.widget.Calendar",
	[dojox.widget._CalendarBase,
	 dojox.widget._CalendarDay,
	 dojox.widget._CalendarMonth,
	 dojox.widget._CalendarYear], {
	 	// summary: The standard Calendar. It includes day, month and year views.
		//  No visual effects are included.
	 }
);

dojo.declare("dojox.widget.DailyCalendar",
	[dojox.widget._CalendarBase,
	 dojox.widget._CalendarDay], {
	 	// summary: A calendar with only a daily view.
	 }
);

dojo.declare("dojox.widget.MonthlyCalendar",
	[dojox.widget._CalendarBase,
	 dojox.widget._CalendarMonth], {
	 	// summary: A calendar with only a month view.
	 }
);
dojo.declare("dojox.widget.YearlyCalendar",
	[dojox.widget._CalendarBase,
	 dojox.widget._CalendarYear], {
	 	// summary: A calendar with only a year view.
	 }
);
